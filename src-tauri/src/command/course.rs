use std::collections::hash_map::OccupiedError;

use cxsign_internal::Course;
use serde::Serialize;

use crate::{AccountPair, CoursesState, SessionsState};
#[derive(Serialize)]
pub struct CoursePair {
    course: Course,
    unames: Vec<AccountPair>,
}
#[derive(Serialize)]
pub struct CoursePairs {
    ok: Vec<CoursePair>,
    excluded: Vec<CoursePair>,
}
#[tauri::command]
pub async fn load_courses(
    sessions_state: tauri::State<'_, SessionsState>,
    courses_state: tauri::State<'_, CoursesState>,
) -> Result<(), String> {
    let sessions = sessions_state.0.lock().unwrap();
    let mut courses = courses_state.0.lock().unwrap();
    for (_, session) in sessions.iter() {
        let courses_ = Course::get_courses(session).map_err(|e| e.to_string())?;
        let account = AccountPair::from(session);
        for course in courses_ {
            if let Err(OccupiedError {
                mut entry,
                mut value,
            }) = courses.try_insert(course, vec![account.clone()])
            {
                entry.get_mut().push(value.pop().unwrap());
            }
        }
    }
    Ok(())
}
#[tauri::command]
pub async fn list_courses(
    courses_state: tauri::State<'_, CoursesState>,
) -> Result<Vec<CoursePair>, String> {
    let courses = &courses_state.0;
    let mut course_pairs = Vec::<CoursePair>::new();
    for (course, unames) in courses.lock().unwrap().iter() {
        course_pairs.push(CoursePair {
            course: course.clone(),
            unames: unames.clone(),
        })
    }
    Ok(course_pairs)
}
