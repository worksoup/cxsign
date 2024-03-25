use crate::protocol;
use crate::sign::{RawSign, SignResult, SignState, SignTrait};
use types::Location;
use ureq::Error;
use user::session::Session;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct RefreshQrCodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) enc: Option<String>,
    pub(crate) location: Option<Location>,
}
impl SignTrait for RefreshQrCodeSign {
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
        let r = self.raw_sign.presign_for_refresh_qrcode_sign(
            &self.raw_sign.sign_detail.c,
            enc,
            session,
        );
        if let Ok(a) = r.as_ref()
            && !a.is_susses()
        {
            let r = protocol::qrcode_sign(
                session,
                enc,
                session.get_uid(),
                session.get_fid(),
                session.get_stu_name(),
                self.raw_sign.active_id.as_str(),
                &self.location,
            )?;
            Ok(self.guess_sign_result(&r.into_string().unwrap()))
        } else {
            r
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalQrCodeSign {
    pub(crate) raw_sign: RawSign,
}
impl SignTrait for NormalQrCodeSign {
    fn is_valid(&self) -> bool {
        self.raw_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.raw_sign.get_attend_info(session)
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        unsafe { self.raw_sign.sign_unchecked(session) }
    }

    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        self.raw_sign.sign(session)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
}
impl SignTrait for QrCodeSign {
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
