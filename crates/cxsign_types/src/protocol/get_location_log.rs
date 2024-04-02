use crate::course::Course;
use ureq::{Agent, Response};

// 获取位置信息列表
static GET_LOCATION_LOG: &str = "https://mobilelearn.chaoxing.com/v2/apis/sign/getLocationLog";

pub fn get_location_log(session: &Agent, course: &Course) -> Result<Response, Box<ureq::Error>> {
    Ok(session
        .get(&format!(
            "{GET_LOCATION_LOG}?DB_STRATEGY=COURSEID&STRATEGY_PARA=courseId&courseId={}&classId={}",
            course.get_id(),
            course.get_class_id()
        ))
        .call()?)
}
