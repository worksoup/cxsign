use log::warn;
use ureq::{Agent, Response};
static PPT_SIGN: &str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";
// // web 聊天页
// static WEB_IM: &str = "https://im.chaoxing.com/webim/me";

// 无课程群聊的预签到
static CHAT_GROUP_PRE_SIGN: &str = "https://mobilelearn.chaoxing.com/sign/preStuSign";
pub fn chat_group_pre_sign(
    client: &Agent,
    active_id: &str,
    uid: &str,
    chat_id: &str,
    tuid: &str,
) -> Result<Response, Box<ureq::Error>> {
    let url = CHAT_GROUP_PRE_SIGN;
    let url = format!("{url}?activeId={active_id}&code=&uid={uid}&courseId=null&classId=0&general=0&chatId={chat_id}&appType=0&tid={tuid}&atype=null&sys=0");
    Ok(client.get(&url).call()?)
}
// 无课程群聊的签到
static CHAT_GROUP_SIGN: &str = "https://mobilelearn.chaoxing.com/sign/stuSignajax";
pub fn chat_group_general_sign(
    client: &Agent,
    active_id: &str,
    uid: &str,
) -> Result<Response, Box<ureq::Error>> {
    let url = CHAT_GROUP_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=");
    Ok(client.get(&url).call()?)
}

pub fn chat_group_photo_sign(
    client: &Agent,
    active_id: &str,
    uid: &str,
    object_id: &str,
) -> Result<Response, Box<ureq::Error>> {
    let url = CHAT_GROUP_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&fid=0&objectId={object_id}");
    Ok(client.get(&url).call()?)
}
pub fn chat_group_location_sign(
    client: &Agent,
    address: &str,
    active_id: &str,
    uid: &str,
    lat: &str,
    lon: &str,
) -> Result<Response, Box<ureq::Error>> {
    let address =
        percent_encoding::utf8_percent_encode(address, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let body = format!(
        r#"address={address}&activeId={active_id}&uid={uid}&clientip=&useragent=&latitude={lat}&longitude={lon}&fid=&ifTiJiao=1"#
    );
    Ok(client
        .post(PPT_SIGN)
        .set(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .send_string(&body)?)
}
pub fn chat_group_signcode_sign(
    client: &Agent,
    active_id: &str,
    uid: &str,
    signcode: &str,
) -> Result<Response, Box<ureq::Error>> {
    warn!("`chat_group_signcode_sign` 该函数需要测试！");
    let url =
        format!("{CHAT_GROUP_SIGN}?activeId={active_id}&uid={uid}&clientip=&signCode={signcode}");
    Ok(client.get(&url).call()?)
}
