pub mod sign;
use crate::session::{course::Struct课程, Struct签到会话};
use serde_derive::{Deserialize, Serialize};
use sign::Struct签到;

#[derive(Debug)]
pub enum Enum活动 {
    签到(Struct签到),
    非签到活动(Struct非签到活动),
}

impl Enum活动 {
    pub async fn 获取课程的所有活动(
        签到会话: &Struct签到会话,
        c: Struct课程,
    ) -> Result<Vec<Enum活动>, reqwest::Error> {
        let r = crate::protocol::active_list(签到会话, c.clone()).await?;
        let r: GetActivityR = r.json().await.unwrap();
        let mut 活动列表 = Vec::new();
        if let Some(data) = r.data {
            for ar in data.activeList {
                if let Some(other_id) = ar.otherId {
                    let other_id_i64: i64 = other_id.parse().unwrap();
                    if (0..=5).contains(&other_id_i64) {
                        let 活动id = ar.id.to_string();
                        let detail = Struct签到::通过active_id获取签到信息(
                            活动id.as_str(),
                            签到会话,
                        )
                        .await?;
                        活动列表.push(Enum活动::签到(Struct签到 {
                            活动id,
                            签到名: ar.nameOne,
                            课程: c.clone(),
                            other_id,
                            状态码: ar.status,
                            开始时间戳: (ar.startTime / 1000) as i64,
                            签到信息: detail,
                        }))
                    } else {
                        活动列表.push(Enum活动::非签到活动(Struct非签到活动 {
                            id: ar.id.to_string(),
                            name: ar.nameOne,
                            course: c.clone(),
                            status: ar.status,
                            start_time_secs: (ar.startTime / 1000) as i64,
                        }))
                    }
                } else {
                    活动列表.push(Enum活动::非签到活动(Struct非签到活动 {
                        id: ar.id.to_string(),
                        name: ar.nameOne,
                        course: c.clone(),
                        status: ar.status,
                        start_time_secs: (ar.startTime / 1000) as i64,
                    }))
                }
            }
        }
        Ok(活动列表)
    }
}

#[derive(Debug)]
pub struct Struct非签到活动 {
    pub id: String,
    pub name: String,
    pub course: Struct课程,
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
