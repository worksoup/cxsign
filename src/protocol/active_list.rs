use crate::session::course::Struct课程;
use reqwest::{Client, Response};

// 查询活动
static ACTIVE_LIST: &str = "https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist";

pub async fn active_list(
    client: &Client, course: Struct课程
) -> Result<Response, reqwest::Error> {
    let url = {
        let mut url = String::from(ACTIVE_LIST);
        url.push_str("?fid=0&courseId=");
        url.push_str(course.get_课程号().to_string().as_str());
        url.push_str("&classId=");
        url.push_str(course.get_班级号().to_string().as_str());
        url.push_str("&showNotStartedActive=0&_=");
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        url.push_str(time.as_str());
        url
    };
    // println!("{url}");
    client.get(url).send().await
}
