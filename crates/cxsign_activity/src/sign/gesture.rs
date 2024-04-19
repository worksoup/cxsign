use crate::sign::{RawSign, SignResult, SignTrait};
use cxsign_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct GestureSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) gesture: Option<String>,
}
impl GestureSign {
    pub fn check(&self, session: &Session) -> bool {
        self.gesture.as_ref().map_or(false, |signcode| {
            RawSign::check_signcode(session, &self.raw_sign.active_id, signcode).unwrap_or(false)
        })
    }
    pub fn set_gesture(&mut self, gesture: String) {
        self.gesture = Some(gesture)
    }
}
impl SignTrait for GestureSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.gesture.is_some()
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        self.as_inner()
            .sign_with_signcode(session, unsafe { self.gesture.as_ref().unwrap_unchecked() })
    }
}
