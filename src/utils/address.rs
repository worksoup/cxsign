use std::f64::consts::PI;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct Address {
    address: String,
    lon: String,
    lat: String,
    altitude: String,
}
#[derive(Debug, Clone)]
pub struct AddressRange {
    pub pos: Address,
    pub range: u32,
}
impl Address {
    pub fn parse_str(pos: &str) -> Result<Self, &str> {
        let pos: Vec<&str> = pos.split(',').map(|item| item.trim()).collect();
        if pos.len() == 4 {
            Ok(Self::new(pos[0], pos[1], pos[2], pos[3]))
        } else {
            Err("位置信息格式错误！格式为：`addr,lon,lat,alt`.")
        }
    }
    pub fn new(address: &str, lon: &str, lat: &str, altitude: &str) -> Address {
        Address {
            address: address.into(),
            lon: lon.into(),
            lat: lat.into(),
            altitude: altitude.into(),
        }
    }
    /// 地址。
    pub fn get_addr(&self) -> &str {
        &self.address
    }
    /// 纬度。
    pub fn get_lat(&self) -> &str {
        &self.lat
    }
    /// 经度。
    pub fn get_lon(&self) -> &str {
        &self.lon
    }
    /// 海拔。
    pub fn get_alt(&self) -> &str {
        &self.altitude
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.address, self.lon, self.lat, self.altitude
        )
    }
}

pub fn add_pos(db: &super::sql::DataBase, course_id: i64, pos: &Address) {
    // 为指定课程添加位置。
    let mut posid = 0_i64;
    loop {
        if db.has_pos(posid) {
            posid += 1;
            continue;
        }
        db.add_pos_or(posid, course_id, pos, |_, _, _, _| {});
        break;
    }
}

pub fn find_pos_needed_in_html(html: &str) -> Option<AddressRange> {
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
    Some(AddressRange {
        pos: Address {
            address: results3[0].to_owned(),
            lon: results3[1].to_owned(),
            lat: results3[2].to_owned(),
            altitude: "1108".to_string(),
        },
        range: if let Ok(s) = results3[3].trim_end_matches('米').parse() {
            s
        } else {
            return None;
        },
    })
}

pub fn pos_rand_shift(pos: AddressRange) -> Address {
    const R: f64 = 6371393.0;
    let AddressRange {
        pos:
            Address {
                address,
                lon,
                lat,
                altitude,
            },
        range,
    } = pos;
    let range_f64 = range as f64;
    let lat: f64 = lat.parse().unwrap();
    let lon: f64 = lon.parse().unwrap();
    let lat = lat * PI / 180.0;
    let lon = lon * PI / 180.0;
    let mut r = rand::thread_rng().gen_range(0..range * 3) as f64 / range_f64 / 3.0;
    let theta = rand::thread_rng().gen_range(0..360) as f64 * PI / 180.0;
    r = range_f64 / R / (1.0 - theta.cos().powi(2) * lat.sin().powi(2)).sqrt() * r;
    let lat = (lat + r * theta.sin()) / PI * 180.0;
    let lon = (lon + r * theta.cos()) / PI * 180.0;
    let lat = format!("{lat:.6}");
    let lon = format!("{lon:.6}");
    Address {
        address,
        lon,
        lat,
        altitude,
    }
}
