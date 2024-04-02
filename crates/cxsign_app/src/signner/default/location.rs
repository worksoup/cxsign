use crate::signner::default::location_or_qrcode_signner_sign_single;
use crate::SignnerTrait;
use cxsign_activity::sign::{LocationSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable, LocationWithRange};
use cxsign_user::Session;
use std::collections::HashMap;

pub struct DefaultLocationSignner<'a> {
    db: &'a DataBase,
    location_str: &'a Option<String>,
    no_rand_shift: bool,
}

impl<'a> DefaultLocationSignner<'a> {
    pub fn new(db: &'a DataBase, location_str: &'a Option<String>, no_rand_shift: bool) -> Self {
        Self {
            db,
            location_str,
            no_rand_shift,
        }
    }
}
impl<'a> SignnerTrait<LocationSign> for DefaultLocationSignner<'a> {
    type ExtData<'e> = Vec<Location>;

    fn sign<'b, Sessions: Iterator<Item = &'b Session> + Clone>(
        &mut self,
        sign: &mut LocationSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'b Session, SignResult>, Error> {
        let locations = match crate::utils::location_str_to_location(self.db, self.location_str) {
            Ok(位置) => {
                vec![位置]
            }
            Err(位置字符串) => {
                let mut 预设位置列表 = HashMap::new();
                if let Some(session) = sessions.clone().next() {
                    预设位置列表 = LocationWithRange::from_log(session, &sign.as_inner().course)
                        .map_err(Error::from)?;
                }
                let 预设位置 = 预设位置列表.get(&sign.as_inner().active_id).map(|l| {
                    if self.no_rand_shift {
                        l.to_location()
                    } else {
                        l.to_shifted_location()
                    }
                });
                let table = LocationTable::from_ref(self.db);
                let locations = if 位置字符串.is_empty() {
                    let mut 全局位置列表 = table.get_location_list_by_course(-1);
                    let mut 位置列表 =
                        table.get_location_list_by_course(sign.as_inner().course.get_id());
                    全局位置列表.append(&mut 位置列表);
                    if let Some(location) = 预设位置 {
                        sign.set_has_range(true);
                        全局位置列表.push(location)
                    }
                    全局位置列表
                } else if let Some(mut location) = 预设位置 {
                    location.set_addr(&位置字符串);
                    sign.set_has_range(true);
                    vec![location]
                } else {
                    vec![]
                };
                locations
            }
        };
        let mut map = HashMap::new();
        for session in sessions {
            let r = Self::sign_single(sign, session, locations.clone())?;
            map.insert(session, r);
        }
        Ok(map)
    }

    fn sign_single(
        sign: &mut LocationSign,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error> {
        location_or_qrcode_signner_sign_single(sign, session, &extra_data)
    }
}
