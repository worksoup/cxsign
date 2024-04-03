use crate::{
    default::location_or_qrcode_signner_sign_single, utils::pic_dir_or_path_to_pic_path,
    SignnerTrait,
};
use cxsign_activity::sign::{
    NormalQrCodeSign, QrCodeSign, RefreshQrCodeSign, SignResult, SignTrait,
};
use cxsign_error::Error;
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable, LocationWithRange};
use cxsign_user::Session;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
trait GetLocationAndEncTraitInternal: SignTrait {
    fn get_locations<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        db: &DataBase,
        location_str: &Option<String>,
        no_rand_shift: bool,
        sessions: Sessions,
    ) -> Result<Vec<Location>, Error> {
        match crate::utils::location_str_to_location(db, location_str) {
            Ok(位置) => Ok(vec![位置]),
            Err(位置字符串) => {
                let mut 预设位置列表 = HashMap::new();
                if let Some(session) = sessions.clone().next() {
                    预设位置列表 = LocationWithRange::from_log(session, &self.as_inner().course)?;
                }
                let 预设位置 = 预设位置列表.get(&self.as_inner().active_id).map(|l| {
                    if no_rand_shift {
                        l.to_location()
                    } else {
                        l.to_shifted_location()
                    }
                });
                let table = LocationTable::from_ref(db);
                let locations = if 位置字符串.is_empty() {
                    let mut 全局位置列表 = table.get_location_list_by_course(-1);
                    let mut 位置列表 =
                        table.get_location_list_by_course(self.as_inner().course.get_id());
                    全局位置列表.append(&mut 位置列表);
                    if let Some(location) = 预设位置 {
                        全局位置列表.push(location)
                    } else if 全局位置列表.is_empty() {
                        // 必须保证有一个地址。否则下面循环无法进入。
                        全局位置列表.push(Location::new("", "", "", ""))
                    }
                    全局位置列表
                } else {
                    let 预设位置 = 预设位置
                        .map(|l| Location::new(&位置字符串, l.get_lon(), l.get_lat(), l.get_alt()));
                    if let Some(location) = 预设位置 {
                        vec![location]
                    } else {
                        // 必须保证有一个地址。否则下面循环无法进入。
                        vec![cxsign_types::Location::new("", "", "", "")]
                    }
                };
                Ok(locations)
            }
        }
    }
    fn is_refresh(&self) -> bool;
    fn get_enc(
        &mut self,
        path: &Option<PathBuf>,
        enc: &Option<String>,
        precisely: bool,
    ) -> Result<String, Error> {
        let enc = if let Some(enc) = enc {
            enc.clone()
        } else if let Some(pic) = path {
            if std::fs::metadata(pic).unwrap().is_dir() {
                if let Some(pic) = pic_dir_or_path_to_pic_path(pic)?
                    && let Some(enc) =
                        crate::utils::pic_path_to_qrcode_result(pic.to_str().unwrap())
                {
                    enc
                } else {
                    return Err(Error::EncError(
                        "图片文件夹下没有图片（`png` 或 `jpg` 文件）！".to_owned(),
                    ));
                }
            } else if let Some(enc) = crate::utils::pic_path_to_qrcode_result(pic.to_str().unwrap())
            {
                enc
            } else {
                return Err(Error::EncError("二维码中没有 `enc` 参数！".to_owned()));
            }
        } else if let Some(enc) = crate::utils::capture_screen_for_enc(self.is_refresh(), precisely)
        {
            enc
        } else {
            return Err(Error::EncError("截屏时未获取到 `enc` 参数！".to_owned()));
        };
        Ok(enc)
    }
}
impl GetLocationAndEncTraitInternal for RefreshQrCodeSign {
    fn is_refresh(&self) -> bool {
        true
    }
}

impl GetLocationAndEncTraitInternal for NormalQrCodeSign {
    fn is_refresh(&self) -> bool {
        false
    }
}
impl GetLocationAndEncTraitInternal for QrCodeSign {
    fn is_refresh(&self) -> bool {
        match self {
            QrCodeSign::RefreshQrCodeSign(_) => true,
            QrCodeSign::NormalQrCodeSign(_) => false,
        }
    }
}
pub struct DefaultQrCodeSignner<'a> {
    db: &'a DataBase,
    location_str: &'a Option<String>,
    path: &'a Option<PathBuf>,
    enc: &'a Option<String>,
    precisely: bool,
    no_rand_shift: bool,
}
impl<'a> DefaultQrCodeSignner<'a> {
    pub fn new(
        db: &'a DataBase,
        location_str: &'a Option<String>,
        path: &'a Option<PathBuf>,
        enc: &'a Option<String>,
        precisely: bool,
        no_rand_shift: bool,
    ) -> Self {
        Self {
            db,
            location_str,
            path,
            enc,
            precisely,
            no_rand_shift,
        }
    }
}
impl<'l> SignnerTrait<QrCodeSign> for DefaultQrCodeSignner<'l> {
    type ExtData<'e> = Vec<Location>;

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut QrCodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        match sign {
            QrCodeSign::RefreshQrCodeSign(sign) => Self::sign(self, sign, sessions),
            QrCodeSign::NormalQrCodeSign(sign) => Self::sign(self, sign, sessions),
        }
    }

    fn sign_single(
        sign: &mut QrCodeSign,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error> {
        location_or_qrcode_signner_sign_single(sign, session, &extra_data)
    }
}

impl<'l> SignnerTrait<NormalQrCodeSign> for DefaultQrCodeSignner<'l> {
    type ExtData<'e> = &'e Vec<Location>;

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut NormalQrCodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let locations = sign.get_locations(
            self.db,
            self.location_str,
            self.no_rand_shift,
            sessions.clone(),
        )?;
        let enc = sign.get_enc(self.path, self.enc, self.precisely)?;
        sign.set_enc(enc);
        let mut map = HashMap::new();
        for session in sessions {
            let state = Self::sign_single(sign, session, &locations)?;
            map.insert(session, state);
        }
        Ok(map)
    }

    fn sign_single(
        sign: &mut NormalQrCodeSign,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error> {
        location_or_qrcode_signner_sign_single(sign, session, extra_data)
    }
}

impl<'l> SignnerTrait<RefreshQrCodeSign> for DefaultQrCodeSignner<'l> {
    type ExtData<'e> = Vec<Location>;

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut RefreshQrCodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let locations = sign.get_locations(
            self.db,
            self.location_str,
            self.no_rand_shift,
            sessions.clone(),
        )?;
        let enc = sign.get_enc(self.path, self.enc, self.precisely)?;
        sign.set_enc(enc);
        let sessions = sessions.collect::<Vec<&'a Session>>();
        let mut map = HashMap::new();
        let index_result_map = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        for (sessions_index, session) in sessions.clone().into_iter().enumerate() {
            let locations = locations.clone();
            let index_result_map = Arc::clone(&index_result_map);
            let mut sign = sign.clone();
            let session = session.clone();
            let h = std::thread::spawn(move || {
                let a = Self::sign_single(&mut sign, &session, locations)
                    .unwrap_or_else(|e| SignResult::Fail { msg: e.to_string() });
                index_result_map.lock().unwrap().insert(sessions_index, a);
            });
            handles.push(h);
        }
        for h in handles {
            h.join().unwrap();
        }
        for (i, r) in Arc::into_inner(index_result_map)
            .unwrap()
            .into_inner()
            .unwrap()
        {
            map.insert(sessions[i], r);
        }
        Ok(map)
    }

    fn sign_single(
        sign: &mut RefreshQrCodeSign,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error> {
        location_or_qrcode_signner_sign_single(sign, session, &extra_data)
    }
}
