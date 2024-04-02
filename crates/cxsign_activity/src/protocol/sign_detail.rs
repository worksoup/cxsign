use log::debug;
use ureq::{Agent, Response};

// 签到信息获取
static SIGN_DETAIL: &str = "https://mobilelearn.chaoxing.com/newsign/signDetail";

pub fn sign_detail(client: &Agent, active_id: &str) -> Result<Response, Box<ureq::Error>> {
    let url = format!("{SIGN_DETAIL}?activePrimaryId={active_id}&type=1");
    debug!("{url}");
    Ok(client.get(&url).call()?)
}
