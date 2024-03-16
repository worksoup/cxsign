use ureq::{Agent, Response};

// 签到码检查
static CHECK_SIGNCODE: &str =
    "https://mobilelearn.chaoxing.com/widget/sign/pcStuSignController/checkSignCode";

pub fn check_signcode(
    client: &Agent,
    active_id: &str,
    signcode: &str,
) -> Result<Response, ureq::Error> {
    client
        .get(&format!(
            "{CHECK_SIGNCODE}?activeId={active_id}&signCode={signcode}"
        ))
        .call()
}
