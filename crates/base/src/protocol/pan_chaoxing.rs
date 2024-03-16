use ureq::{Agent, Response};

// 超星网盘页
static PAN_CHAOXING: &str = "https://pan-yz.chaoxing.com";

pub fn pan_chaoxing(client: &Agent) -> Result<Response, ureq::Error> {
    client.get(PAN_CHAOXING).call()
}
