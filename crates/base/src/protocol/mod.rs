mod active_list;
mod analysis;
mod back_clazz_data;
mod check_signcode;
mod get_attend_info;
mod get_location_log;
mod other;
mod ppt_sign;
mod pre_sign;
mod sign_detail;

pub use active_list::*;
pub use analysis::*;
pub use back_clazz_data::*;
pub use check_signcode::*;
pub use get_attend_info::*;
pub use get_location_log::*;
pub use other::*;
pub use ppt_sign::*;
pub use pre_sign::*;
pub use sign_detail::*;

pub static QRCODE_PAT: &str = "https://mobilelearn.chaoxing.com/widget/sign/e";
