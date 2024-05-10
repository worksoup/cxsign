use std::collections::{HashMap, HashSet};

use crate::xddcc::tools::{json_parsing_error_handler, prog_init_error_handler};
use crate::xddcc::{room::Room, tools::VideoPath};
use cxsign::Session;
use indicatif::MultiProgress;
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Live {
    place: String,
    id: i64,
    #[serde(rename = "weekDay")]
    week_day: u32,
    jie: i32,
}
impl Live {
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }
    pub fn get_week_day(&self) -> u32 {
        self.week_day
    }
    fn get_jie(&self) -> i32 {
        self.jie
    }
    pub fn get_lives(
        session: &Session,
        week: i64,
        term_year: i32,
        term: i32,
    ) -> Result<HashMap<String, i64>, Box<ureq::Error>> {
        let vec =
            crate::xddcc::protocol::list_student_course_live_page(session, week, term_year, term)?
                .into_json::<Vec<Live>>()
                .unwrap_or_else(json_parsing_error_handler);
        let mut map = HashMap::new();
        for i in vec {
            map.insert(i.place, i.id);
        }
        Ok(map)
    }
    fn get_lives_by_time(
        session: &Session,
        term_year: i32,
        term: i32,
        week: i64,
        week_day: u32,
        jie: i32,
    ) -> Result<Option<Live>, Box<ureq::Error>> {
        let vec =
            crate::xddcc::protocol::list_student_course_live_page(session, week, term_year, term)?
                .into_json::<Vec<Live>>()
                .unwrap_or_else(json_parsing_error_handler);
        let iter = vec
            .into_iter()
            .filter(|live| (live.get_week_day() == week_day) && (live.get_jie() >= jie));
        let mut vec = iter.collect::<Vec<_>>();
        vec.sort_by_key(|live| live.get_jie());
        Ok(vec.first().cloned())
    }
    pub fn get_lives_now<'a, Iter: Iterator<Item = &'a Session> + Clone>(
        sessions: Iter,
        this: bool,
        multi: &MultiProgress,
    ) -> HashMap<&'a str, (&'a str, Room, VideoPath)> {
        let sessions = sessions.collect::<Vec<_>>();
        let total = sessions.len() as u64;
        let data_time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
        let mut term_year = 0;
        let mut term = 0;
        let mut week = 0;
        let mut first = true;
        let week_day = chrono::Datelike::weekday(&data_time).number_from_monday();
        let mut lives_map = HashMap::new();
        let sty = indicatif::ProgressStyle::with_template(
            "获取直播号：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap_or_else(prog_init_error_handler);
        let pb = multi.add(indicatif::ProgressBar::new(total));
        pb.set_style(sty);
        for session in sessions.clone() {
            if first {
                (term_year, term, week) = crate::xddcc::tools::term_year_detail(session);
                first = false;
            }
            let jie = crate::xddcc::tools::now_to_jie(this);
            let live = Live::get_lives_by_time(session, term_year, term, week, week_day, jie);
            if let Ok(Some(live)) = live {
                lives_map.insert(session, live);
            }
            pb.inc(1)
        }
        pb.finish_with_message("获取直播号完成。");
        multi.remove(&pb);
        let mut lives = HashSet::new();
        for live in lives_map.values() {
            lives.insert(live.get_id());
        }
        let mut rooms = HashMap::new();
        let sty = indicatif::ProgressStyle::with_template(
            "获取地址中：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap_or_else(prog_init_error_handler);
        let pb = multi.add(indicatif::ProgressBar::new(lives.len() as u64 * 2));
        pb.set_style(sty);
        pb.inc(0);
        if let Some(session) = sessions.clone().into_iter().next() {
            for live in lives {
                match Room::get_rooms(session, &live) {
                    Ok(room) => {
                        if let Some(room) = room {
                            pb.inc(1);
                            let video_path = room.get_live_video_path(session);
                            pb.inc(1);
                            rooms.insert(live, (room, video_path));
                        } else {
                            pb.inc(2);
                        }
                    }
                    Err(e) => {
                        warn!("教室获取错误：{e}.");
                        pb.inc(2);
                    }
                }
            }
        }
        pb.finish_with_message("已获取直播地址。");
        multi.remove(&pb);
        let mut results = HashMap::new();
        for (session, live) in lives_map {
            if let Some((room, video_path)) = rooms.get(&live.get_id()) {
                match video_path {
                    Ok(video_path) => {
                        results.insert(
                            session.get_uid(),
                            (session.get_stu_name(), room.clone(), video_path.clone()),
                        );
                    }
                    Err(e) => {
                        warn!("获取教室失败：{e}.")
                    }
                }
            }
        }
        results
    }
}
