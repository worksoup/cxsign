use super::sign::SignActivity;
use crate::sign_session::{course::Course, session::SignSession};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Activity {
    Sign(SignActivity),
    Other(OtherActivity),
}

impl Activity {
    pub async fn from_course(
        session: &SignSession,
        c: Course,
    ) -> Result<Vec<Activity>, reqwest::Error> {
        let r = crate::utils::api::active_list(session, c.clone()).await?;
        let r: GetActivityR = r.json().await.unwrap();
        let mut arr = Vec::new();
        if let Some(data) = r.data {
            for ar in data.activeList {
                if let Some(other_id) = ar.otherId {
                    let other_id_i64: i64 = other_id.parse().unwrap();
                    if other_id_i64 >= 0 && other_id_i64 <= 5 {
                        let active_id = ar.id.to_string();
                        let detail =
                            SignActivity::get_sign_detial_by_active_id(active_id.as_str(), session)
                                .await?;
                        arr.push(Activity::Sign(SignActivity {
                            id: active_id,
                            name: ar.nameOne,
                            course: c.clone(),
                            other_id,
                            status: ar.status,
                            start_time_secs: (ar.startTime / 1000) as i64,
                            detail,
                        }))
                    } else {
                        arr.push(Activity::Other(OtherActivity {
                            id: ar.id.to_string(),
                            name: ar.nameOne,
                            course: c.clone(),
                            status: ar.status,
                            start_time_secs: (ar.startTime / 1000) as i64,
                        }))
                    }
                } else {
                    arr.push(Activity::Other(OtherActivity {
                        id: ar.id.to_string(),
                        name: ar.nameOne,
                        course: c.clone(),
                        status: ar.status,
                        start_time_secs: (ar.startTime / 1000) as i64,
                    }))
                }
            }
        }
        Ok(arr)
    }
}

#[derive(Debug)]
pub struct OtherActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
struct ActivityRaw {
    nameOne: String,
    id: i64,
    otherId: Option<String>,
    status: i32,
    startTime: u64,
}

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
struct Data {
    activeList: Vec<ActivityRaw>,
}

#[derive(Deserialize, Serialize)]
struct GetActivityR {
    data: Option<Data>,
}
