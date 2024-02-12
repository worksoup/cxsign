use reqwest::{Client, Response};

// 获取课程
static BACK_CLAZZ_DATA: &str = "http://mooc1-api.chaoxing.com/mycourse/backclazzdata";

pub async fn back_clazz_data(client: &Client) -> Result<Response, reqwest::Error> {
    let url = String::from(BACK_CLAZZ_DATA) + "?view=json&rss=1";
    client.get(url).send().await
}