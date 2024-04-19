use cxsign_types::Location;
use cxsign_user::Session;
use ureq::Response;

// 签到
static PPT_SIGN: &str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";

pub fn ureq_get(agent: &ureq::Agent, url: &str) -> Result<Response, Box<ureq::Error>> {
    Ok(agent.get(&url).call()?)
}

pub fn general_sign(session: &Session, active_id: &str) -> Result<Response, Box<ureq::Error>> {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    session.get(&format!("{PPT_SIGN}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}")).call().map_err(|e| e.into())
}

pub fn photo_sign(
    session: &Session,
    active_id: &str,
    object_id: &str,
) -> Result<Response, Box<ureq::Error>> {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    // NOTE 存疑。
    session.get(&format!("{PPT_SIGN}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={}", percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC))).call().map_err(|e| e.into())
}

pub fn qrcode_sign_url(
    session: &Session,
    enc: &str,
    active_id: &str,
    location: Option<&Location>,
) -> String {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    // TODO: 存疑。
    if let Some(location) = location {
        let (addr, lat, lon, alt) = (
            location.get_addr(),
            location.get_lat(),
            location.get_lon(),
            location.get_alt(),
        );
        let location_str = format!(
            r#"{{"result":"1","address":"{addr}","latitude":{lat},"longitude":{lon},"altitude":{alt}}}"#
        );
        let location_str = percent_encoding::utf8_percent_encode(
            &location_str,
            percent_encoding::NON_ALPHANUMERIC,
        )
        .to_string();
        format!(
            r#"{PPT_SIGN}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location={location_str}&latitude=-1&longitude=-1&fid={fid}&appType=15"#
        )
    } else {
        format!(
            r#"{PPT_SIGN}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location=&latitude=-1&longitude=-1&fid={fid}&appType=15"#
        )
    }
}

pub fn qrcode_sign(
    session: &Session,
    enc: &str,
    active_id: &str,
    location: Option<&Location>,
) -> Result<Response, Box<ureq::Error>> {
    let url = qrcode_sign_url(session, enc, active_id, location);
    Ok(session.get(&url).call()?)
}
pub fn location_sign_url(
    session: &Session,
    location: &Location,
    active_id: &str,
    is_auto_location: bool,
) -> String {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    let address = location.get_addr();
    let lat = location.get_lat();
    let lon = location.get_lon();
    let if_tijiao = if is_auto_location { 1 } else { 0 };
    format!("{PPT_SIGN}?name={stu_name}&address={address}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao={if_tijiao}")
}
pub fn location_sign(
    session: &Session,
    location: &Location,
    active_id: &str,
    is_auto_location: bool,
) -> Result<Response, Box<ureq::Error>> {
    Ok(session
        .get(&location_sign_url(
            session,
            location,
            active_id,
            is_auto_location,
        ))
        .call()?)
}

pub fn signcode_sign(
    session: &Session,
    active_id: &str,
    signcode: &str,
) -> Result<Response, Box<ureq::Error>> {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    let url = format!("{PPT_SIGN}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}");
    Ok(session.get(&url).call()?)
}
