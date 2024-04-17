use std::fmt::Display;

pub static CAPTCHA_ID: &str = "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU";
// 获取滑块。
static GET_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/get/verification/image";
// 滑块验证。
static CHECK_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/check/verification/result";
// 获取服务器时间。
static GET_SERVER_TIME: &str = "https://captcha.chaoxing.com/captcha/get/conf";

// Doesn't matter.
static CALLBACK_NAME: &str = "jQuery_114514_1919810";
pub fn get_server_time(
    agent: &ureq::Agent,
    captcha_id: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{GET_SERVER_TIME}?callback={CALLBACK_NAME}&captchaId={captcha_id}&_={time_stamp_mills}"
    );
    Ok(agent.get(&url).call()?)
}

pub fn get_captcha(
    agent: &ureq::Agent,
    captcha_id: &str,
    captcha_key: &str,
    tmp_token: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{GET_CAPTCHA}?{}&{}&{}&{}&{}&{}&{}&_={time_stamp_mills}",
        format_args!("callback={CALLBACK_NAME}",),
        format_args!("captchaId={}", captcha_id),
        format_args!("captchaKey={}", captcha_key),
        format_args!("token={}", tmp_token),
        "type=slide",
        "version=1.1.16",
        "referer=https%3A%2F%2Fmobilelearn.chaoxing.com",
    );
    Ok(agent.get(&url).call()?)
}
pub fn check_captcha(
    agent: &ureq::Agent,
    captcha_id: &str,
    x: i32,
    token: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{CHECK_CAPTCHA}?{}&{}&{}&{}&{}&{}&{}&{}&_={time_stamp_mills}",
        format_args!("callback={CALLBACK_NAME}",),
        format_args!("captchaId={}", captcha_id),
        format_args!("token={}", token),
        format_args!("textClickArr=%5B%7B%22x%22%3A{}%7D%5D", x),
        "type=slide",
        "coordinate=%5B%5D",
        "version=1.1.16",
        "runEnv=10",
    );
    let get = agent
        .get(&url)
        .set("Referer", "https://mobilelearn.chaoxing.com");
    Ok(get.call()?)
}
