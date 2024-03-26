use crate::sign::{RawSign, SignResult, SignTrait};
use ureq::Error;
use user::session::Session;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct SigncodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) signcode: Option<String>,
}
impl SigncodeSign {
    pub fn check(&self, session: &Session) -> bool {
        self.signcode.as_ref().map_or(false, |signcode| {
            RawSign::check_signcode(session, &self.raw_sign.active_id, signcode).unwrap_or(false)
        })
    }
    pub fn set_signcode(&mut self, signcode: String) {
        self.signcode = Some(signcode)
    }
}
impl SignTrait for SigncodeSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.signcode.is_some()
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        self.as_inner().sign_with_signcode(session, unsafe {
            self.signcode.as_ref().unwrap_unchecked()
        })
    }
}
