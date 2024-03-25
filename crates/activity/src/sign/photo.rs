use crate::protocol;
use crate::sign::{RawSign, SignResult, SignState, SignTrait};
use types::Photo;
use ureq::Error;
use user::session::Session;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhotoSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) photo: Option<Photo>,
}

impl SignTrait for PhotoSign {
    fn is_ready_for_sign(&self) -> bool {
        self.photo.is_some()
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
            let photo = self.photo.as_ref().unwrap();
            let r = protocol::photo_sign(
                session,
                session.get_uid(),
                session.get_fid(),
                session.get_stu_name(),
                self.raw_sign.active_id.as_str(),
                photo.get_object_id(),
            )?;
            Ok(self.guess_sign_result(&r.into_string().unwrap()))
        } else {
            r
        }
    }
}
