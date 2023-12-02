#[derive(Debug)]
pub struct Address {
    address: String,
    lat: String,
    lon: String,
    altitude: String,
}

impl Address {
    pub fn parse_str(pos: &str) -> Result<Self, &str> {
        let pos: Vec<&str> = pos.split(',').map(|item| item.trim()).collect();
        if pos.len() == 4 {
            Ok(Self::new(
                pos[0].trim(),
                pos[1].trim(),
                pos[2].trim(),
                pos[3].trim(),
            ))
        } else {
            Err("位置信息格式错误！格式为：`addr,lon,lat,alt`.")
        }
    }
    pub fn new(address: &str, lat: &str, lon: &str, altitude: &str) -> Address {
        Address {
            address: address.into(),
            lat: lat.into(),
            lon: lon.into(),
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
