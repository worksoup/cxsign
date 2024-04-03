use crate::protocol;
use crate::sign::{RawSign, SignResult, SignTrait};
use cxsign_types::Photo;
use cxsign_user::Session;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct PhotoSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) photo: Option<Photo>,
}
impl PhotoSign {
    pub fn set_photo(&mut self, photo: Photo) {
        self.photo = Some(photo)
    }
}
impl SignTrait for PhotoSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }

    fn as_inner_mut(&mut self) -> &mut RawSign {
        &mut self.raw_sign
    }

    fn is_ready_for_sign(&self) -> bool {
        self.photo.is_some()
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let photo = self.photo.as_ref().unwrap();
        let r = protocol::photo_sign(
            session,
            self.raw_sign.active_id.as_str(),
            photo.get_object_id(),
        )?;
        Ok(self.guess_sign_result_by_text(&r.into_string().unwrap()))
    }
}