mod base;
mod gesture;
mod location;
mod normal;
mod photo;
mod qrcode;
mod signcode;

pub use base::*;
pub use gesture::*;
pub use location::*;
pub use normal::*;
pub use photo::*;
pub use qrcode::*;
pub use signcode::*;

use crate::course::Course;
use crate::protocol;
use user::session::Session;
use serde::Deserialize;

pub trait SignTrait: Ord {
    fn is_ready_for_sign(&self) -> bool {
        true
    }
    fn is_valid(&self) -> bool;
    fn get_attend_info(&self, session: &Session) -> Result<SignState, ureq::Error>;
    fn guess_sign_result(&self, text: &str) -> SignResult {
        match text {
            "success" => SignResult::Susses,
            msg => SignResult::Fail {
                msg: if msg.is_empty() {
                    "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。"
                } else {
                    msg
                }
                .into(),
            },
        }
    }
    unsafe fn sign_internal(&self, session: &Session) -> Result<SignResult, ureq::Error>;
    fn sign(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        if self.is_ready_for_sign() {
            unsafe { self.sign_internal(session) }
        } else {
            Ok(SignResult::Fail {
                msg: "签到未准备好！".to_string(),
            })
        }
    }
    fn get_sign_detail(active_id: &str, session: &Session) -> Result<SignDetail, ureq::Error> {
        #[derive(Deserialize)]
        struct GetSignDetailR {
            #[serde(alias = "ifPhoto")]
            is_photo_sign: i64,
            #[serde(alias = "ifRefreshEwm")]
            is_refresh_qrcode: i64,
            #[serde(alias = "signCode")]
            sign_code: Option<String>,
        }
        let r = protocol::sign_detail(session, active_id)?;
        let GetSignDetailR {
            is_photo_sign,
            is_refresh_qrcode,
            sign_code,
        } = r.into_json().unwrap();
        Ok(SignDetail {
            is_photo: is_photo_sign > 0,
            is_refresh_qrcode: is_refresh_qrcode > 0,
            c: if let Some(c) = sign_code {
                c
            } else {
                "".into()
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
    // 拍照签到
    Photo(PhotoSign),
    // 普通签到
    Normal(NormalSign),
    // 二维码签到
    QrCode(QrCodeSign),
    // 手势签到
    Gesture(GestureSign),
    // 位置签到
    Location(LocationSign),
    // 签到码签到
    Signcode(SigncodeSign),
    // 未知
    Unknown(BaseSign),
}
impl SignTrait for Sign {
    fn is_ready_for_sign(&self) -> bool {
        match self {
            Sign::Photo(a) => a.is_ready_for_sign(),
            Sign::Normal(a) => a.is_ready_for_sign(),
            Sign::QrCode(a) => a.is_ready_for_sign(),
            Sign::Gesture(a) => a.is_ready_for_sign(),
            Sign::Location(a) => a.is_ready_for_sign(),
            Sign::Signcode(a) => a.is_ready_for_sign(),
            Sign::Unknown(a) => a.is_ready_for_sign(),
        }
    }
    fn is_valid(&self) -> bool {
        match self {
            Sign::Photo(a) => a.is_valid(),
            Sign::Normal(a) => a.is_valid(),
            Sign::QrCode(a) => a.is_valid(),
            Sign::Gesture(a) => a.is_valid(),
            Sign::Location(a) => a.is_valid(),
            Sign::Signcode(a) => a.is_valid(),
            Sign::Unknown(a) => a.is_valid(),
        }
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, ureq::Error> {
        match self {
            Sign::Photo(a) => a.get_attend_info(session),
            Sign::Normal(a) => a.get_attend_info(session),
            Sign::QrCode(a) => a.get_attend_info(session),
            Sign::Gesture(a) => a.get_attend_info(session),
            Sign::Location(a) => a.get_attend_info(session),
            Sign::Signcode(a) => a.get_attend_info(session),
            Sign::Unknown(a) => a.get_attend_info(session),
        }
    }

    unsafe fn sign_internal(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        unsafe {
            match self {
                Sign::Photo(a) => a.sign_internal(session),
                Sign::Normal(a) => a.sign_internal(session),
                Sign::QrCode(a) => a.sign_internal(session),
                Sign::Gesture(a) => a.sign_internal(session),
                Sign::Location(a) => a.sign_internal(session),
                Sign::Signcode(a) => a.sign_internal(session),
                Sign::Unknown(a) => a.sign_internal(session),
            }
        }
    }
}
#[derive(Debug)]
pub enum SignResult {
    Susses,
    Fail { msg: String },
}
impl SignResult {
    pub fn is_susses(&self) -> bool {
        match self {
            SignResult::Susses => true,
            SignResult::Fail { .. } => false,
        }
    }
}

#[derive(num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i64)]
pub enum SignState {
    #[default]
    未签 = 0,
    签到成功 = 1,
    教师代签 = 2,
    请假 = 4,
    缺勤 = 5,
    病假 = 7,
    事假 = 8,
    迟到 = 9,
    早退 = 10,
    签到已过期 = 11,
    公假 = 12,
}

#[derive(Debug)]
pub struct SignActivityRaw {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignDetail {
    is_photo: bool,
    is_refresh_qrcode: bool,
    c: String,
}
