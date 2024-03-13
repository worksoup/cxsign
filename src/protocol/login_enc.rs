use reqwest::header::HeaderMap;
use reqwest::{Client, Response};

// 非明文密码登录
static LOGIN_ENC: &str = "http://passport2.chaoxing.com/fanyalogin";

pub async fn login_enc(
    client: &Client,
    uname: &str,
    pwd_enc: &str,
) -> Result<Response, reqwest::Error> {
    let body = format!("uname={uname}&password={pwd_enc}&fid=-1&t=true&refer=https%253A%252F%252Fi.chaoxing.com&forbidotherlogin=0&validate=");
    let headers = {
        let mut header = HeaderMap::new();
        header.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        header.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        header
    };
    client
        .post(LOGIN_ENC)
        .headers(headers)
        .body(body)
        .send()
        .await
}
