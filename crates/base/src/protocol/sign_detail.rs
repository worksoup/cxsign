use ureq::{Agent, Response};

// 签到信息获取
static SIGN_DETAIL: &str = "https://mobilelearn.chaoxing.com/newsign/signDetail";

pub fn sign_detail(client: &Agent, active_id: &str) -> Result<Response, ureq::Error> {
    client
        .get(&format!("{SIGN_DETAIL}?activePrimaryId={active_id}&type=1"))
        .call()
}
