use crate::SignnerTrait;
use cxsign_activity::sign::{LocationSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_types::{Location, LocationWithRange};
use cxsign_user::Session;
use log::{info, warn};
use std::collections::HashMap;

pub struct DefaultLocationSignner<'a> {
    locations: &'a Vec<Location>,
    auto_get_location: bool,
    no_rand_shift: bool,
}

impl<'a> DefaultLocationSignner<'a> {
    pub fn new(locations: &'a Vec<Location>, auto_get_location: bool, no_rand_shift: bool) -> Self {
        Self {
            locations,
            auto_get_location,
            no_rand_shift,
        }
    }
}
impl SignnerTrait<LocationSign> for DefaultLocationSignner<'_> {
    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &self,
        sign: &mut LocationSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let mut map = HashMap::new();
        for session in sessions {
            let a = self.sign_single(sign, session)?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(&self, sign: &mut LocationSign, session: &Session) -> Result<SignResult, Error> {
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
                    for location in self.locations {
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
                                    warn!(
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
