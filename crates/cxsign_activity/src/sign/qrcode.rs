use crate::protocol;
use crate::sign::{RawSign, SignResult, SignTrait};
use cxsign_types::Location;
use cxsign_user::Session;
use log::{debug, info};
use serde::{Deserialize, Serialize};

fn sign_unchecked<T: SignTrait>(
    sign: &T,
    enc: &str,
    location: &Option<Location>,
    session: &Session,
) -> Result<SignResult, Box<ureq::Error>> {
    let url = protocol::qrcode_sign_url(session, enc, sign.as_inner().active_id.as_str(), location);
    let r = protocol::ureq_get(session, &url)?;
    let result = match sign.guess_sign_result_by_text(&r.into_string().unwrap()) {
        SignResult::Susses => SignResult::Susses,
        SignResult::Fail { msg } => {
            if msg.starts_with("validate_") {
                let enc2 = &msg[9..msg.len()];
                debug!("enc2: {enc2:?}");
                let url = url + "&enc2=" + enc2;
                // captcha validate.
                // get token.
                let url_param = cxsign_captcha::utils::tmp_solver(session)?.get_validate_info();
                if let Some(url_param) = url_param {
                    let url = url + "&validate=" + &url_param;
                    let r = protocol::ureq_get(session, &url)?;
                    sign.guess_sign_result_by_text(&r.into_string().unwrap())
                } else {
                    SignResult::Fail {
                        msg: "滑块验证失败，请重试。".to_string(),
                    }
                }
            } else {
                SignResult::Fail { msg }
            }
        }
    };
    Ok(result)
}
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct RefreshQrCodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) enc: Option<String>,
    pub(crate) c: String,
    pub(crate) location: Option<Location>,
}
impl RefreshQrCodeSign {
    pub fn set_enc(&mut self, enc: String) {
        self.enc = Some(enc)
    }
    pub fn set_location(&mut self, location: Location) {
        self.location = Some(location)
    }
}
impl SignTrait for RefreshQrCodeSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.enc.is_some()
    }
    fn pre_sign(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let enc = self.enc.as_deref().unwrap_or("");
        let raw = self.as_inner();
        let active_id = raw.active_id.as_str();
        let uid = session.get_uid();
        let response_of_presign = protocol::pre_sign_for_qrcode_sign(
            session,
            raw.course.clone(),
            active_id,
            uid,
            &self.c,
            enc,
        )?;
        info!("用户[{}]预签到已请求。", session.get_stu_name());
        raw.analysis_after_presign(active_id, session, response_of_presign)
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let enc = unsafe { self.enc.as_ref().unwrap_unchecked() };
        sign_unchecked::<RefreshQrCodeSign>(self, enc, &self.location, session)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct NormalQrCodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) enc: Option<String>,
    pub(crate) c: String,
    pub(crate) location: Option<Location>,
}
impl NormalQrCodeSign {
    pub fn set_enc(&mut self, enc: String) {
        self.enc = Some(enc)
    }
    pub fn set_location(&mut self, location: Location) {
        self.location = Some(location)
    }
}
impl SignTrait for NormalQrCodeSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.enc.is_some()
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let enc = unsafe { self.enc.as_ref().unwrap_unchecked() };
        sign_unchecked::<NormalQrCodeSign>(self, enc, &self.location, session)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub enum QrCodeSign {
    RefreshQrCodeSign(RefreshQrCodeSign),
    NormalQrCodeSign(NormalQrCodeSign),
}
impl QrCodeSign {
    pub fn get_qrcode_arg_c(&self) -> &str {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => &a.c,
            QrCodeSign::NormalQrCodeSign(a) => &a.c,
        }
    }
    pub fn set_enc(&mut self, enc: String) {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.set_enc(enc),
            QrCodeSign::NormalQrCodeSign(a) => a.set_enc(enc),
        }
    }
    pub fn set_location(&mut self, location: Location) {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.set_location(location),
            QrCodeSign::NormalQrCodeSign(a) => a.set_location(location),
        }
    }
}
impl SignTrait for QrCodeSign {
    fn as_inner(&self) -> &RawSign {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.as_inner(),
            QrCodeSign::NormalQrCodeSign(a) => a.as_inner(),
        }
    }
    fn pre_sign(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        println!("sessions len: {}", session.get_stu_name());
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.pre_sign(session),
            QrCodeSign::NormalQrCodeSign(a) => a.pre_sign(session),
        }
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        unsafe {
            match self {
                QrCodeSign::RefreshQrCodeSign(a) => a.sign_unchecked(session),
                QrCodeSign::NormalQrCodeSign(a) => a.sign_unchecked(session),
            }
        }
    }
}
