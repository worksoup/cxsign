use log::debug;
use cxsign_types::Course;
use ureq::{Agent, Response};

// 查询活动
static ACTIVE_LIST: &str = "https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist";

pub fn active_list(client: &Agent, course: Course) -> Result<Response, ureq::Error> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let url = format!(
        "{ACTIVE_LIST}?fid=0&courseId={}&classId={}&showNotStartedActive=0&_={time}",
        course.get_id(),
        course.get_class_id(),
    );
    debug!("{url}");
    client.get(&url).call()
}
