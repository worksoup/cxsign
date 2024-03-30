use crate::SignnerTrait;
use cxsign_activity::sign::{NormalSign, RawSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_user::Session;
use std::collections::HashMap;

pub struct DefaultNormalOrRawSignner;

impl SignnerTrait<NormalSign> for DefaultNormalOrRawSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut NormalSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        self.sign(sign.as_inner_mut(), sessions)
    }

    fn sign_single(sign: &mut NormalSign, session: &Session, _: ()) -> Result<SignResult, Error> {
        Self::sign_single(sign.as_inner_mut(), session, ())
    }
}

impl SignnerTrait<RawSign> for DefaultNormalOrRawSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut RawSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let mut map = HashMap::new();
        for session in sessions {
            let a = Self::sign_single(sign, session, ())?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(sign: &mut RawSign, session: &Session, _: ()) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session).map_err(|e| e.into())
    }
}
