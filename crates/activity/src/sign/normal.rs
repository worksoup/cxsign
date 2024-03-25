use crate::sign::{RawSign, SignResult, SignState, SignTrait};
use ureq::Error;
use user::session::Session;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct NormalSign {
    pub(crate) raw_sign: RawSign,
}

impl SignTrait for NormalSign {
    fn get_raw(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_valid(&self) -> bool {
        self.raw_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.raw_sign.get_attend_info(session)
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        unsafe { self.raw_sign.sign_unchecked(session) }
    }
}
