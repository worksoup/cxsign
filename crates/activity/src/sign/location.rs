use crate::protocol;
use crate::sign::{RawSign, SignResult, SignState, SignTrait};
use types::Location;
use ureq::Error;
use user::session::Session;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct LocationSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) location: Option<Location>,
    pub(crate) is_auto_location: bool,
}
impl LocationSign {
    pub fn set_location(&mut self, location: Location) {
        self.location = Some(location)
    }
    pub fn set_is_auto_location(&mut self, is_auto_location: bool) {
        self.is_auto_location = is_auto_location
    }
}
impl SignTrait for LocationSign {
    fn get_raw(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.location.is_some()
    }
    fn is_valid(&self) -> bool {
        self.raw_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.raw_sign.get_attend_info(session)
    }

    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Error> {
        let r = protocol::location_sign(
            session,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            unsafe { self.location.as_ref().unwrap_unchecked() },
            self.raw_sign.active_id.as_str(),
            self.is_auto_location,
        )?;
        Ok(self.guess_sign_result(&r.into_string().unwrap()))
    }
}
