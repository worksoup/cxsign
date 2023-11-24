use serde_derive::Deserialize;

use crate::sign_session::{course::Course, session::SignSession};
use crate::utils::address::Address;
use crate::utils::photo::Photo;
use crate::utils::{self, get_unicode_correct_display_width};
pub enum SignType {
    // 拍照签到
    Photo,
    // 普通签到
    Common,
    // 二维码签到
    QrCode,
    // 手势签到
    Gesture,
    // 位置签到
    Location,
    // 签到码签到
    SignCode,
    // 未知
    Unknown,
}
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
    async fn check_signcode(
        session: &SignSession,
        active_id: &str,
        signcode: &str,
    ) -> Result<bool, reqwest::Error> {
        #[derive(Deserialize)]
        struct CheckR {
            #[allow(unused)]
            result: i64,
        }
        let CheckR { result } = utils::api::check_signcode(&session, active_id, signcode)
            .await?
            .json()
            .await
            .unwrap();
        Ok(result == 1)
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
impl SignActivity {
    pub fn display(&self, already_course: bool) {
        let name_width = get_unicode_correct_display_width(self.name.as_str(), 12);
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
        let r = utils::api::sign_detail(session, self.id.as_str()).await?;
        let r = r.text().await?;
        Ok(r.find(r#""ifphoto":0"#).is_none())
    }
    // async fn get_sign_detial(&self, session: &SignSession) -> Result<bool, reqwest::Error> {
    //     let r = utils::api::sign_detail(session, self.id.as_str()).await?;
    //     let r = r.text().await?;
    //     Ok(r.find(r#""ifphoto":0"#).is_none())
    // }
    async fn pre_sign_internal(
        &self,
        active_id: &str,
        session: &SignSession,
        response_of_presign: reqwest::Response,
    ) -> Result<SignState, reqwest::Error> {
        let response_of_analysis = utils::api::analysis(session, active_id).await?;
        let data = response_of_analysis.text().await.unwrap();
        let code = {
            let start_of_code = data.find("code='+'").unwrap() + 8;
            let data = &data[start_of_code..data.len()];
            let end_of_code = data.find('\'').unwrap();
            &data[0..end_of_code]
        };
        println!("code: {code:?}");
        let response_of_analysis2 = utils::api::analysis2(session, code).await?;
        println!(
            "analysis 结果：{}",
            response_of_analysis2.text().await.unwrap()
        );
        let presign_status = {
            let html = response_of_presign.text().await.unwrap();
            if let Some(start_of_statuscontent_h1) = html.find("id=\"statuscontent\"") {
                let html = &html[start_of_statuscontent_h1 + 19..html.len()];
                let end_of_statuscontent_h1 = html.find('>').unwrap();
                let statuscontent_h1_content = html[0..end_of_statuscontent_h1].trim();
                if statuscontent_h1_content == "签到成功" {
                    SignState::Success
                } else {
                    SignState::Fail(statuscontent_h1_content.into())
                }
            } else {
                SignState::Fail("还未签到".into())
            }
        };
        Ok(presign_status)
    }
    pub async fn pre_sign(&self, session: &SignSession) -> Result<SignState, reqwest::Error> {
        let active_id = self.id.as_str();
        let uid = session.get_uid();
        let response_of_presign =
            utils::api::pre_sign(session, self.course.clone(), active_id, uid).await?;
        println!("预签到已请求。");
        self.pre_sign_internal(active_id, session, response_of_presign)
            .await
    }
    pub async fn pre_sign_for_qrcode_sign(
        &self,
        c: &str,
        enc: &str,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let active_id = self.id.as_str();
        let uid = session.get_uid();
        let response_of_presign = utils::api::pre_sign_for_qrcode_sign(
            session,
            self.course.clone(),
            active_id,
            uid,
            c,
            enc,
        )
        .await?;
        println!("预签到已请求。");
        self.pre_sign_internal(active_id, session, response_of_presign)
            .await
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
        if Self::check_signcode(session, &self.id, signcode).await? {
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
        } else {
            Ok(SignState::Fail("签到码或手势不正确".into()))
        }
    }
    pub async fn location_sign(
        &self,
        address: &Address,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::api::location_sign(
            session,
            session.get_stu_name(),
            address.get_addr(),
            self.id.as_str(),
            session.get_uid(),
            address.get_lat(),
            address.get_lon(),
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
    pub async fn sign_by_qrcode(
        &self,
        enc: &str,
        address: &Address,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::api::qrcode_sign(
            session,
            enc,
            session.get_stu_name(),
            address.get_addr(),
            self.id.as_str(),
            session.get_uid(),
            address.get_lat(),
            address.get_lon(),
            address.get_alt(),
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
            address.get_addr(),
            self.id.as_str(),
            session.get_uid(),
            address.get_lat(),
            address.get_lon(),
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
