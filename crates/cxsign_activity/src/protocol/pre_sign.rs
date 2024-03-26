use cxsign_types::Course;
use ureq::{Agent, Response};

// 预签到
static PRE_SIGN: &str = "https://mobilelearn.chaoxing.com/newsign/preSign";

pub fn pre_sign(
    client: &Agent,
    course: Course,
    active_id: &str,
    uid: &str,
) -> Result<Response, ureq::Error> {
    let course_id = course.get_id();
    let class_id = course.get_class_id();
    let url = PRE_SIGN;
    let url =
        format!("{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0");

    client.get(&url).call()
}
pub fn pre_sign_for_qrcode_sign(
    client: &Agent,
    course: Course,
    active_id: &str,
    uid: &str,
    c: &str,
    enc: &str,
) -> Result<Response, ureq::Error> {
    let course_id = course.get_id();
    let class_id = course.get_class_id();
    let url =
        format!("{PRE_SIGN}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0&rcode={}", format!(
            "&rcode={}",
            percent_encoding::utf8_percent_encode(
                &format!("SIGNIN:aid={active_id}&source=15&Code={c}&enc={enc}"),
                percent_encoding::NON_ALPHANUMERIC,
            )
        ));

    client.get(&url).call()
}
