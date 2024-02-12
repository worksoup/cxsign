use crate::session::course::Struct课程;
use crate::session::Struct签到会话;
use reqwest::Response;

// 获取位置信息列表
static GET_LOCATION_LOG: &str = "https://mobilelearn.chaoxing.com/v2/apis/sign/getLocationLog";

pub async fn get_location_log(
    session: &Struct签到会话,
    课程: Struct课程,
) -> Result<Response, reqwest::Error> {
    let url = format!(
        "{GET_LOCATION_LOG}?DB_STRATEGY=COURSEID&STRATEGY_PARA=courseId&courseId={}&classId={}",
        课程.get_课程号(),
        课程.get_班级号()
    );
    session.get(url).send().await
}
