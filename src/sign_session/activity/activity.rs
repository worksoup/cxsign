use crate::sign_session::activity::sign::{Photo, SignType};
use crate::sign_session::course::Course;
use crate::sign_session::sign_session::SignSession;
use crate::utils;
use serde_derive::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SignActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
}
#[derive(Debug)]
pub enum SignState {
    Success,
    Fail(String),
}
impl SignActivity {
    pub fn speculate_type_by_text(text: &str) -> SignType {
        if text.contains("拍照") {
            SignType::Photo
        } else if text.contains("位置") {
            SignType::Location
        } else if text.contains("二维码") {
            SignType::QrCode
        } else if text.contains("手势") {
            // ?
            SignType::Gesture
        } else if text.contains("签到码") {
            // ?
            SignType::SignCode
        } else {
            SignType::Common
        }
    }
    pub fn get_sign_result_by_text(text: &str) -> SignState {
        match text {
            "success" => SignState::Success,
            msg => SignState::Fail(if msg.is_empty() {
                "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。".into()
            } else {
                msg.into()
            }),
        }
        .into()
    }
}
#[derive(Debug)]
pub struct SignActivityRaw {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Debug)]
pub struct Address {
    address: String,
    lat: String,
    lon: String,
    altitude: String,
}
impl Address {
    pub fn parse_str(pos: &str) -> Self {
        let pos: Vec<&str> = pos.split('/').collect();
        assert_eq!(
            pos.len(),
            4,
            "位置信息格式错误！格式为：`addr/lat/lon/alt`."
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

impl SignActivity {
    pub fn display(&self, already_course: bool) {
        let name_width = if UnicodeWidthStr::width(self.name.as_str()) > 12 {
            12
        } else {
            UnicodeWidthStr::width(self.name.as_str()) + 12 - self.name.len()
        };
        if already_course {
            println!(
                "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}",
                self.id,
                self.name,
                self.status,
                chrono::DateTime::from_timestamp(self.start_time_secs, 0).unwrap(),
                self.is_available(),
                width = name_width,
            );
        } else {
            println!(
                "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}, course: {}/{}",
                self.id,
                self.name,
                self.status,
                chrono::DateTime::from_timestamp(self.start_time_secs, 0).unwrap(),
                self.is_available(),
                self.course.get_id(),
                self.course.get_name(),
                width = name_width,
            );
        }
    }
    pub fn is_available(&self) -> bool {
        let time = std::time::SystemTime::from(
            chrono::DateTime::from_timestamp(self.start_time_secs, 0).unwrap(),
        );
        let one_hour = std::time::Duration::from_secs(7200);
        self.status == 1 && std::time::SystemTime::now().duration_since(time).unwrap() < one_hour
    }
    async fn is_photo(&self, session: &SignSession) -> Result<bool, reqwest::Error> {
        let r = utils::api::ppt_active_info(session, self.id.as_str()).await?;
        let r = r.text().await?;
        Ok(r.find(r#""ifphoto":0"#).is_none())
    }
    pub async fn pre_sign(&self, session: &SignSession) -> Result<(), reqwest::Error> {
        let active_id = self.id.as_str();
        let uid = session.get_uid();
        let _r = utils::api::pre_sign(session, self.course.clone(), active_id, uid).await?;
        println!("预签到已请求。");
        Ok(())
    }
    pub async fn general_sign(&self, session: &SignSession) -> Result<SignState, reqwest::Error> {
        let r = utils::api::general_sign(
            session,
            self.id.as_str(),
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
        )
        .await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn signcode_sign(
        &self,
        session: &SignSession,
        signcode: &str,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::api::signcode_sign(
            session,
            self.id.as_str(),
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            signcode,
        )
        .await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn location_sign(
        &self,
        address: &Address,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::api::location_sign(
            session,
            session.get_stu_name(),
            address.address.as_str(),
            self.id.as_str(),
            session.get_uid(),
            address.lat.as_str(),
            address.lon.as_str(),
            session.get_fid(),
        )
        .await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn photo_sign(
        &self,
        photo: &Photo,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::api::photo_sign(
            session,
            self.id.as_str(),
            session.get_uid(),
            session.get_fid(),
            photo.get_object_id(),
            session.get_stu_name(),
        )
        .await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn qrcode_sign(
        &self,
        enc: &str,
        address: &Address,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::api::qrcode_sign(
            session,
            enc,
            session.get_stu_name(),
            address.address.as_str(),
            self.id.as_str(),
            session.get_uid(),
            address.lat.as_str(),
            address.lon.as_str(),
            address.altitude.as_str(),
            session.get_fid(),
        )
        .await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn get_sign_type(&self, session: &SignSession) -> Result<SignType, reqwest::Error> {
        Ok(
            match self.other_id.parse::<u8>().unwrap_or_else(|e| {
                println!("{}", self.other_id);
                println!("{}", self.course.get_name());
                panic!("{e}")
            }) {
                0 => {
                    if self.is_photo(session).await? {
                        SignType::Photo
                    } else {
                        SignType::Common
                    }
                }
                1 => SignType::Unknown,
                2 => SignType::QrCode,
                3 => SignType::Gesture,
                4 => SignType::Location,
                5 => SignType::SignCode,
                _ => SignType::Unknown,
            },
        )
    }
}
impl SignActivity {
    pub async fn chat_group_pre_sign(
        &self,
        chat_id: &str,
        tuid: &str,
        session: &SignSession,
    ) -> Result<(), reqwest::Error> {
        let id = self.id.as_str();
        let uid = session.get_uid();
        let _r = utils::api::chat_group_pre_sign(session, id, uid, chat_id, tuid).await?;

        Ok(())
    }
    pub async fn chat_group_general_sign(
        &self,
        session: &SignSession,
    ) -> Result<(), reqwest::Error> {
        let r = utils::api::chat_group_general_sign(session, self.id.as_str(), session.get_uid())
            .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    pub async fn chat_group_signcode_sign(
        &self,
        session: &SignSession,
        signcode: &str,
    ) -> Result<(), reqwest::Error> {
        let r = utils::api::chat_group_signcode_sign(
            session,
            self.id.as_str(),
            session.get_uid(),
            signcode,
        )
        .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    pub async fn chat_group_location_sign(
        &self,
        address: &Address,
        session: &SignSession,
    ) -> Result<(), reqwest::Error> {
        let r = utils::api::chat_group_location_sign(
            session,
            address.address.as_str(),
            self.id.as_str(),
            session.get_uid(),
            address.lat.as_str(),
            address.lon.as_str(),
        )
        .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    pub async fn chat_group_photo_sign(
        &self,
        photo: &Photo,
        session: &SignSession,
    ) -> Result<(), reqwest::Error> {
        let r = utils::api::chat_group_photo_sign(
            session,
            self.id.as_str(),
            session.get_uid(),
            photo.get_object_id(),
        )
        .await?;
        println!("{:?}", r.text().await.unwrap());

        Ok(())
    }
}
#[derive(Debug)]
pub enum Activity {
    Sign(SignActivity),
    Other(OtherActivity),
}

#[derive(Debug)]
pub struct OtherActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ActivityRaw {
    pub nameOne: String,
    pub id: i64,
    pub otherId: Option<String>,
    pub status: i32,
    pub startTime: u64,
}

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Data {
    pub activeList: Vec<ActivityRaw>,
}

#[derive(Deserialize, Serialize)]
pub struct GetActivityR {
    pub data: Option<Data>,
}
