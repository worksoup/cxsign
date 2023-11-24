use crate::sign_session::course::Course;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};

pub static UA: &'static str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 (schild:eaf4fb193ec970c0a9775e2a27b0232b) (device:iPhone11,2) Language/zh-Hans com.ssreader.ChaoXingStudy/ChaoXingStudy_3_6.0.2_ios_phone_202209281930_99 (@Kalimdor)_1665876591620212942";

// 登陆页
static LOGIN_PAGE: &'static str =
    "http://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com";
#[allow(unused)]
pub async fn login_page(client: &Client) -> Result<Response, reqwest::Error> {
    Ok(client.get(LOGIN_PAGE).send().await?)
}
// 明文密码登陆
static LOGIN: &'static str = "https://passport2-api.chaoxing.com/v11/loginregister";
pub async fn login(client: &Client, uname: &str, pwd: &str) -> Result<Response, reqwest::Error> {
    let body = format!("code={pwd}&cx_xxt_passport=json&uname={uname}&loginType=1&roleSelect=true");
    let url = {
        let mut str = String::from(LOGIN);
        str.push_str("?");
        str.push_str(body.as_str());
        str
    };
    Ok(client.get(url).send().await?)
}
// 非明文密码登陆
static LOGIN_ENC: &'static str = "http://passport2.chaoxing.com/fanyalogin";
pub async fn login_enc(
    client: &Client,
    uname: &str,
    pwd_enc: &str,
) -> Result<Response, reqwest::Error> {
    let body = format!("uname={uname}&password={pwd_enc}&fid=-1&t=true&refer=https%253A%252F%252Fi.chaoxing.com&forbidotherlogin=0&validate=");
    let headers = {
        let mut header = HeaderMap::new();
        header.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        header.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        header
    };
    let response = client
        .post(LOGIN_ENC)
        .headers(headers)
        .body(body)
        .send()
        .await?;
    Ok(response)
}
// 预签到
static PRE_SIGN: &'static str = "https://mobilelearn.chaoxing.com/newsign/preSign";
pub async fn pre_sign(
    client: &Client,
    course: Course,
    active_id: &str,
    uid: &str,
) -> Result<Response, reqwest::Error> {
    let course_id = course.get_id();
    let class_id = course.get_class_id();
    let url = PRE_SIGN;
    let url = format!("{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0");
    Ok(client.get(url).send().await?)
}
pub async fn pre_sign_for_qrcode_sign(
    client: &Client,
    course: Course,
    active_id: &str,
    uid: &str,
    c: &str,
    enc: &str,
) -> Result<Response, reqwest::Error> {
    let course_id = course.get_id();
    let class_id = course.get_class_id();
    let url = PRE_SIGN;
    let ex_args = format!("SIGNIN:aid={active_id}&source=15&Code={c}&enc={enc}");
    let ex_args = "&rcode=".to_owned()
        + percent_encoding::utf8_percent_encode(
            ex_args.as_str(),
            percent_encoding::NON_ALPHANUMERIC,
        )
        .to_string()
        .as_str();
    let url = format!("{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0&rcode={ex_args}");
    Ok(client.get(url).send().await?)
}
// analysis
static ANALYSIS: &'static str = "https://mobilelearn.chaoxing.com/pptSign/analysis";
pub async fn analysis(client: &Client, active_id: &str) -> Result<Response, reqwest::Error> {
    let url = ANALYSIS;
    let url = format!("{url}?vs=1&DB_STRATEGY=RANDOM&aid={active_id}");
    Ok(client.get(url).send().await?)
}
// analysis 2
static ANALYSIS2: &'static str = "https://mobilelearn.chaoxing.com/pptSign/analysis2";
pub async fn analysis2(client: &Client, code: &str) -> Result<Response, reqwest::Error> {
    let url = ANALYSIS2;
    let url = format!("{url}?DB_STRATEGY=RANDOM&code={code}");
    Ok(client.get(url).send().await?)
}
// 签到
static PPT_SIGN: &'static str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";
pub async fn general_sign(
    client: &Client,
    active_id: &str,
    uid: &str,
    fid: &str,
    stu_name: &str,
) -> Result<Response, reqwest::Error> {
    let url = PPT_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}");
    Ok(client.get(url).send().await?)
}
pub async fn photo_sign(
    client: &Client,
    active_id: &str,
    uid: &str,
    fid: &str,
    object_id: &str,
    stu_name: &str,
) -> Result<Response, reqwest::Error> {
    // NOTE 存疑。
    let name = percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC)
        .to_string();
    let url = PPT_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={name}");
    Ok(client.get(url).send().await?)
}
pub async fn qrcode_sign(
    client: &Client,
    enc: &str,
    stu_name: &str,
    address: &str,
    active_id: &str,
    uid: &str,
    lat: &str,
    lon: &str,
    altitude: &str,
    fid: &str,
) -> Result<Response, reqwest::Error> {
    let url = PPT_SIGN;
    let location = format!(
        r#"{{"result":"1","address":"{address}","latitude":{lat},"longitude":{lon},"altitude":{altitude}}}"#
    );
    let location =
        percent_encoding::utf8_percent_encode(&location, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let url = format!(
        r#"{url}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location={location}&latitude=-1&longitude=-1&fid={fid}&appType=15"#
    );
    Ok(client.get(url).send().await?)
}
pub async fn location_sign(
    client: &Client,
    stu_name: &str,
    address: &str,
    active_id: &str,
    uid: &str,
    lat: &str,
    lon: &str,
    fid: &str,
) -> Result<Response, reqwest::Error> {
    let url = PPT_SIGN;
    let url = format!("{url}?name={stu_name}&address={address}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao=1&validate=");
    Ok(client.get(url).send().await?)
}
pub async fn signcode_sign(
    client: &Client,
    active_id: &str,
    uid: &str,
    fid: &str,
    stu_name: &str,
    signcode: &str,
) -> Result<Response, reqwest::Error> {
    let url = PPT_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}");
    Ok(client.get(url).send().await?)
}

// 签到码检查
static CHECK_SINGCODE: &'static str =
    "https://mobilelearn.chaoxing.com/widget/sign/pcStuSignController/checkSignCode";
pub async fn check_signcode(
    client: &Client,
    active_id: &str,
    signcode: &str,
) -> Result<Response, reqwest::Error> {
    let r = client
        .get(String::from(CHECK_SINGCODE) + "?activeId=" + active_id + "&signCode=" + signcode)
        .send()
        .await?;
    Ok(r)
}
// ppt_active_info
// 签到信息获取
static PPT_ACTIVE_INFO: &'static str =
    "https://mobilelearn.chaoxing.com/v2/apis/active/getPPTActiveInfo";
pub async fn ppt_active_info(client: &Client, active_id: &str) -> Result<Response, reqwest::Error> {
    let r = client
        .get(String::from(PPT_ACTIVE_INFO) + "?activeId=" + active_id)
        .send()
        .await?;
    Ok(r)
}
// 签到信息获取 2
static SIGN_DETAIL: &'static str = "https://mobilelearn.chaoxing.com/newsign/signDetail";
pub async fn sign_detail(client: &Client, active_id: &str) -> Result<Response, reqwest::Error> {
    let r = client
        .get(String::from(SIGN_DETAIL) + "?activePrimaryId=" + active_id + "&type=1")
        .send()
        .await?;
    Ok(r)
}
// 获取课程（`chaoxing-sign-cli` 并未使用）
static BACK_CLAZZ_DATA: &'static str = "http://mooc1-api.chaoxing.com/mycourse/backclazzdata";
pub async fn back_clazz_data(client: &Client) -> Result<Response, reqwest::Error> {
    let url = String::from(BACK_CLAZZ_DATA) + "?view=json&rss=1";
    Ok(client.get(url).send().await?)
}
// 查询活动 1
static ACTIVE_LIST: &'static str =
    "https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist";
pub async fn active_list(client: &Client, course: Course) -> Result<Response, reqwest::Error> {
    let url = {
        let mut url = String::from(ACTIVE_LIST);
        url.push_str("?fid=0&courseId=");
        url.push_str(course.get_id().to_string().as_str());
        url.push_str("&classId=");
        url.push_str(course.get_class_id().to_string().as_str());
        url.push_str("&showNotStartedActive=0&_=");
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        url.push_str(time.as_str());
        url
    };
    // println!("{url}");
    Ok(client.get(url).send().await?)
}
// 查询活动 2
static TASK_ACTIVE_LIST: &'static str =
    "https://mobilelearn.chaoxing.com/ppt/activeAPI/taskactivelist";
// 账号设置页
static ACCOUNT_MANAGE: &'static str = "http://passport2.chaoxing.com/mooc/accountManage";
pub async fn account_manage(client: &Client) -> Result<Response, reqwest::Error> {
    Ok(client.get(ACCOUNT_MANAGE).send().await?)
}
// 超星网盘页
static PAN_CHAOXING: &'static str = "https://pan-yz.chaoxing.com";
pub async fn pan_chaoxing(client: &Client) -> Result<Response, reqwest::Error> {
    let url = PAN_CHAOXING;
    Ok(client.get(url).send().await?)
}
// 网盘列表
static PAN_LIST: &'static str = "https://pan-yz.chaoxing.com/opt/listres";
pub async fn pan_list(
    client: &Client,
    parent_id: &str,
    enc: &str,
) -> Result<Response, reqwest::Error> {
    let url = PAN_LIST;
    let url = format!("{url}?puid=0&shareid=0&parentId={parent_id}&page=1&size=50&enc={enc}");
    Ok(client.post(url).send().await?)
}
// 获取超星云盘的 token
static PAN_TOKEN: &'static str = "https://pan-yz.chaoxing.com/api/token/uservalid";
pub async fn pan_token(client: &Client) -> Result<Response, reqwest::Error> {
    Ok(client.get(PAN_TOKEN).send().await?)
}
// 网盘上传接口
static PAN_UPLOAD: &'static str = "https://pan-yz.chaoxing.com/upload";
pub async fn pan_upload(
    client: &Client,
    buffer: Vec<u8>,
    uid: &str,
    token: &str,
    file_name: &str,
) -> Result<Response, reqwest::Error> {
    let part = reqwest::multipart::Part::bytes(buffer).file_name(file_name.to_string());
    let form_data = reqwest::multipart::Form::new()
        .part("file", part)
        .text("puid", uid.to_string());
    let url = PAN_UPLOAD;
    let url = format!("{url}?_from=mobilelearn&_token={token}");
    Ok(client.post(url).multipart(form_data).send().await?)
}
// web 聊天页
static WEB_IM: &'static str = "https://im.chaoxing.com/webim/me";
// 无课程群聊的预签到
static CHAT_GROUP_PRE_SIGN: &'static str = "https://mobilelearn.chaoxing.com/sign/preStuSign";
pub async fn chat_group_pre_sign(
    client: &Client,
    active_id: &str,
    uid: &str,
    chat_id: &str,
    tuid: &str,
) -> Result<Response, reqwest::Error> {
    let url = CHAT_GROUP_PRE_SIGN;
    let url = format!("{url}?activeId={active_id}&code=&uid={uid}&courseId=null&classId=0&general=0&chatId={chat_id}&appType=0&tid={tuid}&atype=null&sys=0");
    let r = client.get(url).send().await?;
    Ok(r)
}
// 无课程群聊的签到
static CHAT_GROUP_SIGN: &'static str = "https://mobilelearn.chaoxing.com/sign/stuSignajax";
pub async fn chat_group_general_sign(
    client: &Client,
    active_id: &str,
    uid: &str,
) -> Result<Response, reqwest::Error> {
    let url = CHAT_GROUP_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=");
    Ok(client.get(url).send().await?)
}

pub async fn chat_group_photo_sign(
    client: &Client,
    active_id: &str,
    uid: &str,
    object_id: &str,
) -> Result<Response, reqwest::Error> {
    let url = CHAT_GROUP_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&fid=0&objectId={object_id}");
    Ok(client.get(url).send().await?)
}
pub async fn chat_group_location_sign(
    client: &Client,
    address: &str,
    active_id: &str,
    uid: &str,
    lat: &str,
    lon: &str,
) -> Result<Response, reqwest::Error> {
    let address =
        percent_encoding::utf8_percent_encode(address, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let body = format!(
        r#"address={address}&activeId={active_id}&uid={uid}&clientip=&useragent=&latitude={lat}&longitude={lon}&fid=&ifTiJiao=1"#
    );
    let headers = {
        let mut h = HeaderMap::new();
        h.insert(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded; charset=UTF-8"
                .parse()
                .unwrap(),
        );
        h
    };
    let url = PPT_SIGN;
    Ok(client.post(url).headers(headers).body(body).send().await?)
}
pub async fn chat_group_signcode_sign(
    client: &Client,
    active_id: &str,
    uid: &str,
    signcode: &str,
) -> Result<Response, reqwest::Error> {
    eprintln!("`chat_group_signcode_sign` 该函数需要测试！");
    let url = CHAT_GROUP_SIGN;
    let url = format!("{url}?activeId={active_id}&uid={uid}&clientip=&signCode={signcode}");
    Ok(client.get(url).send().await?)
}
