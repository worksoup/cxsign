use std::collections::HashMap;
use std::f64::consts::PI;

use crate::protocol::get_location_log;
use crate::session::course::Struct课程;
use crate::session::Struct签到会话;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Struct位置 {
    地址: String,
    经度: String,
    纬度: String,
    海拔: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Data {
    #[serde(rename= "data")]
    位置及范围及签到id列表: Vec<Struct位置及范围及签到id>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Struct位置及范围及签到id {
    #[serde(rename= "activeid")]
    签到id: i64,
    #[serde(rename= "address")]
    地址: String,
    #[serde(rename= "longitude")]
    经度: f64,
    #[serde(rename= "latitude")]
    纬度: f64,
    #[serde(rename= "locationrange")]
    范围: String,
}

impl Struct位置及范围及签到id {
    pub fn to_位置及范围(&self) -> Struct位置及范围 {
        Struct位置及范围 {
            地址: self.地址.clone(),
            经度: self.经度.to_string(),
            纬度: self.纬度.to_string(),
            范围: self.范围.trim().parse().unwrap_or(100),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Struct位置及范围 {
    #[serde(rename= "address")]
    地址: String,
    #[serde(rename= "longitude")]
    经度: String,
    #[serde(rename= "latitude")]
    纬度: String,
    #[serde(rename= "locationrange")]
    范围: u32,
}

impl Struct位置及范围 {
    pub async fn 从log获取位置列表(
        签到会话: &Struct签到会话,
        课程: &Struct课程,
    ) -> Result<HashMap<String, Struct位置及范围>, reqwest::Error> {
        let r = get_location_log(签到会话, 课程).await?;
        let data: Data = r.json().await.unwrap();
        let mut map = HashMap::new();
        for l in data.位置及范围及签到id列表 {
            map.insert(l.签到id.to_string(), l.to_位置及范围());
        }
        Ok(map)
    }
    pub fn 获取随机偏移后的位置(&self) -> Struct位置 {
        const 地球半径: f64 = 6371393.0;
        let Struct位置及范围 {
            地址,
            经度,
            纬度,
            范围,
        } = self;
        let f64_范围 = *范围 as f64;
        let 纬度: f64 = 纬度.parse().unwrap();
        let 经度: f64 = 经度.parse().unwrap();
        let 纬度 = 纬度 * PI / 180.0;
        let 经度 = 经度 * PI / 180.0;
        let mut r = rand::thread_rng().gen_range(0..范围 * 3) as f64 / f64_范围 / 3.0;
        let theta = rand::thread_rng().gen_range(0..360) as f64 * PI / 180.0;
        r = f64_范围 / 地球半径 / (1.0 - theta.cos().powi(2) * 纬度.sin().powi(2)).sqrt() * r;
        let 纬度 = (纬度 + r * theta.sin()) / PI * 180.0;
        let 经度 = (经度 + r * theta.cos()) / PI * 180.0;
        let 纬度 = format!("{:.6}", 纬度);
        let 经度 = format!("{:.6}", 经度);
        Struct位置 {
            地址: 地址.clone(),
            经度,
            纬度,
            海拔: "1108".into(),
        }
    }
    pub fn 获取位置(&self) -> Struct位置 {
        Struct位置 {
            地址: self.地址.clone(),
            经度: self.经度.clone(),
            纬度: self.纬度.clone(),
            海拔: "1108".to_string(),
        }
    }
    pub fn 获取范围(&self) -> u32 {
        self.范围
    }
}

impl Struct位置 {
    pub fn 从字符串解析(位置字符串: &str) -> Result<Self, &str> {
        let 位置字符串: Vec<&str> = 位置字符串.split(',').map(|item| item.trim()).collect();
        if 位置字符串.len() == 4 {
            Ok(Self::new(
                位置字符串[0],
                位置字符串[1],
                位置字符串[2],
                位置字符串[3],
            ))
        } else {
            Err("位置信息格式错误！格式为：`地址,经度,纬度,海拔`.")
        }
    }
    pub fn new(地址: &str, 经度: &str, 纬度: &str, 海拔: &str) -> Struct位置 {
        Struct位置 {
            地址: 地址.into(),
            经度: 经度.into(),
            纬度: 纬度.into(),
            海拔: 海拔.into(),
        }
    }
    /// 地址。
    pub fn get_地址(&self) -> &str {
        &self.地址
    }
    /// 纬度。
    pub fn get_纬度(&self) -> &str {
        &self.纬度
    }
    /// 经度。
    pub fn get_经度(&self) -> &str {
        &self.经度
    }
    /// 海拔。
    pub fn get_海拔(&self) -> &str {
        &self.海拔
    }
}

impl std::fmt::Display for Struct位置 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{},{}", self.地址, self.经度, self.纬度, self.海拔)
    }
}

pub fn 为数据库添加位置(
    db: &super::sql::DataBase,
    course_id: i64,
    位置: &Struct位置,
) -> i64 {
    // 为指定课程添加位置。
    let mut 位置id = 0_i64;
    loop {
        if db.是否存在为某id的位置(位置id) {
            位置id += 1;
            continue;
        }
        db.添加位置_失败后则(位置id, course_id, 位置, |_, _, _, _| {});
        break;
    }
    位置id
}

pub fn 在html文本中寻找位置及范围(html: &str) -> Option<Struct位置及范围> {
    let p = vec![
        "id=\"locationText\"",
        "id=\"locationLongitude\"",
        "id=\"locationLatitude\"",
        "id=\"locationRange\"",
    ];
    let mut start = vec![None, None, None, None];
    let mut results1 = Vec::new();
    for i in 0..4 {
        let s = html.find(p[i]);
        start[i] = s;
        if let Some(s) = s {
            let r = &html[s + p[i].len()..html.len()];
            results1.push(r);
        } else {
            return None;
        }
    }
    let mut results2 = Vec::new();
    for r in &results1 {
        let s = r.find("value=\"");
        if let Some(s) = s {
            let r = &r[s + 7..r.len()];
            results2.push(r);
        } else {
            return None;
        }
    }
    let mut results3 = Vec::new();
    for r in &results2 {
        let e = r.find("\"");
        if let Some(e) = e {
            let r = &r[0..e];
            results3.push(r);
        } else {
            return None;
        }
    }
    Some(Struct位置及范围 {
        地址: results3[0].to_owned(),
        经度: results3[1].to_owned(),
        纬度: results3[2].to_owned(),
        范围: if let Ok(s) = results3[3].trim_end_matches('米').parse() {
            s
        } else {
            return None;
        },
    })
}
