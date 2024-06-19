use cxsign::Session;
use ureq::{Agent, Response};

static GET_VIEW_URL_HLS: &str = "http://newesxidian.chaoxing.com/live/getViewUrlHls";
pub fn get_view_url_hls(agent: &Agent, live_id: i64) -> Result<Response, Box<ureq::Error>> {
    let url = format!("{GET_VIEW_URL_HLS}?liveId={live_id}&status=1&jie=&isStudent=");
    Ok(agent.get(&url).call()?)
}
static LIST_STUDENT_COURSE_LIVE_PAGE: &str =
    "http://newesxidian.chaoxing.com/frontLive/listStudentCourseLivePage";
pub fn list_student_course_live_page(
    session: &Session,
    week: i64,
    term_year: i32,
    term: i32,
) -> Result<Response, Box<ureq::Error>> {
    let url = format!(
        "{LIST_STUDENT_COURSE_LIVE_PAGE}?fid=16820&userId={}&week={week}&termYear={term_year}&termId={term}&type=1",
        session.get_uid(),
    );
    Ok(session.get(&url).call()?)
}
static LIST_SINGLE_COURSE: &str = "http://newesxidian.chaoxing.com/live/listSignleCourse";
pub fn list_single_course(session: &Session, live_id: i64) -> Result<Response, Box<ureq::Error>> {
    let url = format!(
        "{LIST_SINGLE_COURSE}?fid=16820&liveId={live_id}&uId={}",
        session.get_uid()
    );
    Ok(session.get(&url).call()?)
}

static GET_VIEW_URL: &str = "http://newesxidian.chaoxing.com/live/getViewUrlNoCourseLive";
pub fn get_live_url(agent: &Agent, device_conde: &str) -> Result<Response, Box<ureq::Error>> {
    let url = format!("{GET_VIEW_URL}?deviceCode={device_conde}&status=1&fid=16820");
    Ok(agent.get(&url).call()?)
}
// pub fn get_recording_url(
//     agent: &Agent,
//     device_conde: &str,
//     start_time: &str,
//     end_time: &str,
// ) -> Result<Response, ureq::Error> {
//     let url = format!("{GET_VIEW_URL}?deviceCode={device_conde}&status=2&fid=16820&startTime={start_time}&endTime={end_time}");
//     agent.get(&url).call()
// }
static GET_WEEK_DETAIL: &str = "http://newesxidian.chaoxing.com/frontLive/getWeekDetail";
pub fn get_week_detail(
    agent: &Agent,
    week: i32,
    semester_id: i32,
) -> Result<Response, Box<ureq::Error>> {
    let url = format!("{GET_WEEK_DETAIL}?week={week}&semesterId={semester_id}");
    Ok(agent.get(&url).call()?)
}
