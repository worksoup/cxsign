use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cxsign_internal::{
    store::{tables::LocationTable, DataBase, DataBaseTableTrait},
    Error, Location, LocationWithRange, QrCodeSign, Session, SignResult, SignTrait, SignnerTrait,
};
use tauri::{AppHandle, Manager};

use crate::CurrentSignState;

pub struct TauriQrCodeSignner {
    app_handle: AppHandle,
    db: Arc<Mutex<DataBase>>,
}
impl TauriQrCodeSignner {
    pub fn new(db: Arc<Mutex<DataBase>>, app_handle: AppHandle) -> Self {
        Self { db, app_handle }
    }
}

impl<'l> SignnerTrait<QrCodeSign> for TauriQrCodeSignner {
    type ExtData<'e> = Location;

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut QrCodeSign,
        _: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let sessions = self.app_handle.state::<CurrentSignState>().accounts.clone();
        let unames = self
            .app_handle
            .state::<crate::state::CurrentSignUnamesState>()
            .0
            .clone();
        let sessions_lock = sessions.lock().unwrap();
        let preset_location = sessions_lock
            .iter()
            .next()
            .map(|s| LocationWithRange::from_log(s, &sign.as_inner().course))
            .transpose()?
            .map(|mut m| m.remove(&sign.as_inner().active_id))
            .flatten();
        drop(sessions_lock);
        log::info!("获取预设位置。");
        let app_handle_ = self.app_handle.clone();
        let db = Arc::clone(&self.db);
        let mut location = Arc::new(Mutex::new(Location::get_none_location()));
        log::info!("初始化位置信息处理程序。");
        let location_info_thread_handle = if let Some(preset_location) = preset_location {
            // global_locations.append(&mut course_locations);
            // let locations = global_locations;
            // let locations = Arc::new(Mutex::new(locations));
            location = Arc::new(Mutex::new(preset_location.to_shifted_location()));
            let preset_location = Arc::new(Mutex::new(preset_location));
            let location = Arc::clone(&location);
            std::thread::spawn(move || {
                let app = app_handle_.clone();
                app_handle_.listen("sign:qrcode:location", move |p| {
                    log::info!("reserve: `sign:qrcode:location`.");
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app.unlisten(p.id());
                        return;
                    }
                    let crate::LocationSignnerInfo { location_str } = p.payload().parse().unwrap();
                    let mut preset_location = preset_location.lock().unwrap().to_location();
                    match cxsign_internal::utils::location_str_to_location(
                        &db.lock().unwrap(),
                        &location_str,
                    ) {
                        Ok(位置) => {
                            preset_location.set_addr(位置.get_addr());
                            (*location.lock().unwrap()) = 位置;
                        }
                        Err(位置字符串) => {
                            if !位置字符串.is_empty() {
                                preset_location.set_addr(&位置字符串);
                            }
                            (*location.lock().unwrap()) = preset_location;
                        }
                    }
                    log::info!("reserve: `sign:qrcode:location`, end.");
                });
            })
        } else {
            let db_ = self.db.lock().unwrap();
            let table = LocationTable::from_ref(&db_);
            let mut global_locations = table.get_location_list_by_course(-1);
            let mut course_locations =
                table.get_location_list_by_course(sign.as_inner().course.get_id());
            global_locations.append(&mut course_locations);
            let locations = global_locations;
            if let Some(location_) = locations.into_iter().next() {
                location = Arc::new(Mutex::new(location_.clone()));
                let location_fallback = location_.clone();
                let location = Arc::clone(&location);
                std::thread::spawn(move || {
                    let app = app_handle_.clone();
                    app_handle_.listen("sign:qrcode:location", move |p| {
                        log::info!("reserve: `sign:qrcode:location`.");
                        if p.payload() == "\"quit\"" {
                            log::info!("quit");
                            app.unlisten(p.id());
                            return;
                        }
                        let crate::LocationSignnerInfo { location_str } =
                            p.payload().parse().unwrap();
                        match cxsign_internal::utils::location_str_to_location(
                            &db.lock().unwrap(),
                            &location_str,
                        ) {
                            Ok(位置) => {
                                (*location.lock().unwrap()) = 位置;
                            }
                            Err(_) => {
                                (*location.lock().unwrap()) = location_fallback.clone();
                            }
                        }
                        log::info!("reserve: `sign:qrcode:location`, end.");
                    });
                })
            } else {
                let location = Arc::clone(&location);
                std::thread::spawn(move || {
                    let app = app_handle_.clone();
                    app_handle_.listen("sign:qrcode:location", move |p| {
                        log::info!("reserve: `sign:qrcode:location`.");
                        if p.payload() == "\"quit\"" {
                            log::info!("quit");
                            app.unlisten(p.id());
                            return;
                        }
                        let crate::LocationSignnerInfo { location_str } =
                            p.payload().parse().unwrap();
                        match cxsign_internal::utils::location_str_to_location(
                            &db.lock().unwrap(),
                            &location_str,
                        ) {
                            Ok(位置) => {
                                (*location.lock().unwrap()) = 位置;
                            }
                            _ => {}
                        }
                        log::info!("reserve: `sign:qrcode:location`, end.");
                    });
                })
            }
        };
        let app_handle = self.app_handle.clone();
        let sign = sign.clone();
        log::info!("初始化二维码签到处理程序。");
        let enc_thread_handle = std::thread::spawn(move || {
            let unames = Arc::clone(&unames);
            let app = app_handle.clone();
            app_handle.listen("sign:qrcode:enc", move |p| {
                log::info!("reserve: `sign:qrcode:enc`.");
                if p.payload() == "\"quit\"" {
                    log::info!("quit");
                    app.unlisten(p.id());
                    return;
                }
                let mut enc = p.payload().trim_matches(|c| c == '"').to_string();
                if enc.is_empty() {
                    enc = get_enc().unwrap_or_default();
                }
                let mut sign = sign.clone();
                sign.set_enc(enc);
                let unames = unames.lock().unwrap();
                let mut handles = Vec::new();
                let sessions_lock = sessions.lock().unwrap();
                let sessions_ = sessions_lock.clone();
                drop(sessions_lock);
                // 这种写法会有死锁。应该是获取锁的顺序不确定。
                // let locations = [
                //     location.lock().unwrap().clone(),
                //     location_fallback.lock().unwrap().clone(),
                // ];
                let location1 = location.lock().unwrap().clone();
                for session in sessions_.iter().filter(|a| unames.contains(a.get_uname())) {
                    let mut sign = sign.clone();
                    let session = session.clone();
                    let app = app.clone();
                    let location = location1.clone();
                    let h = std::thread::spawn(move || {
                        match <Self as SignnerTrait<QrCodeSign>>::sign_single(
                            &mut sign, &session, location,
                        )
                        .unwrap_or_else(|e| SignResult::Fail { msg: e.to_string() })
                        {
                            SignResult::Susses => {
                                app.emit("sign:susses", session.get_uname()).unwrap();
                            }
                            SignResult::Fail { msg } => {
                                app.emit("sign:fail", [session.get_uname(), &msg]).unwrap()
                            }
                        };
                    });
                    handles.push(h);
                }
                for h in handles {
                    h.join().unwrap();
                }
                log::info!("reserve: `sign:qrcode:enc`, end.");
            });
        });
        location_info_thread_handle.join().unwrap();
        enc_thread_handle.join().unwrap();
        fn get_enc() -> Result<String, Error> {
            let enc = {
                #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
                let enc = crate::tools::capture_screen_for_enc().unwrap_or_default();
                #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
                let enc = Default::default();
                enc
            };
            Ok(enc)
        }
        Ok(Default::default())
    }

    fn sign_single(
        sign: &mut QrCodeSign,
        session: &Session,
        location: Location,
    ) -> Result<SignResult, Error> {
        let r = sign.pre_sign(session).map_err(Error::from)?;
        sign.set_location(location);
        Ok(unsafe { sign.sign_unchecked(session, r) }.map_err(Error::from)?)
    }
}
