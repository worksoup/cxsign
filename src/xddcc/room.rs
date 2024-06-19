use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::{Datelike, Local};
use cxsign::Session;
use indicatif::MultiProgress;
use serde::{Deserialize, Serialize};

use crate::xddcc::tools::{
    arc_into_inner_error_handler, json_parsing_error_handler, mutex_into_inner_error_handler,
    prog_init_error_handler,
};
use crate::xddcc::{live::Live, tools::VideoPath};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Room {
    #[serde(rename = "schoolRoomName")]
    name: String,
    #[serde(rename = "deviceCode")]
    device_code: String,
    #[serde(rename = "schoolRoomId")]
    room_id: i32,
    id: i64,
}
impl Room {
    fn trim(mut self) -> Self {
        let name = self.name.trim().to_string();
        let _ = std::mem::replace(&mut self.name, name);
        self
    }
    pub fn get_live_video_path(&self, session: &Session) -> Result<VideoPath, Box<ureq::Error>> {
        crate::xddcc::tools::get_live_video_path(session, &self.device_code)
    }

    // pub fn get_live_video_path(&self, session: &Session) -> VideoPath {
    //     crate::tools::get_live_video_path(session, &self.device_code)
    // }
    // pub fn get_live_url(&self, session: &Session) -> WebUrl {
    //     crate::tools::get_live_web_url(session, &self.device_code)
    // }
    pub fn get_rooms(session: &Session, live_id: i64) -> Result<Option<Room>, Box<ureq::Error>> {
        let rooms: Vec<Room> = crate::xddcc::protocol::list_single_course(session, live_id)?
            .into_json()
            .unwrap_or_else(json_parsing_error_handler);
        Ok(rooms
            .into_iter()
            .find(|r| r.id == live_id)
            .map(|r| r.trim()))
    }
    pub fn get_all_rooms<'a, Iter: Iterator<Item = &'a Session> + Clone>(
        mut sessions: Iter,
        multi: &MultiProgress,
    ) -> HashMap<String, String> {
        let map = Arc::new(Mutex::new(HashMap::new()));
        Room::get_all_live_id(
            &sessions.clone().collect::<Vec<_>>(),
            Arc::clone(&map),
            multi,
        );
        let rooms = Arc::new(Mutex::new(HashMap::new()));
        if let Some(session) = sessions.next() {
            Room::id_to_rooms(map.clone(), (*session).clone(), rooms.clone(), multi);
        }
        Arc::into_inner(rooms)
            .unwrap_or_else(arc_into_inner_error_handler)
            .into_inner()
            .unwrap_or_else(mutex_into_inner_error_handler)
    }
    pub fn get_all_live_id(
        sessions: &[&Session],
        id_map: Arc<Mutex<HashMap<String, i64>>>,
        multi: &MultiProgress,
    ) {
        let now_year = Local::now().year();
        let thread_count = 64 / sessions.len() as i32;
        let week_total = 6 * 60;
        let total = week_total * sessions.len() as i32;
        let sty = indicatif::ProgressStyle::with_template(
            "获取直播号：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap_or_else(prog_init_error_handler);
        let pb = multi.add(indicatif::ProgressBar::new(total as u64));
        pb.set_style(sty);
        let pb = Arc::new(Mutex::new(pb));
        let mut handles = Vec::new();
        for session in sessions.iter() {
            let week_thread = week_total / (thread_count - 1) + 1;
            let thread_count = week_total / week_thread + 1;
            let week_rest = week_total % week_thread;
            for i in 0..thread_count {
                let session = (*session).clone();
                let id_map = Arc::clone(&id_map);
                let pb = Arc::clone(&pb);
                let handle = std::thread::spawn(move || {
                    for date_count in i * week_thread..if i != thread_count - 1 {
                        (i + 1) * week_thread
                    } else {
                        i * week_thread + week_rest
                    } {
                        let (year, term, week) =
                            crate::xddcc::tools::date_count_to_year_term_week(now_year, date_count);
                        let lives = Live::get_lives(&session, week, year, term).unwrap_or_default();
                        for live in lives {
                            id_map.lock().unwrap().insert(live.0, live.1);
                        }
                        pb.lock().unwrap().inc(1)
                    }
                });
                handles.push(handle);
            }
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let pb = Arc::into_inner(pb)
            .unwrap_or_else(arc_into_inner_error_handler)
            .into_inner()
            .unwrap_or_else(mutex_into_inner_error_handler);
        pb.finish_with_message("获取直播号完成。");
        multi.remove(&pb);
    }
    pub fn id_to_rooms(
        id_map: Arc<Mutex<HashMap<String, i64>>>,
        session: Session,
        rooms: Arc<Mutex<HashMap<String, String>>>,
        multi: &MultiProgress,
    ) {
        let ids = id_map.lock().unwrap().values().copied().collect::<Vec<_>>();
        let len = ids.len() as i32;
        let total = len;
        let sty = indicatif::ProgressStyle::with_template(
            "获取设备码：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap();
        let pb = multi.add(indicatif::ProgressBar::new(total as u64));
        pb.set_style(sty);
        let pb = Arc::new(Mutex::new(pb));
        let thread_count = 64;
        let chunk_rest = len % thread_count;
        let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
        for i in 0..chunk_count {
            let mut handles = Vec::new();
            let ids = &ids[(i * thread_count) as usize..if i != chunk_count - 1 {
                (i + 1) * thread_count
            } else {
                len
            } as usize];
            for id in ids {
                let id = *id;
                let session = session.clone();
                let rooms = rooms.clone();
                let pb = Arc::clone(&pb);
                let handle = std::thread::spawn(move || {
                    let room = Room::get_rooms(&session, id).unwrap();
                    if let Some(room) = room {
                        rooms.lock().unwrap().insert(room.name, room.device_code);
                    }
                    pb.lock().unwrap().inc(1);
                });
                handles.push(handle)
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }
        let pb = Arc::into_inner(pb).unwrap().into_inner().unwrap();
        pb.finish_with_message("获取设备码完成。");
        multi.remove(&pb);
    }
}
