use crate::session::course::Struct课程;
use reqwest::{Client, Response};

// 预签到
static PRE_SIGN: &str = "https://mobilelearn.chaoxing.com/newsign/preSign";

pub async fn pre_sign(
    client: &Client,
    course: Struct课程,
    active_id: &str,
    uid: &str,
    is_qrcode_sign: bool,
    c: &str,
    enc: &str,
) -> Result<Response, reqwest::Error> {
    let course_id = course.get_课程号();
    let class_id = course.get_班级号();
    let url = PRE_SIGN;
    let url = if is_qrcode_sign {
        let ex_args = format!("SIGNIN:aid={active_id}&source=15&Code={c}&enc={enc}");
        let ex_args = "&rcode=".to_owned()
            + percent_encoding::utf8_percent_encode(
                ex_args.as_str(),
                percent_encoding::NON_ALPHANUMERIC,
            )
            .to_string()
            .as_str();
        format!("{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0&rcode={ex_args}")
    } else {
        format!("{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0")
    };

    client.get(url).send().await
}
