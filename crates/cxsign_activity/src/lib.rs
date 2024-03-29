#![feature(let_chains)]
#![feature(map_try_insert)]

pub mod protocol;
pub mod sign;

use crate::sign::{RawSign, Sign, SignTrait};
use cxsign_types::Course;
use cxsign_user::Session;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::OccupiedError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Activity {
    Sign(Sign),
    Other(OtherActivity),
}

impl Activity {
    pub fn get_all_activities<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        sessions: Sessions,
    ) -> Result<
        (
            HashMap<Sign, Vec<Session>>,
            HashMap<Sign, Vec<Session>>,
            HashMap<OtherActivity, Vec<Session>>,
        ),
        ureq::Error,
    > {
        let mut courses = HashMap::new();
        for session in sessions.clone() {
            let courses_ = Course::get_courses(session)?;
            for course in courses_ {
                if let Err(OccupiedError {
                    mut entry,
                    value: _,
                }) = courses.try_insert(course, vec![session.clone()])
                {
                    entry.get_mut().push(session.clone());
                }
            }
        }
        let course_sessions_map = courses;
        let courses = course_sessions_map
            .keys()
            .map(|c| c.clone())
            .collect::<Vec<_>>();
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
                for session in &course_sessions_map[&course] {
                    let course = course.clone();
                    let session = (*session).clone();
                    let valid_signs = Arc::clone(&valid_signs);
                    let other_signs = Arc::clone(&other_signs);
                    let other_activities = Arc::clone(&other_activities);
                    let sessions = course_sessions_map[&course].clone();
                    let handle = std::thread::spawn(move || {
                        let activities =
                            Self::get_list_from_course(&session, &course).unwrap_or(vec![]);
                        let mut v = Vec::new();
                        let mut n = Vec::new();
                        let mut o = Vec::new();
                        for activity in activities {
                            if let Self::Sign(sign) = activity {
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
                    });
                    handles.push(handle);
                    break;
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
        Ok((valid_signs, other_signs, other_activities))
    }
    pub fn get_list_from_course(session: &Session, c: &Course) -> Result<Vec<Self>, ureq::Error> {
        let r = crate::protocol::active_list(session, c.clone())?;
        let r: GetActivityR = r.into_json().unwrap();
        let activities = Arc::new(Mutex::new(Vec::new()));
        if let Some(data) = r.data {
            let thread_count = 1;
            let len = data.active_list.len();
            let chunk_rest = len % thread_count;
            let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 0 };
            for i in 0..chunk_count {
                let ars = &data.active_list[i * thread_count..if i != chunk_count - 1 {
                    (i + 1) * thread_count
                } else {
                    len
                }];
                let mut handles = Vec::new();
                for ar in ars {
                    let ar = ar.clone();
                    let session = session.clone();
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
                            if let Ok(detail) =
                                RawSign::get_sign_detail(active_id.as_str(), &session)
                            {
                                let base_sign = RawSign {
                                    active_id,
                                    name: ar.name_one,
                                    course: c.clone(),
                                    other_id,
                                    status_code: ar.status,
                                    start_timestamp: (ar.start_time / 1000) as i64,
                                    sign_detail: detail,
                                };
                                activities
                                    .lock()
                                    .unwrap()
                                    .push(Self::Sign(base_sign.to_sign()))
                            }
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
