use ureq::{Agent, Response};

// 获取超星云盘的 token
static PAN_TOKEN: &str = "https://pan-yz.chaoxing.com/api/token/uservalid";

pub fn pan_token(client: &Agent) -> Result<Response, ureq::Error> {
    client.get(PAN_TOKEN).call()
}