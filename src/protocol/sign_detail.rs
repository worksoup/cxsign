use reqwest::{Client, Response};

// 签到信息获取
static SIGN_DETAIL: &str = "https://mobilelearn.chaoxing.com/newsign/signDetail";

pub async fn sign_detail(client: &Client, active_id: &str) -> Result<Response, reqwest::Error> {
    client
        .get(String::from(SIGN_DETAIL) + "?activePrimaryId=" + active_id + "&type=1")
        .send()
        .await
}