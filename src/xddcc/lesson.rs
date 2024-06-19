use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cxsign::user::Session;
use indicatif::MultiProgress;
use serde::{Deserialize, Serialize};

use super::tools::{
    arc_into_inner_error_handler, json_parsing_error_handler, mutex_into_inner_error_handler,
    prog_init_error_handler, VideoPath,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Time_ {
    time: i64,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Lesson {
    #[serde(rename = "startTime")]
    start_time: Time_,
    id: i64,
}

impl Lesson {
    pub fn get_start_time(&self) -> i64 {
        self.start_time.time
    }
    pub fn get_live_id(&self) -> i64 {
        self.id
    }
    pub fn get_recording_url(
        session: &Session,
        live_id: i64,
    ) -> Result<VideoPath, Box<ureq::Error>> {
        crate::xddcc::tools::get_recording_live_video_path(session, live_id)
    }
    pub fn get_all_lessons(session: &Session, live_id: i64) -> Result<Vec<i64>, Box<ureq::Error>> {
        let mut lessons: Vec<Lesson> =
            crate::xddcc::protocol::list_single_course(&session, live_id)?
                .into_json()
                .unwrap_or_else(json_parsing_error_handler);
        lessons.sort_by_key(|l| l.get_start_time());
        Ok(lessons.into_iter().map(|l| l.get_live_id()).collect())
    }
    pub fn get_recording_lives(
        session: &Session,
        live_id: i64,
        multi: &MultiProgress,
    ) -> Result<HashMap<i64, VideoPath>, Box<ureq::Error>> {
        let lessons: Vec<Lesson> = crate::xddcc::protocol::list_single_course(&session, live_id)?
            .into_json()
            .unwrap_or_else(json_parsing_error_handler);
        let total = lessons.len();
        let thread_count = total / 64;
        let rest_count = total % 64;
        let sty = indicatif::ProgressStyle::with_template(
            "获取回放地址：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap_or_else(prog_init_error_handler);
        let pb = multi.add(indicatif::ProgressBar::new(total as u64));
        pb.set_style(sty);
        let pb = Arc::new(Mutex::new(pb));
        let pathes = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        for block in 0..64 {
            let ref_ = if block < rest_count {
                ((thread_count + 1) * block)..((thread_count + 1) * (block + 1))
            } else {
                (thread_count * block + rest_count)..(thread_count * (block + 1) + rest_count)
            };
            let session = (*session).clone();
            let pathes = Arc::clone(&pathes);
            let pb = Arc::clone(&pb);
            let mut lessons_ = vec![];
            for lesson in &lessons[ref_] {
                lessons_.push(lesson.clone())
            }
            let handle = std::thread::spawn(move || {
                for lesson in lessons_ {
                    if let Some(path) =
                        Lesson::get_recording_url(&session, lesson.get_live_id()).ok()
                    {
                        pathes.lock().unwrap().insert(lesson.get_start_time(), path);
                    }
                    pb.lock().unwrap().inc(1);
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let pathes = Arc::into_inner(pathes)
            .unwrap_or_else(arc_into_inner_error_handler)
            .into_inner()
            .unwrap_or_else(mutex_into_inner_error_handler);
        let pb = Arc::into_inner(pb)
            .unwrap_or_else(arc_into_inner_error_handler)
            .into_inner()
            .unwrap_or_else(mutex_into_inner_error_handler);
        pb.finish_with_message("获取回放地址完成。");
        multi.remove(&pb);
        Ok(pathes)
    }
}
