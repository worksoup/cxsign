mod gesture;
mod location;
mod normal;
mod photo;
mod qrcode;
mod raw;
mod signcode;

pub use gesture::*;
pub use location::*;
pub use normal::*;
pub use photo::*;
pub use qrcode::*;
pub use raw::*;
pub use signcode::*;

use cxsign_types::Course;
use cxsign_user::Session;
use serde::Deserialize;

pub trait SignTrait: Ord {
    fn as_inner(&self) -> &RawSign;
    fn as_inner_mut(&mut self) -> &mut RawSign;
    fn is_ready_for_sign(&self) -> bool {
        true
    }
    fn is_valid(&self) -> bool {
        let time = std::time::SystemTime::from(
            chrono::DateTime::from_timestamp(self.as_inner().start_timestamp, 0).unwrap(),
        );
        let one_hour = std::time::Duration::from_secs(7200);
        self.as_inner().status_code == 1
            && std::time::SystemTime::now().duration_since(time).unwrap() < one_hour
    }
    fn get_sign_state(&self, session: &Session) -> Result<SignState, ureq::Error> {
        let r = crate::protocol::get_attend_info(&session, &self.as_inner().active_id)?;
        #[derive(Deserialize)]
        struct Status {
            status: i64,
        }
        #[derive(Deserialize)]
        struct Data {
            data: Status,
        }
        let Data {
            data: Status { status },
        } = r.into_json().unwrap();
        Ok(status.into())
    }
    fn get_sign_detail(&self, session: &Session) -> Result<SignDetail, ureq::Error> {
        RawSign::get_sign_detail(&self.as_inner().active_id, session)
    }
    fn guess_sign_result_by_text(&self, text: &str) -> SignResult {
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
    fn pre_sign(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        self.as_inner().pre_sign(session)
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        unsafe { self.as_inner().sign_unchecked(session) }
    }
    fn sign(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        if self.is_ready_for_sign() {
            unsafe { self.sign_unchecked(session) }
        } else {
            Ok(SignResult::Fail {
                msg: "签到未准备好！".to_string(),
            })
        }
    }
    fn pre_sign_and_sign(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        let r = self.pre_sign(session);
        if let Ok(a) = r.as_ref()
            && !a.is_susses()
        {
            self.sign(session)
        } else {
            r
        }
    }
}
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
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
    Unknown(RawSign),
}
impl SignTrait for Sign {
    fn as_inner(&self) -> &RawSign {
        match self {
            Sign::Photo(a) => a.as_inner(),
            Sign::Normal(a) => a.as_inner(),
            Sign::QrCode(a) => a.as_inner(),
            Sign::Gesture(a) => a.as_inner(),
            Sign::Location(a) => a.as_inner(),
            Sign::Signcode(a) => a.as_inner(),
            Sign::Unknown(a) => a.as_inner(),
        }
    }
    fn as_inner_mut(&mut self) -> &mut RawSign {
        match self {
            Sign::Photo(a) => a.as_inner_mut(),
            Sign::Normal(a) => a.as_inner_mut(),
            Sign::QrCode(a) => a.as_inner_mut(),
            Sign::Gesture(a) => a.as_inner_mut(),
            Sign::Location(a) => a.as_inner_mut(),
            Sign::Signcode(a) => a.as_inner_mut(),
            Sign::Unknown(a) => a.as_inner_mut(),
        }
    }
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

    fn get_sign_state(&self, session: &Session) -> Result<SignState, ureq::Error> {
        match self {
            Sign::Photo(a) => a.get_sign_state(session),
            Sign::Normal(a) => a.get_sign_state(session),
            Sign::QrCode(a) => a.get_sign_state(session),
            Sign::Gesture(a) => a.get_sign_state(session),
            Sign::Location(a) => a.get_sign_state(session),
            Sign::Signcode(a) => a.get_sign_state(session),
            Sign::Unknown(a) => a.get_sign_state(session),
        }
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        unsafe {
            match self {
                Sign::Photo(a) => a.sign_unchecked(session),
                Sign::Normal(a) => a.sign_unchecked(session),
                Sign::QrCode(a) => a.sign_unchecked(session),
                Sign::Gesture(a) => a.sign_unchecked(session),
                Sign::Location(a) => a.sign_unchecked(session),
                Sign::Signcode(a) => a.sign_unchecked(session),
                Sign::Unknown(a) => a.sign_unchecked(session),
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
