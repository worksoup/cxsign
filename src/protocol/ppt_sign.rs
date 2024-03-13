use crate::session::Struct签到会话;
use crate::utils::address::Struct位置;
use reqwest::Response;

// 签到
static PPT_SIGN: &str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";

pub async fn general_sign(
    session: &Struct签到会话,
    active_id: &str,
) -> Result<Response, reqwest::Error> {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_用户真名();
    let url = PPT_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}");
    session.get(url).send().await
}

pub async fn photo_sign(
    session: &Struct签到会话,
    active_id: &str,
    object_id: &str,
) -> Result<Response, reqwest::Error> {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_用户真名();
    // NOTE 存疑。
    let name = percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC)
        .to_string();
    let url = PPT_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={name}");
    session.get(url).send().await
}

pub async fn qrcode_sign(
    session: &Struct签到会话,
    enc: &str,
    active_id: &str,
    位置: &Struct位置,
) -> Result<Response, reqwest::Error> {
    let address = 位置.get_地址();
    let lat = 位置.get_纬度();
    let lon = 位置.get_经度();
    let altitude = 位置.get_海拔();
    let url = PPT_SIGN;
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_用户真名();
    let location = format!(
        r#"{{"result":"1","address":"{address}","latitude":{lat},"longitude":{lon},"altitude":{altitude}}}"#
    );
    let location =
        percent_encoding::utf8_percent_encode(&location, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let url = format!(
        r#"{url}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location={location}&latitude=-1&longitude=-1&fid={fid}&appType=15"#
    );
    session.get(url).send().await
}

pub async fn location_sign(
    session: &Struct签到会话,
    位置: &Struct位置,
    active_id: &str,
    是否限定位置: bool,
) -> Result<Response, reqwest::Error> {
    let address = 位置.get_地址();
    let lat = 位置.get_纬度();
    let lon = 位置.get_经度();
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_用户真名();
    let if_tijiao = if 是否限定位置 { 1 } else { 0 };
    let url = format!("{PPT_SIGN}?name={stu_name}&address={address}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao={if_tijiao}&validate=");
    session.get(url).send().await
}

pub async fn signcode_sign(
    session: &Struct签到会话,
    active_id: &str,
    signcode: &str,
) -> Result<Response, reqwest::Error> {
    let url = PPT_SIGN;
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_用户真名();
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}");
    session.get(url).send().await
}
