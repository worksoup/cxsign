use crate::SignnerTrait;
use cxsign_activity::sign::{GestureSign, SignResult, SignTrait, SigncodeSign};
use cxsign_error::Error;
use cxsign_user::Session;
use std::collections::HashMap;

pub struct DefaultGestureOrSigncodeSignner(String);

impl DefaultGestureOrSigncodeSignner {
    pub fn new(signcode: &str) -> Self {
        Self(signcode.to_string())
    }
}

impl SignnerTrait<GestureSign> for DefaultGestureOrSigncodeSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut GestureSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let mut map = HashMap::new();
        sign.set_gesture(self.0.clone());
        for session in sessions {
            let a = Self::sign_single(sign, session, ())?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(sign: &mut GestureSign, session: &Session, _: ()) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session).map_err(|e| e.into())
    }
}

impl SignnerTrait<SigncodeSign> for DefaultGestureOrSigncodeSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut SigncodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        sign.set_signcode(self.0.clone());
        let mut map = HashMap::new();
        for session in sessions {
            let a = Self::sign_single(sign, session, ())?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(sign: &mut SigncodeSign, session: &Session, _: ()) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session).map_err(|e| e.into())
    }
}
