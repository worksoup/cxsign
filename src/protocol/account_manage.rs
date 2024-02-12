use reqwest::{Client, Response};

// 账号设置页
static ACCOUNT_MANAGE: &str = "http://passport2.chaoxing.com/mooc/accountManage";

pub async fn account_manage(client: &Client) -> Result<Response, reqwest::Error> {
    client.get(ACCOUNT_MANAGE).send().await
}