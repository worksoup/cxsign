use reqwest::{Client, Response};

// 获取超星云盘的 token
static PAN_TOKEN: &str = "https://pan-yz.chaoxing.com/api/token/uservalid";

pub async fn pan_token(client: &Client) -> Result<Response, reqwest::Error> {
    client.get(PAN_TOKEN).send().await
}