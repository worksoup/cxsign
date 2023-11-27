#[derive(Debug)]
pub struct Address {
    address: String,
    lat: String,
    lon: String,
    altitude: String,
}

impl Address {
    pub fn parse_str(pos: &str) -> Self {
        let pos: Vec<&str> = pos.split(',').map(|item| item.trim()).collect();
        assert_eq!(
            pos.len(),
            4,
            "位置信息格式错误！格式为：`addr,lon,lat,alt`."
        );
        Self::new(pos[0], pos[1], pos[2], pos[3])
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
