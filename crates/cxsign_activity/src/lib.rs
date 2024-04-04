#![feature(let_chains)]
#![feature(map_try_insert)]
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

pub mod protocol;
pub mod sign;

use crate::sign::{RawSign, SignTrait};
use cxsign_store::ExcludeTable;
use cxsign_types::Course;
use cxsign_user::Session;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::OccupiedError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ActivitiesSessionsMap = (
    HashMap<RawSign, Vec<Session>>,
    HashMap<RawSign, Vec<Session>>,
    HashMap<OtherActivity, Vec<Session>>,
);
pub type Activities = (Vec<RawSign>, Vec<RawSign>, Vec<OtherActivity>);
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Activity {
    RawSign(RawSign),
    Other(OtherActivity),
}

impl Activity {
    pub fn get_course_activities(
        table: ExcludeTable,
        session: &Session,
        course: &Course,
    ) -> Result<Activities, Box<ureq::Error>> {
        let mut v = Vec::new();
        let mut n = Vec::new();
        let mut o = Vec::new();
        let activities = Self::get_list_from_course(session, course).unwrap_or_default();
        let mut dont_exclude = false;
        for activity in activities {
            if let Self::RawSign(sign) = activity {
                let start_time = chrono::DateTime::from_timestamp(sign.start_timestamp, 0).unwrap();
                let now = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
                if now.signed_duration_since(start_time).num_days() < 160 {
                    dont_exclude = true;
                }
                if sign.is_valid() {
                    v.push(sign);
                } else {
                    n.push(sign);
                }
            } else if let Self::Other(other_activity) = activity {
                o.push(other_activity);
            }
        }
        let id = course.get_id();
        let excluded = table.has_exclude(id);
        if dont_exclude && excluded {
            table.delete_exclude(id);
        } else if !dont_exclude && !excluded {
            table.add_exclude(id);
        }
        Ok((v, n, o))
    }
    fn get_activities(
        table: ExcludeTable,
        set_excludes: bool,
        courses: HashMap<Course, Vec<Session>>,
    ) -> Result<ActivitiesSessionsMap, Box<ureq::Error>> {
        let course_sessions_map = courses;
        let courses = course_sessions_map.keys().cloned().collect::<Vec<_>>();
        let excludes = Arc::new(Mutex::new(Vec::new()));
        let valid_signs = Arc::new(Mutex::new(HashMap::new()));
        let other_signs = Arc::new(Mutex::new(HashMap::new()));
        let other_activities = Arc::new(Mutex::new(HashMap::new()));
        let thread_count = 256;
        let len = courses.len();
        let chunk_rest = len % thread_count;
        let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
        for i in 0..chunk_count {
            let courses = &courses[i * thread_count..if i != chunk_count - 1 {
                (i + 1) * thread_count
            } else {
                len
            }];
            let mut handles = Vec::new();
            for course in courses {
                info!("加载课程{course}的签到。");
                if let Some(session) = &course_sessions_map[course].first() {
                    let course = course.clone();
                    let session = (*session).clone();
                    let valid_signs = Arc::clone(&valid_signs);
                    let other_signs = Arc::clone(&other_signs);
                    let other_activities = Arc::clone(&other_activities);
                    let excludes = excludes.clone();
                    let sessions = course_sessions_map[&course].clone();
                    let handle = std::thread::spawn(move || {
                        let activities =
                            Self::get_list_from_course(&session, &course).unwrap_or(vec![]);
                        let mut v = Vec::new();
                        let mut n = Vec::new();
                        let mut o = Vec::new();
                        // NOTE: 此处也会将没有过签到的课程排除掉。
                        // TODO: 需要修改。
                        let mut dont_exclude = false;
                        for activity in activities {
                            if let Self::RawSign(sign) = activity {
                                if set_excludes {
                                    let start_time =
                                        chrono::DateTime::from_timestamp(sign.start_timestamp, 0)
                                            .unwrap();
                                    let now = chrono::DateTime::<chrono::Local>::from(
                                        std::time::SystemTime::now(),
                                    );
                                    if now.signed_duration_since(start_time).num_days() < 160 {
                                        dont_exclude = true;
                                    }
                                }
                                if sign.is_valid() {
                                    v.push(sign);
                                } else {
                                    n.push(sign);
                                }
                            } else if let Self::Other(other_activity) = activity {
                                o.push(other_activity);
                            }
                        }
                        for v in v {
                            valid_signs.lock().unwrap().insert(v, sessions.clone());
                        }
                        for n in n {
                            other_signs.lock().unwrap().insert(n, sessions.clone());
                        }
                        for o in o {
                            other_activities.lock().unwrap().insert(o, sessions.clone());
                        }
                        debug!("course: list_activities, ok.");
                        if set_excludes && !dont_exclude {
                            excludes.lock().unwrap().push(course.get_id())
                        }
                    });
                    handles.push(handle);
                }
            }
            for h in handles {
                debug!("handle: {h:?} join.");
                h.join().unwrap();
            }
        }
        let valid_signs = Arc::into_inner(valid_signs).unwrap().into_inner().unwrap();
        let other_signs = Arc::into_inner(other_signs).unwrap().into_inner().unwrap();
        let other_activities = Arc::into_inner(other_activities)
            .unwrap()
            .into_inner()
            .unwrap();
        if set_excludes {
            table.update_excludes(&Arc::into_inner(excludes).unwrap().into_inner().unwrap());
        }
        Ok((valid_signs, other_signs, other_activities))
    }
    pub fn get_all_activities<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        table: ExcludeTable,
        sessions: Sessions,
        set_excludes: bool,
    ) -> Result<ActivitiesSessionsMap, Box<ureq::Error>> {
        let excludes = table.get_excludes();
        let set_excludes = set_excludes || excludes.is_empty();
        let mut courses = HashMap::new();
        for session in sessions.clone() {
            let courses_ = Course::get_courses(session)?;
            for course in courses_ {
                if (set_excludes || !excludes.contains(&course.get_id()))
                    && let Err(OccupiedError {
                        mut entry,
                        value: _,
                    }) = courses.try_insert(course, vec![session.clone()])
                {
                    entry.get_mut().push(session.clone());
                }
            }
        }
        Self::get_activities(table, set_excludes, courses)
    }
    pub fn get_list_from_course(
        session: &Session,
        c: &Course,
    ) -> Result<Vec<Self>, Box<ureq::Error>> {
        let r = crate::protocol::active_list(session, c.clone())?;
        let r: GetActivityR = r.into_json().unwrap();
        let activities = Arc::new(Mutex::new(Vec::new()));
        if let Some(data) = r.data {
            let thread_count = 1;
            let len = data.active_list.len();
            let chunk_rest = len % thread_count;
            let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
            for i in 0..chunk_count {
                let ars = &data.active_list[i * thread_count..if i != chunk_count - 1 {
                    (i + 1) * thread_count
                } else {
                    len
                }];
                let mut handles = Vec::new();
                for ar in ars {
                    let ar = ar.clone();
                    let c = c.clone();
                    let activities = activities.clone();
                    let handle = std::thread::spawn(move || {
                        if let Some(other_id) = ar.other_id
                            && {
                                let other_id_i64: i64 = other_id.parse().unwrap();
                                (0..=5).contains(&other_id_i64)
                            }
                        {
                            let active_id = ar.id.to_string();
                            let base_sign = RawSign {
                                active_id,
                                name: ar.name_one,
                                course: c.clone(),
                                other_id,
                                status_code: ar.status,
                                start_timestamp: (ar.start_time / 1000) as i64,
                            };
                            activities.lock().unwrap().push(Self::RawSign(base_sign))
                        } else {
                            activities.lock().unwrap().push(Self::Other(OtherActivity {
                                id: ar.id.to_string(),
                                name: ar.name_one,
                                course: c.clone(),
                                status: ar.status,
                                start_time_secs: (ar.start_time / 1000) as i64,
                            }))
                        }
                    });
                    handles.push(handle);
                }
                for h in handles {
                    h.join().unwrap();
                }
            }
        }
        let activities = Arc::into_inner(activities).unwrap().into_inner().unwrap();
        Ok(activities)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct OtherActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Deserialize, Serialize, Clone)]
struct ActivityRaw {
    #[serde(rename = "nameOne")]
    name_one: String,
    id: i64,
    #[serde(rename = "otherId")]
    other_id: Option<String>,
    status: i32,
    #[serde(rename = "startTime")]
    start_time: u64,
}

#[derive(Deserialize, Serialize)]
struct Data {
    #[serde(rename = "activeList")]
    active_list: Vec<ActivityRaw>,
}

#[derive(Deserialize, Serialize)]
struct GetActivityR {
    data: Option<Data>,
}
