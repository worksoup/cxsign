#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(let_chains)]

mod app;
mod signner;
pub mod utils;

pub use app::*;
use cxsign_activity::sign;
pub use cxsign_activity::{Activity, OtherActivity};
pub use cxsign_error::*;
pub use cxsign_types::{Course, Location, LocationWithRange, Photo};
pub use cxsign_user::{Session, UserCookies};
pub use sign::*;
pub use signner::*;

pub mod protocol {
    pub use cxsign_activity::protocol::*;
    pub use cxsign_login::protocol::*;
    pub use cxsign_pan::protocol::*;
    pub use cxsign_types::protocol::*;
    pub use cxsign_user::protocol::*;
}

pub mod store {
    pub use cxsign_store::{DataBase, DataBaseTableTrait};
    pub mod tables {
        pub use cxsign_store::{AccountTable, AliasTable, ExcludeTable};
        pub use cxsign_types::{CourseTable, LocationTable};
    }
}
