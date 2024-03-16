use ureq::{Agent, Response};

// 登录页
static LOGIN_PAGE: &str =
    "http://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com";

pub fn login_page(client: &Agent) -> Result<Response, ureq::Error> {
    client.get(LOGIN_PAGE).call()
}

// 非明文密码登录
static LOGIN_ENC: &str = "http://passport2.chaoxing.com/fanyalogin";

pub fn login_enc(client: &Agent, uname: &str, pwd_enc: &str) -> Result<Response, ureq::Error> {
    client
        .post(LOGIN_ENC)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("X-Requested-With", "XMLHttpRequest")
        .send_string(&format!("uname={uname}&password={pwd_enc}&fid=-1&t=true&refer=https%253A%252F%252Fi.chaoxing.com&forbidotherlogin=0&validate="))
}
