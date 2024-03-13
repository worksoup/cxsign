use reqwest::{Client, Response};

// 获取签到之后的信息，例如签到时的 ip, UA, 时间等
// 参见 "http://mobilelearn.chaoxing.com/page/sign/signIn?courseId={course_id}&classId={class_id}&activeId={active_id}&fid={??}"
static GET_ATTEND_INFO: &str = "http://mobilelearn.chaoxing.com/v2/apis/sign/getAttendInfo";

pub async fn get_attend_info(client: &Client, active_id: &str) -> Result<Response, reqwest::Error> {
    client
        .get(String::from(GET_ATTEND_INFO) + "?activeId=" + active_id + "&type=1")
        .send()
        .await
}