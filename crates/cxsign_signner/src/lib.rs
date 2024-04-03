#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(let_chains)]

mod default;
pub mod utils;

use cxsign_error::Error;
use cxsign_user::Session;
pub use default::*;
use std::collections::HashMap;

use cxsign_activity::sign::{SignResult, SignTrait};

pub trait SignnerTrait<T: SignTrait> {
    type ExtData<'e>;
    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut T,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error>;
    fn sign_single(
        sign: &mut T,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error>;
}
