use ureq::Response;

static CAPTCHA_ID: &str = "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU";
// 获取滑块。
static GET_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/get/verification/image";
// 滑块验证。
static CHECK_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/check/verification/result";
// 获取服务器时间。
static GET_SERVER_TIME: &str = "https://captcha.chaoxing.com/captcha/get/conf";
