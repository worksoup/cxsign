use std::collections::HashMap;
use std::f64::consts::PI;
use std::str::FromStr;

use crate::Course;
use cxsign_user::Session;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
pub struct Location {
    addr: String,
    lon: String,
    lat: String,
    alt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocationWithRange {
    #[serde(rename = "address")]
    addr: String,
    #[serde(rename = "longitude")]
    lon: String,
    #[serde(rename = "latitude")]
    lat: String,
    #[serde(rename = "locationrange")]
    range: u32,
}

impl LocationWithRange {
    pub fn from_log(
        session: &Session,
        course: &Course,
    ) -> Result<HashMap<String, Self>, Box<ureq::Error>> {
        #[derive(Debug, Clone, Deserialize, Serialize)]
        struct LocationWithRangeAndActiveId {
            #[serde(rename = "activeid")]
            active_id: i64,
            #[serde(rename = "address")]
            addr: String,
            #[serde(rename = "longitude")]
            lon: f64,
            #[serde(rename = "latitude")]
            lat: f64,
            #[serde(rename = "locationrange")]
            range: String,
        }
        impl LocationWithRangeAndActiveId {
            pub fn to_location_with_range(&self) -> LocationWithRange {
                LocationWithRange {
                    addr: self.addr.clone(),
                    lon: self.lon.to_string(),
                    lat: self.lat.to_string(),
                    range: self.range.trim().parse().unwrap_or(100),
                }
            }
        }
        #[derive(Debug, Clone, Deserialize, Serialize)]
        struct Data {
            #[serde(rename = "data")]
            data: Vec<LocationWithRangeAndActiveId>,
        }
        let r = crate::protocol::get_location_log(session, course)?;
        let data: Data = r.into_json().unwrap();
        let mut map = HashMap::new();
        for l in data.data {
            map.insert(l.active_id.to_string(), l.to_location_with_range());
        }
        Ok(map)
    }
    pub fn find_in_html(html: &str) -> Option<LocationWithRange> {
        let p = [
            "id=\"locationText\"",
            "id=\"locationLongitude\"",
            "id=\"locationLatitude\"",
            "id=\"locationRange\"",
        ];
        let mut start = [None, None, None, None];
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
            let e = r.find('"');
            if let Some(e) = e {
                let r = &r[0..e];
                results3.push(r);
            } else {
                return None;
            }
        }
        Some(LocationWithRange {
            addr: results3[0].to_owned(),
            lon: results3[1].to_owned(),
            lat: results3[2].to_owned(),
            range: if let Ok(s) = results3[3].trim_end_matches('米').parse() {
                s
            } else {
                return None;
            },
        })
    }
    pub fn to_shifted_location(&self) -> Location {
        const R: f64 = 6371393.0;
        let LocationWithRange {
            addr,
            lon,
            lat,
            range,
        } = self;
        let lat: f64 = lat.parse().unwrap();
        let lon: f64 = lon.parse().unwrap();
        let mut r = rand::thread_rng().gen_range(0..range * 3) as f64 / (*range as f64) / 60.0;
        let theta = rand::thread_rng().gen_range(0..360) as f64 * PI / 180.0;
        r *= (*range as f64)
            / R
            / (1.0 - theta.cos().powi(2) * (lat * PI / 180.0).sin().powi(2)).sqrt();
        let lat = format!("{:.6}", ((lat * PI / 180.0) + r * theta.sin()) / PI * 180.0);
        let lon = format!("{:.6}", (lon * PI / 180.0 + r * theta.cos()) / PI * 180.0);
        Location {
            addr: addr.clone(),
            lon,
            lat,
            alt: "1108".into(),
        }
    }
    pub fn to_location(&self) -> Location {
        Location {
            addr: self.addr.clone(),
            lon: self.lon.clone(),
            lat: self.lat.clone(),
            alt: "1108".to_string(),
        }
    }
    pub fn get_range(&self) -> u32 {
        self.range
    }
}
impl FromStr for Location {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Location::parse(s)
    }
}
impl Location {
    pub fn get_none_location() -> Self {
        Location {
            addr: "".to_string(),
            lon: "".to_string(),
            lat: "".to_string(),
            alt: "".to_string(),
        }
    }
    pub fn parse(location_str: &str) -> Result<Self, String> {
        let location_str: Vec<&str> = location_str.split(',').map(|item| item.trim()).collect();
        if location_str.len() == 4 {
            Ok(Self::new(
                location_str[0],
                location_str[1],
                location_str[2],
                location_str[3],
            ))
        } else {
            Err("位置信息格式错误！格式为：`地址,经度,纬度,海拔`.".to_string())
        }
    }
    pub fn new(addr: &str, lon: &str, lat: &str, alt: &str) -> Location {
        Location {
            addr: addr.into(),
            lon: lon.into(),
            lat: lat.into(),
            alt: alt.into(),
        }
    }
    /// 地址。
    pub fn get_addr(&self) -> &str {
        &self.addr
    }
    /// 经度。
    pub fn get_lon(&self) -> &str {
        &self.lon
    }
    /// 纬度。
    pub fn get_lat(&self) -> &str {
        &self.lat
    }
    /// 海拔。
    pub fn get_alt(&self) -> &str {
        &self.alt
    }
    /// 地址。
    pub fn set_addr(&mut self, addr: &str) {
        addr.clone_into(&mut self.addr)
    }
    /// 经度。
    pub fn set_lon(&mut self, lon: &str) {
        lon.clone_into(&mut self.lon)
    }
    /// 纬度。
    pub fn set_lat(&mut self, lat: &str) {
        lat.clone_into(&mut self.lat)
    }
    /// 海拔。
    pub fn set_alt(&mut self, alt: &str) {
        alt.clone_into(&mut self.alt)
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{},{}", self.addr, self.lon, self.lat, self.alt)
    }
}

// pub fn 为数据库添加位置(
//     db: &super::sql::DataBase, course_id: i64, 位置: &Location
// ) -> i64 {
//     // 为指定课程添加位置。
//     let mut 位置id = 0_i64;
//     loop {
//         if db.是否存在为某id的位置(位置id) {
//             位置id += 1;
//             continue;
//         }
//         db.添加位置_失败后则(位置id, course_id, 位置, |_, _, _, _| {});
//         break;
//     }
//     位置id
// }
#[cfg(test)]
mod tests {
    use crate::LocationWithRange;

    #[test]
    fn a() {
        let l = LocationWithRange {
            addr: "addr".into(),
            lon: "108.840053".into(),
            lat: "34.129522".into(),
            range: 100,
        };
        println!("{}", l.to_shifted_location())
    }
}
