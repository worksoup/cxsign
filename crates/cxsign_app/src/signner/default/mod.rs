mod gesture_or_signcode;
mod location;
mod normal_or_raw;
mod photo;
mod qrcode;

use cxsign_activity::sign::{LocationSign, QrCodeSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_types::Location;
use cxsign_user::Session;
pub use gesture_or_signcode::*;
pub use location::*;
use log::{info, warn};
pub use normal_or_raw::*;
pub use photo::*;
pub use qrcode::*;

trait SetLocationTrait {
    fn set_location(&mut self, location: Location);
}
impl SetLocationTrait for QrCodeSign {
    fn set_location(&mut self, location: Location) {
        self.set_location(location)
    }
}
impl SetLocationTrait for LocationSign {
    fn set_location(&mut self, location: Location) {
        self.set_location(location)
    }
}
fn location_or_qrcode_signner_sign_single<R: SignTrait + SetLocationTrait>(
    sign: &mut R,
    session: &Session,
    locations: &Vec<Location>,
) -> Result<SignResult, Error> {
    let state = match sign.pre_sign(session)? {
        SignResult::Susses => SignResult::Susses,
        SignResult::Fail { .. } => {
            let mut state = SignResult::Fail {
                msg: "所有位置均不可用".into(),
            };
            for location in locations.iter().rev() {
                sign.set_location(location.clone());
                match unsafe { sign.sign_unchecked(session) }? {
                    SignResult::Susses => {
                        state = SignResult::Susses;
                        break;
                    }
                    SignResult::Fail { msg } => {
                        if msg == "您已签到过了".to_owned() {
                            state = SignResult::Susses;
                            info!(
                                "用户[{}]: 您已经签过[{}]了！",
                                session.get_stu_name(),
                                sign.as_inner().name,
                            );
                            break;
                        } else {
                            warn!(
                                "用户[{}]在二维码签到[{}]中尝试位置[{}]时失败！失败信息：[{:?}]",
                                session.get_stu_name(),
                                sign.as_inner().name,
                                location,
                                msg
                            );
                        }
                    }
                };
            }
            state
        }
    };
    Ok(state)
}
