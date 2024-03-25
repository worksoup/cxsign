use crate::protocol;
use crate::sign::{RawSign, SignResult, SignState, SignTrait};
use types::Location;
use ureq::Error;
use user::session::Session;

unsafe fn qrcode_sign_unchecked(
    raw_sign: &RawSign,
    enc: &str,
    location: &Option<Location>,
    session: &Session,
) -> Result<SignResult, Error> {
    let r = raw_sign.presign_for_refresh_qrcode_sign(&raw_sign.sign_detail.c, enc, session);
    if let Ok(a) = r.as_ref()
        && !a.is_susses()
    {
        let r = protocol::qrcode_sign(
            session,
            enc,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            raw_sign.active_id.as_str(),
            location,
        )?;
        Ok(raw_sign.guess_sign_result(&r.into_string().unwrap()))
    } else {
        r
    }
}
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct RefreshQrCodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) enc: Option<String>,
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
    fn get_raw(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.enc.is_some()
    }
    fn is_valid(&self) -> bool {
        self.raw_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.raw_sign.get_attend_info(session)
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        let enc = unsafe { self.enc.as_ref().unwrap_unchecked() };
        unsafe { qrcode_sign_unchecked(&self.raw_sign, enc, &self.location, session) }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalQrCodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) enc: Option<String>,
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
    fn get_raw(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.enc.is_some()
    }
    fn is_valid(&self) -> bool {
        self.raw_sign.is_valid()
    }
    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.raw_sign.get_attend_info(session)
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        let enc = unsafe { self.enc.as_ref().unwrap_unchecked() };
        unsafe { qrcode_sign_unchecked(&self.raw_sign, enc, &self.location, session) }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QrCodeSign {
    RefreshQrCodeSign(RefreshQrCodeSign),
    NormalQrCodeSign(NormalQrCodeSign),
}
impl QrCodeSign {
    pub fn get_qrcode_arg_c(&self) -> &str {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => &a.raw_sign.sign_detail.c,
            QrCodeSign::NormalQrCodeSign(a) => &a.raw_sign.sign_detail.c,
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
    fn get_raw(&self) -> &RawSign {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.get_raw(),
            QrCodeSign::NormalQrCodeSign(a) => a.get_raw(),
        }
    }
    fn is_valid(&self) -> bool {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.is_valid(),
            QrCodeSign::NormalQrCodeSign(a) => a.is_valid(),
        }
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.get_attend_info(session),
            QrCodeSign::NormalQrCodeSign(a) => a.get_attend_info(session),
        }
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        unsafe {
            match self {
                QrCodeSign::RefreshQrCodeSign(a) => a.sign_unchecked(session),
                QrCodeSign::NormalQrCodeSign(a) => a.sign_unchecked(session),
            }
        }
    }

    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.sign(session),
            QrCodeSign::NormalQrCodeSign(a) => a.sign(session),
        }
    }
}
