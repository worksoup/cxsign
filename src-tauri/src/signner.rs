use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cxsign_internal::{
    store::{tables::LocationTable, DataBase, DataBaseTableTrait},
    Error, Location, LocationWithRange, NormalQrCodeSign, RefreshQrCodeSign, Session, SignResult,
    SignTrait, SignnerTrait,
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

macro_rules! signner_trait_impl {
    ($qrcode_sign_type:ty) => {
        impl<'l> SignnerTrait<$qrcode_sign_type> for TauriQrCodeSignner {
            type ExtData<'e> = [Location; 2];

            fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
                &mut self,
                sign: &mut $qrcode_sign_type,
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
                let mut location_fallback = Arc::clone(&location);
                log::info!("初始化位置信息处理程序。");
                let location_info_thread_handle = if let Some(preset_location) = preset_location {
                    // global_locations.append(&mut course_locations);
                    // let locations = global_locations;
                    // let locations = Arc::new(Mutex::new(locations));
                    location = Arc::new(Mutex::new(preset_location.to_shifted_location()));
                    location_fallback = Arc::clone(&location);
                    let preset_location = Arc::new(Mutex::new(preset_location));
                    let location = Arc::clone(&location);
                    let location_fallback = Arc::clone(&location_fallback);
                    std::thread::spawn(move || {
                        let app = app_handle_.clone();
                        app_handle_.listen("sign:qrcode:location", move |p| {
                            log::info!("reserve: `sign:qrcode:location`.");
                            if p.payload() == "\"quit\"" {
                                log::info!("quit");
                                app.unlisten(p.id());
                                return;
                            }
                            let crate::LocationSignnerInfo {
                                location_str,
                                no_random_shift,
                            } = p.payload().parse().unwrap();
                            let mut preset_location = if no_random_shift {
                                preset_location.lock().unwrap().to_location()
                            } else {
                                preset_location.lock().unwrap().to_shifted_location()
                            };
                            match cxsign_internal::utils::location_str_to_location(
                                &db.lock().unwrap(),
                                &location_str,
                            ) {
                                Ok(位置) => {
                                    preset_location.set_addr(位置.get_addr());
                                    (*location.lock().unwrap()) = 位置;
                                    (*location_fallback.lock().unwrap()) = preset_location;
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
                                let crate::LocationSignnerInfo {
                                    location_str,
                                    no_random_shift: _,
                                } = p.payload().parse().unwrap();
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
                                let crate::LocationSignnerInfo {
                                    location_str,
                                    no_random_shift: _,
                                } = p.payload().parse().unwrap();
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
                        let location2 = location_fallback.lock().unwrap().clone();
                        let locations = [location1, location2];
                        for session in sessions_.iter().filter(|a| unames.contains(a.get_uname())) {
                            let mut sign = sign.clone();
                            let session = session.clone();
                            let app = app.clone();
                            let locations = locations.clone();
                            let h = std::thread::spawn(move || {
                                match <Self as SignnerTrait<$qrcode_sign_type>>::sign_single(
                                    &mut sign, &session, locations,
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
                        #[cfg(any(
                            target_os = "linux",
                            target_os = "windows",
                            target_os = "macos"
                        ))]
                        let enc = crate::tools::capture_screen_for_enc().unwrap_or_default();
                        #[cfg(not(any(
                            target_os = "linux",
                            target_os = "windows",
                            target_os = "macos"
                        )))]
                        let enc = Default::default();
                        enc
                    };
                    Ok(enc)
                }
                Ok(Default::default())
            }

            fn sign_single(
                sign: &mut $qrcode_sign_type,
                session: &Session,
                locations: [Location; 2],
            ) -> Result<SignResult, Error> {
                let state = match sign.pre_sign(session).map_err(Error::from)? {
                    SignResult::Susses => SignResult::Susses,
                    SignResult::Fail { .. } => {
                        let mut state = SignResult::Fail {
                            msg: "所有位置均不可用".into(),
                        };
                        for location in locations.into_iter().rev() {
                            sign.set_location(location);
                            match unsafe { sign.sign_unchecked(session) }.map_err(Error::from)? {
                                SignResult::Susses => {
                                    state = SignResult::Susses;
                                    break;
                                }
                                SignResult::Fail { msg } => {
                                    if msg == *"签到失败，请重新扫描。" {
                                        state = SignResult::Fail {
                                            msg: "签到失败，请重新扫描。".to_string(),
                                        };
                                        break;
                                    }
                                }
                            };
                        }
                        state
                    }
                };
                Ok(state)
            }
        }
    };
}
signner_trait_impl!(RefreshQrCodeSign);
signner_trait_impl!(NormalQrCodeSign);
