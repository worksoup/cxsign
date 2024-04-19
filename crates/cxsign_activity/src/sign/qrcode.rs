use crate::protocol;
use crate::sign::{CaptchaId, LocationSign, PreSignResult, RawSign, SignResult, SignTrait};
use cxsign_types::{Location, LocationWithRange};
use cxsign_user::Session;
use log::info;
use serde::{Deserialize, Serialize};

fn sign_unchecked<T: SignTrait>(
    sign: &T,
    enc: &str,
    location: &Location,
    preset_location: &Option<LocationWithRange>,
    captcha_id: Option<CaptchaId>,
    session: &Session,
) -> Result<SignResult, Box<ureq::Error>> {
    let url_getter = |l: &Location| {
        protocol::qrcode_sign_url(session, enc, sign.as_inner().active_id.as_str(), Some(l))
    };
    crate::utils::sign_unchecked_with_location(
        sign,
        url_getter,
        location,
        preset_location,
        captcha_id,
        session,
    )
}
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct QrCodeSign {
    pub(crate) is_refresh: bool,
    pub(crate) raw_sign: LocationSign,
    pub(crate) enc: Option<String>,
    pub(crate) c: String,
}
impl QrCodeSign {
    pub fn set_enc(&mut self, enc: String) {
        self.enc = Some(enc)
    }
    pub fn set_location(&mut self, location: Location) {
        self.raw_sign.set_location(location)
    }
    pub fn as_location_sign_mut(&mut self) -> &mut LocationSign {
        &mut self.raw_sign
    }
    pub fn is_refresh(&self) -> bool {
        self.is_refresh
    }
}
impl SignTrait for QrCodeSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign.as_inner()
    }
    fn is_ready_for_sign(&self) -> bool {
        self.enc.is_some()
    }
    fn pre_sign(&self, session: &Session) -> Result<PreSignResult, Box<ureq::Error>> {
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
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, Box<ureq::Error>> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(captcha_id) => {
                let enc = unsafe { self.enc.as_ref().unwrap_unchecked() };
                sign_unchecked::<QrCodeSign>(
                    self,
                    enc,
                    &self.raw_sign.location,
                    &self.raw_sign.preset_location,
                    captcha_id.into_first(),
                    session,
                )
            }
        }
    }
}
