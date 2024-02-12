use reqwest::{Client, Response};

// 签到码检查
static CHECK_SINGCODE: &str =
    "https://mobilelearn.chaoxing.com/widget/sign/pcStuSignController/checkSignCode";

pub async fn check_signcode(
    client: &Client,
    active_id: &str,
    signcode: &str,
) -> Result<Response, reqwest::Error> {
    client
        .get(String::from(CHECK_SINGCODE) + "?activeId=" + active_id + "&signCode=" + signcode)
        .send()
        .await
}
