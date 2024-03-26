use crate::protocol;
use crate::sign::{RawSign, SignResult, SignTrait};
use cxsign_types::Location;
use ureq::Error;
use cxsign_user::Session;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct LocationSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) location: Option<Location>,
    pub(crate) need_location: bool,
}
impl LocationSign {
    pub fn set_location(&mut self, location: Location) {
        self.location = Some(location)
    }
    pub fn set_need_location(&mut self, is_auto_location: bool) {
        self.need_location = is_auto_location
    }
}
impl SignTrait for LocationSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.location.is_some()
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        let r = protocol::location_sign(
            session,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            unsafe { self.location.as_ref().unwrap_unchecked() },
            self.raw_sign.active_id.as_str(),
            self.need_location,
        )?;
        Ok(self.guess_sign_result_by_text(&r.into_string().unwrap()))
    }
}
