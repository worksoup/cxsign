use crate::sign_session::activity::sign::{Photo, SignType};
use crate::sign_session::course::Course;
use crate::sign_session::sign_session::SignSession;
use crate::utils;
use serde_derive::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug)]
pub struct SignActivity<'a> {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub session: &'a SignSession,
    pub status: i32,
    pub start_time_secs: i64,
}

pub struct Address {
    address: String,
    lat: String,
    lon: String,
    altitude: String,
}

impl<'a> SignActivity<'a> {
    pub fn is_available(&self) -> bool {
        let other_id_i64: i64 = self.other_id.parse().unwrap();
        let time = std::time::SystemTime::from(
            chrono::DateTime::from_timestamp(self.start_time_secs, 0).unwrap(),
        );
        let one_hour = std::time::Duration::from_secs(7200);
        other_id_i64 >= 0
            && other_id_i64 <= 5
            && self.status == 1
            && std::time::SystemTime::now().duration_since(time).unwrap() < one_hour
    }
    async fn is_photo(&self) -> Result<bool, reqwest::Error> {
        let r = utils::api::ppt_active_info(self.session, self.id.as_str()).await?;
        let r = r.text().await?;
        Ok(r.find(r#""ifphoto":0"#).is_none())
    }
    pub async fn pre_sign(&self) -> Result<(), reqwest::Error> {
        let active_id = self.id.as_str();
        let uid = self.session.get_uid();
        let _r = utils::api::pre_sign(self.session.deref(), self.course.clone(), active_id, uid);
        Ok(())
    }
    pub async fn chat_group_pre_sign(
        &self,
        chat_id: &str,
        tuid: &str,
    ) -> Result<(), reqwest::Error> {
        let id = self.id.as_str();
        let uid = self.session.get_uid();
        let _r = utils::api::chat_group_pre_sign(self.session, id, uid, chat_id, tuid).await?;
        Ok(())
    }
    async fn general_sign(&self) -> Result<(), reqwest::Error> {
        let r = utils::api::general_sign(
            self.session,
            self.id.as_str(),
            self.session.get_uid(),
            self.session.get_fid(),
            self.session.get_stu_name(),
        )
            .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    async fn chat_group_general_sign(&self) -> Result<(), reqwest::Error> {
        let r =
            utils::api::chat_group_general_sign(self.session, self.id.as_str(), self.session.get_uid())
                .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    async fn location_sign(&self, address: &Address) -> Result<(), reqwest::Error> {
        let r = utils::api::location_sign(
            self.session,
            self.session.get_stu_name(),
            address.address.as_str(),
            self.id.as_str(),
            self.session.get_uid(),
            address.lat.as_str(),
            address.lon.as_str(),
            self.session.get_fid(),
        )
            .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    async fn chat_group_location_sign(&self, address: &Address) -> Result<(), reqwest::Error> {
        let r = utils::api::chat_group_location_sign(
            self.session,
            address.address.as_str(),
            self.id.as_str(),
            self.session.get_uid(),
            address.lat.as_str(),
            address.lon.as_str(),
        )
            .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    async fn to_photo_sign(&self, photo: &Photo) -> Result<(), reqwest::Error> {
        let r = utils::api::photo_sign(
            self.session,
            self.id.as_str(),
            self.session.get_uid(),
            self.session.get_fid(),
            photo.get_object_id(),
            self.session.get_stu_name(),
        )
            .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    async fn chat_group_photo_sign(&self, photo: &Photo) -> Result<(), reqwest::Error> {
        let r = utils::api::chat_group_photo_sign(
            self.session,
            self.id.as_str(),
            self.session.get_uid(),
            photo.get_object_id(),
        )
            .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    async fn qrcode_sign(&self, enc: &str, address: &Address) -> Result<(), reqwest::Error> {
        let r = utils::api::qrcode_sign(
            self.session,
            enc,
            self.session.get_stu_name(),
            address.address.as_str(),
            self.id.as_str(),
            self.session.get_uid(),
            address.lat.as_str(),
            address.lon.as_str(),
            address.altitude.as_str(),
            self.session.get_fid(),
        )
            .await?;
        println!("{:?}", r.text().await?);
        Ok(())
    }
    pub async fn get_sign_type(&self) -> Result<SignType, reqwest::Error> {
        Ok(match self.other_id.parse::<u8>().unwrap() {
            0 => {
                if self.is_photo().await? {
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
        })
    }
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
    pub fn get_sign_result_by_text(text: &str) -> String {
        match text {
            "success" => "成功",
            "fail" => "失败",
            "fail-need-qrcode" => "请发送二维码",
            _ => text,
        }
            .into()
    }
}

#[derive(Debug)]
pub enum Activity<'a> {
    Sign(SignActivity<'a>),
    Other(OtherActivity<'a>),
}

#[derive(Debug)]
pub struct OtherActivity<'a> {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub session: &'a SignSession,
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
