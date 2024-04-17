mod gesture_or_signcode;
mod location;
mod normal_or_raw;
mod photo;
mod qrcode;

use cxsign_activity::sign::{
    LocationSign, NormalQrCodeSign, QrCodeSign, RefreshQrCodeSign, SignResult, SignTrait,
};
use cxsign_error::Error;
use cxsign_types::Location;
use cxsign_user::Session;
pub use gesture_or_signcode::*;
pub use location::*;
use log::warn;
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
impl SetLocationTrait for RefreshQrCodeSign {
    fn set_location(&mut self, location: Location) {
        self.set_location(location)
    }
}
impl SetLocationTrait for NormalQrCodeSign {
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
    locations: &[Location],
) -> Result<SignResult, Error> {
    let state = match sign.pre_sign(session).map_err(Error::from)? {
        SignResult::Susses => SignResult::Susses,
        SignResult::Fail { .. } => {
            let mut state = SignResult::Fail {
                msg: "所有位置均不可用".into(),
            };
            for location in locations.iter().rev() {
                sign.set_location(location.clone());
                match unsafe { sign.sign_unchecked(session) }.map_err(Error::from)? {
                    SignResult::Susses => {
                        state = SignResult::Susses;
                        break;
                    }
                    SignResult::Fail { msg } => {
                        warn!(
                            "用户[{}]在二维码签到[{}]中尝试位置[{}]时失败！失败信息：[{:?}]",
                            session.get_stu_name(),
                            sign.as_inner().name,
                            location,
                            msg
                        );
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
