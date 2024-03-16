use ureq::{Agent, Response};

// 账号设置页
static ACCOUNT_MANAGE: &str = "http://passport2.chaoxing.com/mooc/accountManage";

pub fn account_manage(client: &Agent) -> Result<Response, ureq::Error> {
    client.get(ACCOUNT_MANAGE).call()
}
