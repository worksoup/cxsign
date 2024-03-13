use reqwest::{Client, Response};

// 登录页
static LOGIN_PAGE: &str =
    "http://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com";

pub async fn login_page(client: &Client) -> Result<Response, reqwest::Error> {
    client.get(LOGIN_PAGE).send().await
}
