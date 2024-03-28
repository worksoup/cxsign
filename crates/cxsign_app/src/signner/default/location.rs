use crate::SignnerTrait;
use cxsign_activity::sign::{LocationSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable, LocationWithRange};
use cxsign_user::Session;
use log::{info, warn};
use std::collections::HashMap;

pub struct DefaultLocationSignner<'a> {
    db: &'a DataBase,
    location_str: &'a Option<String>,
    auto_get_location: bool,
    no_rand_shift: bool,
}

impl<'a> DefaultLocationSignner<'a> {
    pub fn new(
        db: &'a DataBase,
        location_str: &'a Option<String>,
        auto_get_location: bool,
        no_rand_shift: bool,
    ) -> Self {
        Self {
            db,
            location_str,
            auto_get_location,
            no_rand_shift,
        }
    }
}
impl<'a> SignnerTrait<LocationSign> for DefaultLocationSignner<'a> {
    type ExtData = &'a Vec<Location>;

    fn sign<'b, Sessions: Iterator<Item = &'b Session> + Clone>(
        &self,
        sign: &mut LocationSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'b Session, SignResult>, Error> {
        let locations = if let Ok(位置) =
            crate::utils::location_str_to_location(self.db, self.location_str)
        {
            println!("解析位置成功，将使用位置 `{}` 签到。", 位置);
            vec![位置]
        } else {
            let table = LocationTable::from_ref(self.db);
            let mut 位置列表 = table.get_location_list_by_course(sign.as_inner().course.get_id());
            let mut 全局位置列表 = table.get_location_list_by_course(-1);
            位置列表.append(&mut 全局位置列表);
            位置列表
        };
        let mut map = HashMap::new();
        for session in sessions {
            let r = self.sign_single(sign, session, &locations)?;
            map.insert(session, r);
        }
        Ok(map)
    }

    fn sign_single(
        &self,
        sign: &mut LocationSign,
        session: &Session,
        locations: &'a Vec<Location>,
    ) -> Result<SignResult, Error> {
        sign.set_has_range(self.auto_get_location);
        let r = match sign.pre_sign(session)? {
            SignResult::Susses => SignResult::Susses,
            SignResult::Fail { msg } => {
                if self.auto_get_location
                    && let Some(location_with_range) = LocationWithRange::find_in_html(&msg)
                {
                    let location = if self.no_rand_shift {
                        location_with_range.to_location()
                    } else {
                        location_with_range.to_shifted_location()
                    };
                    info!(
                        "用户[{}]签到使用位置：{}.",
                        session.get_stu_name(),
                        location
                    );
                    sign.set_location(location);
                    unsafe { sign.sign_unchecked(session) }?
                } else {
                    let mut result = SignResult::Fail {
                        msg: "所有位置均不可用".into(),
                    };
                    for location in locations {
                        match {
                            sign.set_location(location.clone());
                            unsafe { sign.sign_unchecked(session) }?
                        } {
                            SignResult::Susses => {
                                result = SignResult::Susses;
                                break;
                            }
                            SignResult::Fail { msg } => {
                                if msg == "您已签到过了".to_owned() {
                                    info!(
                                        "用户[{}]: 您已经签过[{}]了！",
                                        session.get_stu_name(),
                                        sign.as_inner().name,
                                    );
                                    result = SignResult::Susses;
                                    break;
                                } else {
                                    warn!(
                                        "用户[{}]在位置签到[{}]中尝试位置[{}]时失败！失败信息：[{:?}]",
                                        session.get_stu_name(),
                                        sign.as_inner().name,
                                        location,
                                        msg,
                                    );
                                }
                            }
                        }
                    }
                    result
                }
            }
        };
        Ok(r)
    }
}
