use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
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
    fn as_inner_mut(&mut self) -> &mut RawSign {
        &mut self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.gesture.is_some()
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, Box<ureq::Error>> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            _ => self
                .as_inner()
                .sign_with_signcode(session, unsafe { self.gesture.as_ref().unwrap_unchecked() }),
        }
    }
}
