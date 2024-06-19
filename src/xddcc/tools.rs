use chrono::{Local, Timelike};
use cxsign::Session;
use log::error;
use serde::{Deserialize, Serialize};
use std::error::Error as ErrorTrait;
use std::{collections::HashMap, hash::Hash};

pub(crate) fn json_parsing_error_handler<T>(e: impl ErrorTrait) -> T {
    error!("json 解析出错！错误信息：{e}.");
    panic!()
}
pub(crate) fn resp_parsing_error_handler<T>(e: impl ErrorTrait) -> T {
    error!("响应数据无法转为字符串，错误信息：{e}.");
    panic!()
}
pub(crate) fn prog_init_error_handler<T>(e: impl ErrorTrait) -> T {
    error!("json 解析出错！错误信息：{e}.");
    panic!()
}
pub(crate) fn arc_into_inner_error_handler<T>() -> T {
    error!("Arc 指针为空！");
    panic!()
}
pub(crate) fn mutex_into_inner_error_handler<T>(e: impl ErrorTrait) -> T {
    error!("保有互斥锁的其他线程发生 panic, 错误信息：{e}.");
    panic!()
}
#[derive(Serialize, Default, Debug, Clone)]
pub struct VideoPath {
    ppt_video: Option<String>,
    teacher_full: Option<String>,
    teacher_track: Option<String>,
    student_full: Option<String>,
}
#[derive(Serialize, Default, Debug, Clone)]
struct WebUrl {
    url: String,
}
fn web_url_to_video_path(url: &WebUrl) -> VideoPath {
    let url = &url.url.split("?info=").collect::<Vec<_>>()[1];
    let url = percent_encoding::percent_decode_str(url)
        .decode_utf8()
        .unwrap_or_default()
        .to_string();
    #[derive(Deserialize)]
    struct VideoPathInternal {
        #[serde(rename = "pptVideo")]
        ppt_video: Option<String>,
        #[serde(rename = "teacherFull")]
        teacher_full: Option<String>,
        #[serde(rename = "teacherTrack")]
        teacher_track: Option<String>,
        #[serde(rename = "studentFull")]
        student_full: Option<String>,
    }
    #[derive(Deserialize)]
    struct Info {
        #[serde(rename = "videoPath")]
        video_path: VideoPathInternal,
    }
    let Info {
        video_path:
            VideoPathInternal {
                ppt_video,
                teacher_full,
                teacher_track,
                student_full,
            },
    } = serde_json::from_str(&url).unwrap_or_else(json_parsing_error_handler);
    VideoPath {
        ppt_video,
        teacher_full,
        teacher_track,
        student_full,
    }
}
fn get_live_web_url(session: &Session, device_code: &str) -> Result<WebUrl, Box<ureq::Error>> {
    let url = crate::xddcc::protocol::get_live_url(session, device_code)?
        .into_string()
        .unwrap_or_else(resp_parsing_error_handler);
    Ok(WebUrl { url })
}
fn get_recording_live_web_url(session: &Session, live_id: i64) -> Result<WebUrl, Box<ureq::Error>> {
    let url = crate::xddcc::protocol::get_view_url_hls(session, live_id)?
        .into_string()
        .unwrap_or_else(resp_parsing_error_handler);
    Ok(WebUrl { url })
}
pub fn get_live_video_path(
    session: &Session,
    device_code: &str,
) -> Result<VideoPath, Box<ureq::Error>> {
    let url = get_live_web_url(session, device_code);
    Ok(web_url_to_video_path(&url?))
}
pub fn get_recording_live_video_path(
    session: &Session,
    live_id: i64,
) -> Result<VideoPath, Box<ureq::Error>> {
    let url = get_recording_live_web_url(session, live_id);
    Ok(web_url_to_video_path(&url?))
}
pub fn year_to_semester_id(year: i32, term: i32) -> i32 {
    let mut r = 2 * year - 4035 + term;
    if year == 2018 {
        r -= 1;
    } else if r < 1 {
        r = 1;
    }
    r
}
pub fn date_count_to_year_term_week(now_year: i32, date_count: i32) -> (i32, i32, i64) {
    (
        now_year - 6 + (date_count / 30) % 2 + date_count / 60,
        2 - (date_count / 30) % 2,
        date_count as i64 % 30 + 1,
    )
}
// pub fn out<S: Serialize>(contents: &S, path: Option<PathBuf>) {
//     let contents = toml::to_string_pretty(contents).unwrap();
//     if let Some(path) = path {
//         std::fs::write(path, contents).expect("写入内容出错！");
//     } else {
//         debug!("{contents}")
//     }
// }
pub fn now_to_jie(this: bool) -> i32 {
    fn now_to_jie_internal() -> i32 {
        let date_time = Local::now();
        let s1 = Local::now().with_hour(8).unwrap().with_minute(30).unwrap();
        let s3 = Local::now().with_hour(10).unwrap().with_minute(25).unwrap();
        let s5 = Local::now().with_hour(14).unwrap().with_minute(0).unwrap();
        let s7 = Local::now().with_hour(15).unwrap().with_minute(55).unwrap();
        let s9 = Local::now().with_hour(19).unwrap().with_minute(0).unwrap();
        if date_time < s1 {
            -1
        } else if date_time >= s1 && date_time < s3 {
            1
        } else if date_time >= s3 && date_time < s5 {
            3
        } else if date_time >= s5 && date_time < s7 {
            5
        } else if date_time >= s7 && date_time < s9 {
            7
        } else {
            9
        }
    }
    if !this {
        now_to_jie_internal() + 2
    } else {
        now_to_jie_internal()
    }
}
pub fn map_sort_by_key<K: Ord + Hash, V>(map: HashMap<K, V>) -> Vec<(K, V)> {
    let mut map = map.into_iter().collect::<Vec<_>>();
    map.sort_by(|x, y| x.0.cmp(&y.0));
    map.into_iter().collect()
}
pub fn term_year_detail(session: &Session) -> (i32, i32, i64) {
    let data_time = chrono::DateTime::<Local>::from(std::time::SystemTime::now());
    let year = chrono::Datelike::year(&data_time);
    let semester_id = year_to_semester_id(year - 1, 2);

    #[derive(Deserialize)]
    struct WeekDetail {
        date1: String,
    }
    let WeekDetail { date1, .. } = crate::xddcc::protocol::get_week_detail(session, 1, semester_id)
        .unwrap()
        .into_json()
        .unwrap();
    let date = date1.split('-').map(|s| s.trim()).collect::<Vec<_>>();
    let month = date[0].parse::<u32>().unwrap();
    let day = date[1].parse::<u32>().unwrap();
    let term_begin_data_time = <chrono::DateTime<Local> as std::str::FromStr>::from_str(&format!(
        "{year}-{month}-{day}T00:00:00.0+08:00"
    ))
    .unwrap();
    let week = data_time
        .signed_duration_since(term_begin_data_time)
        .num_weeks()
        + 1;
    let (term_year, term) = if chrono::Datelike::month(&data_time) * 100
        + chrono::Datelike::day(&data_time)
        > month * 100 + day
        && chrono::Datelike::month(&data_time) * 100 + chrono::Datelike::day(&data_time) < 700
    {
        (year - 1, 2)
    } else {
        (year, 1)
    };
    (term_year, term, week)
}

pub struct PairVec<K, V> {
    vec: Vec<(K, V)>,
}
impl<K, V> PairVec<K, V> {
    pub fn new(vec: Vec<(K, V)>) -> Self {
        Self { vec }
    }
}
impl<K: Serialize, V: Serialize> Serialize for PairVec<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.vec.len()))?;
        for (k, v) in &self.vec {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}
pub fn out<S: Serialize>(contents: &S, path: Option<std::path::PathBuf>) {
    let contents = serde_json::to_string_pretty(contents).unwrap();
    if let Some(path) = path {
        std::fs::write(path, contents).expect("写入内容出错！");
    } else {
        println!("{contents}")
    }
}
