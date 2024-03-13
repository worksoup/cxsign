use reqwest::{Client, Response};

// 超星网盘页
static PAN_CHAOXING: &str = "https://pan-yz.chaoxing.com";

pub async fn pan_chaoxing(client: &Client) -> Result<Response, reqwest::Error> {
    let url = PAN_CHAOXING;
    client.get(url).send().await
}
