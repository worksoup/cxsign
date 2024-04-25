use std::collections::HashMap;
use cxsign::{Location, LocationPreprocessorTrait};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
struct LocationWithoutAlt {
    latitude: f64,
    longitude: f64,
    address: String,
}

impl LocationWithoutAlt {
    fn to_location(&self, alt: &str) -> Location {
        Location::new(&self.address, &format!("{:.6}", self.longitude), &format!("{:.6}", self.latitude), alt)
    }
}

/// See [here](https://github.com/Pairman/Xdcheckin/blob/main/src/xdcheckin/core/locations.py).
static LOCATIONS_JSON: &str = r#"
{
	"A楼": {
		"latitude": 34.133171,
		"longitude": 108.837420,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"B楼": {
		"latitude": 34.132297,
		"longitude": 108.838367,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"C楼": {
		"latitude": 34.131125,
		"longitude": 108.838983,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"D楼": {
		"latitude": 34.130856,
		"longitude": 108.841579,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"EI楼": {
		"latitude": 34.130878,
		"longitude": 108.839863,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"EII楼": {
		"latitude": 34.130856,
		"longitude": 108.841579,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"EIII楼": {
		"latitude": 34.130056,
		"longitude": 108.843268,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"F楼": {
		"latitude": 34.130654,
		"longitude": 108.843016,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"G楼": {
		"latitude": 34.129660,
		"longitude": 108.845244,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	},
	"信远楼": {
		"latitude": 34.131640,
		"longitude": 108.845415,
		"address": "西安市长安区兴隆街道外环北路西安电子科技大学(南校区)"
	},
	"图书馆": {
		"latitude": 34.131125,
		"longitude": 108.838983,
		"address": "西安市长安区兴隆街道内环南路西安电子科技大学(南校区)"
	},
	"大学生活动中心": {
		"latitude": 34.134972,
		"longitude": 108.835282,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"北操场": {
		"latitude": 34.137362,
		"longitude": 108.837906,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"北篮球场": {
		"latitude": 34.134972,
		"longitude": 108.835282,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"北乒乓球场": {
		"latitude": 34.134972,
		"longitude": 108.835282,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"游泳中心": {
		"latitude": 34.134972,
		"longitude": 108.835282,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"南操场": {
		"latitude": 34.132559,
		"longitude": 108.832542,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"南篮球场": {
		"latitude": 34.128472,
		"longitude": 108.832443,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"南乒乓球场": {
		"latitude": 34.129376,
		"longitude": 108.834375,
		"address": "西安市长安区兴隆街道梧桐大道西安电子科技大学(南校区)"
	},
	"远望谷体育馆": {
		"latitude": 34.126418,
		"longitude": 108.844544,
		"address": "西安市长安区兴隆街道内环南路西安电子科技大学(南校区)"
	},
	"博物馆": {
		"latitude": 34.125589,
		"longitude": 108.844337,
		"address": "西安市长安区兴隆街道内环南路西安电子科技大学(南校区)"
	},
	"网安大楼": {
		"latitude": 34.128353,
		"longitude": 108.842010,
		"address": "西安市长安区兴隆街道内环南路西安电子科技大学(南校区)"
	},
	"礼仪广场": {
		"latitude": 34.132006,
		"longitude": 108.842118,
		"address": "西安市长安区兴隆街道内环北路西安电子科技大学(南校区)"
	}
}
"#;
lazy_static! {
    static ref LOCATIONS_WITHOUT_ALT: HashMap< String, LocationWithoutAlt >
		= serde_json::from_str(LOCATIONS_JSON).unwrap();
    pub static ref ADDRS: HashMap< String, String >
		= LOCATIONS_WITHOUT_ALT.iter().map(|(k,v)|(k.clone(),v.address.clone())).collect();
    pub static ref LOCATIONS: HashMap< String, Location >
		= LOCATIONS_WITHOUT_ALT.iter().map(|(k,v)|(k.clone(),v.to_location("1108"))).collect();
}
pub struct LocationPreprocessor;

impl LocationPreprocessorTrait for LocationPreprocessor {
    fn do_preprocess(&self, location: Location) -> Location {
        let [mut addr, lon, lat, alt] = location.to_owned_fields();
        for (k, v) in ADDRS.iter() {
            if addr.starts_with(k.trim_matches('楼')) {
                addr = v.to_string();
            }
        }
        #[cfg(debug_assertions)]
        if addr.contains("菜鸟驿站") {
            addr = "TEST".to_string();
        }
        Location::from_owned_fields([addr, lon, lat, alt])
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{ADDRS, LOCATIONS};

    #[test]
    fn print_addrs() {
        println!("{:?}", ADDRS.keys().collect::<Vec<_>>());
    }

    #[test]
    fn print_locations() {
        println!("{:?}", LOCATIONS.values().collect::<Vec<_>>());
    }
}