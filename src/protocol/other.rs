// // web 聊天页
// static WEB_IM: &str = "https://im.chaoxing.com/webim/me";

// // 无课程群聊的预签到
// static CHAT_GROUP_PRE_SIGN: &str = "https://mobilelearn.chaoxing.com/sign/preStuSign";
// pub async fn chat_group_pre_sign(
//     client: &Client,
//     active_id: &str,
//     uid: &str,
//     chat_id: &str,
//     tuid: &str,
// ) -> Result<Response, reqwest::Error> {
//     let url = CHAT_GROUP_PRE_SIGN;
//     let url = format!("{url}?activeId={active_id}&code=&uid={uid}&courseId=null&classId=0&general=0&chatId={chat_id}&appType=0&tid={tuid}&atype=null&sys=0");
//     client.get(url).send().await
// }
// // 无课程群聊的签到
// static CHAT_GROUP_SIGN: &str = "https://mobilelearn.chaoxing.com/sign/stuSignajax";
// pub async fn chat_group_general_sign(
//     client: &Client,
//     active_id: &str,
//     uid: &str,
// ) -> Result<Response, reqwest::Error> {
//     let url = CHAT_GROUP_SIGN;
//     let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=");
//     client.get(url).send().await
// }

// pub async fn chat_group_photo_sign(
//     client: &Client,
//     active_id: &str,
//     uid: &str,
//     object_id: &str,
// ) -> Result<Response, reqwest::Error> {
//     let url = CHAT_GROUP_SIGN;
//     let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&fid=0&objectId={object_id}");
//     client.get(url).send().await
// }
// pub async fn chat_group_location_sign(
//     client: &Client,
//     address: &str,
//     active_id: &str,
//     uid: &str,
//     lat: &str,
//     lon: &str,
// ) -> Result<Response, reqwest::Error> {
//     let address =
//         percent_encoding::utf8_percent_encode(address, percent_encoding::NON_ALPHANUMERIC)
//             .to_string();
//     let body = format!(
//         r#"address={address}&activeId={active_id}&uid={uid}&clientip=&useragent=&latitude={lat}&longitude={lon}&fid=&ifTiJiao=1"#
//     );
//     let headers = {
//         let mut h = HeaderMap::new();
//         h.insert(
//             reqwest::header::CONTENT_TYPE,
//             "application/x-www-form-urlencoded; charset=UTF-8"
//                 .parse()
//                 .unwrap(),
//         );
//         h
//     };
//     let url = PPT_SIGN;
//     client.post(url).headers(headers).body(body).send().await
// }
// pub async fn chat_group_signcode_sign(
//     client: &Client,
//     active_id: &str,
//     uid: &str,
//     signcode: &str,
// ) -> Result<Response, reqwest::Error> {
//     eprintln!("`chat_group_signcode_sign` 该函数需要测试！");
//     let url = CHAT_GROUP_SIGN;
//     let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&signCode={signcode}");
//     client.get(url).send().await
// }
