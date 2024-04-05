use crate::protocol;
use crate::sign::{RawSign, SignResult, SignTrait};
use cxsign_types::Location;
use cxsign_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct LocationSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) location: Option<Location>,
    pub(crate) has_range: bool,
}
impl LocationSign {
    pub fn set_location(&mut self, location: Location) {
        self.location = Some(location)
    }
    pub fn set_has_range(&mut self, has_range: bool) {
        self.has_range = has_range
    }
}
impl SignTrait for LocationSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn as_inner_mut(&mut self) -> &mut RawSign {
        &mut self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.location.is_some()
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let r = protocol::location_sign(
            session,
            unsafe { self.location.as_ref().unwrap_unchecked() },
            self.raw_sign.active_id.as_str(),
            self.has_range,
        )?;
        Ok(self.guess_sign_result_by_text(&r.into_string().unwrap()))
    }
}
