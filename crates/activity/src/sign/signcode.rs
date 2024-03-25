use crate::sign::{RawSign, SignResult, SignState, SignTrait};
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
}
impl SignTrait for SigncodeSign {
    fn get_raw(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.signcode.is_some()
    }
    fn is_valid(&self) -> bool {
        self.raw_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.raw_sign.get_attend_info(session)
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        let r = self.raw_sign.presign(session);
        if let Ok(a) = r.as_ref()
            && !a.is_susses()
        {
            self.raw_sign
                .sign_with_signcode(session, self.signcode.as_ref().unwrap())
        } else {
            r
        }
    }
}
