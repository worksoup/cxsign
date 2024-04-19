use crate::protocol;
use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxsign_types::{Location, LocationWithRange};
use cxsign_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct LocationSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) preset_location: Option<LocationWithRange>,
    pub(crate) location: Location,
}
impl LocationSign {
    pub fn set_location(&mut self, location: Location) {
        self.location = location
    }
    pub fn get_preset_location(&self, addr: Option<&str>) -> Option<Location> {
        if let Some(location) = self.preset_location.as_ref() {
            let mut location = location.to_location();
            addr.map(|addr| {
                location.set_addr(addr);
            });
            Some(location)
        } else {
            None
        }
    }
}
impl SignTrait for LocationSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, Box<ureq::Error>> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(captcha_id) => {
                let url_getter = |l: &Location| {
                    protocol::location_sign_url(
                        session,
                        l,
                        self.raw_sign.active_id.as_str(),
                        self.preset_location.is_some(),
                    )
                };
                crate::utils::sign_unchecked_with_location(
                    self,
                    url_getter,
                    &self.location,
                    &self.preset_location,
                    captcha_id.into_first(),
                    session,
                )
            }
        }
    }
    fn pre_sign_and_sign(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let r = self.pre_sign(session)?;
        self.sign(session, r)
    }
}
