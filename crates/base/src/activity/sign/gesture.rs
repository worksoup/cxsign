use crate::activity::sign::base::BaseSign;
use crate::activity::sign::{SignResult, SignState, SignTrait};
use crate::user::session::Session;
use ureq::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GestureSign {
    pub(crate) base_sign: BaseSign,
    pub(crate) gesture: Option<String>,
}
impl SignTrait for GestureSign {
    fn is_ready_for_sign(&self) -> bool {
        self.gesture.is_some()
    }
    fn is_valid(&self) -> bool {
        self.base_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.base_sign.get_attend_info(session)
    }

    unsafe fn sign_internal(&self, session: &Session) -> Result<SignResult, Error> {
        let r = self.base_sign.presign(session);
        if let Ok(a) = r.as_ref()
            && !a.is_susses()
        {
            self.base_sign
                .sign_with_signcode(session, self.gesture.as_ref().unwrap())
        } else {
            r
        }
    }

    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        self.base_sign.sign(session)
    }
}
