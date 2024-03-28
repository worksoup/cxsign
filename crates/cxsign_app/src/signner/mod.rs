mod default;

use cxsign_error::Error;
use cxsign_user::Session;
pub use default::*;
use std::collections::HashMap;

use crate::sign::{SignResult, SignTrait};

pub trait SignnerTrait<T: SignTrait> {
    type ExtData;
    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &self,
        sign: &mut T,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error>;
    fn sign_single(
        &self,
        sign: &mut T,
        session: &Session,
        extra_data: Self::ExtData,
    ) -> Result<SignResult, Error>;
}
