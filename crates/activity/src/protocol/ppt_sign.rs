use types::Location;
use ureq::{Agent, Response};

// 签到
static PPT_SIGN: &str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";

pub fn general_sign(
    session: &Agent,
    uid: &str,
    fid: &str,
    stu_name: &str,
    active_id: &str,
) -> Result<Response, ureq::Error> {
    session.get(&format!("{PPT_SIGN}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}")).call()
}

pub fn photo_sign(
    session: &Agent,
    uid: &str,
    fid: &str,
    stu_name: &str,
    active_id: &str,
    object_id: &str,
) -> Result<Response, ureq::Error> {
    // NOTE 存疑。
    session.get(&format!("{PPT_SIGN}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={}", percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC))).call()
}

pub fn qrcode_sign(
    session: &Agent,
    uid: &str,
    fid: &str,
    stu_name: &str,
    enc: &str,
    active_id: &str,
    location: &Option<Location>,
) -> Result<Response, ureq::Error> {
    // TODO: 存疑。
    let (address, lat, lon, altitude) = if let Some(location) = location {
        (
            location.get_addr(),
            location.get_lat(),
            location.get_lon(),
            location.get_alt(),
        )
    } else {
        ("", "", "", "")
    };
    let location_str = format!(
        r#"{{"result":"1","address":"{address}","latitude":{lat},"longitude":{lon},"altitude":{altitude}}}"#
    );
    let location_str =
        percent_encoding::utf8_percent_encode(&location_str, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let url = format!(
        r#"{PPT_SIGN}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location={location_str}&latitude=-1&longitude=-1&fid={fid}&appType=15"#
    );
    session.get(&url).call()
}

pub fn location_sign(
    session: &Agent,
    uid: &str,
    fid: &str,
    stu_name: &str,
    location: &Location,
    active_id: &str,
    is_auto_location: bool,
) -> Result<Response, ureq::Error> {
    let address = location.get_addr();
    let lat = location.get_lat();
    let lon = location.get_lon();
    let if_tijiao = if is_auto_location { 1 } else { 0 };
    let url = format!("{PPT_SIGN}?name={stu_name}&address={address}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao={if_tijiao}&validate=");
    session.get(&url).call()
}

pub fn signcode_sign(
    session: &Agent,
    uid: &str,
    fid: &str,
    stu_name: &str,
    active_id: &str,
    signcode: &str,
) -> Result<Response, ureq::Error> {
    session.get(&format!("{PPT_SIGN}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}")).call()
}
