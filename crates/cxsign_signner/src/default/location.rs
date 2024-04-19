use crate::SignnerTrait;
use cxsign_activity::sign::{LocationSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_store::DataBase;
use cxsign_types::Location;
use cxsign_user::Session;
use log::error;
use std::collections::HashMap;

pub struct DefaultLocationSignner<'a> {
    db: &'a DataBase,
    location_str: &'a Option<String>,
}

impl<'a> DefaultLocationSignner<'a> {
    pub fn new(db: &'a DataBase, location_str: &'a Option<String>) -> Self {
        Self { db, location_str }
    }
}
impl<'a> SignnerTrait<LocationSign> for DefaultLocationSignner<'a> {
    type ExtData<'e> = ();

    fn sign<'b, Sessions: Iterator<Item = &'b Session> + Clone>(
        &mut self,
        sign: &mut LocationSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'b Session, SignResult>, Error> {
        let location = crate::utils::get_locations(&sign, self.db, self.location_str);
        if location == Location::get_none_location() {
            error!("未获取到位置信息，请检查位置列表或检查输入。");
            return Err(Error::LocationError);
        }
        sign.set_location(location.clone());
        let mut map = HashMap::new();
        for session in sessions {
            let r = Self::sign_single(sign, session, ())?;
            map.insert(session, r);
        }
        Ok(map)
    }

    fn sign_single(sign: &mut LocationSign, session: &Session, _: ()) -> Result<SignResult, Error> {
        Ok(sign.pre_sign_and_sign(session)?)
    }
}
