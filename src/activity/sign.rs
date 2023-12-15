use serde_derive::Deserialize;

use crate::session::{course::Course, SignSession};
use crate::utils::address::Struct位置;
use crate::utils::photo::Photo;
use crate::utils::{self, get_unicode_correct_display_width};
pub enum Enum签到类型 {
    // 拍照签到
    拍照签到,
    // 普通签到
    普通签到,
    // 二维码签到
    二维码签到,
    // 手势签到
    手势签到,
    // 位置签到
    位置签到,
    // 签到码签到
    签到码签到,
    // 未知
    非已知签到,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SignActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
    pub detail: SignDetail,
}
#[derive(Debug)]
pub enum SignState {
    Success,
    Fail(String),
}
impl SignActivity {
    pub fn speculate_type_by_text(text: &str) -> Enum签到类型 {
        if text.contains("拍照") {
            Enum签到类型::拍照签到
        } else if text.contains("位置") {
            Enum签到类型::位置签到
        } else if text.contains("二维码") {
            Enum签到类型::二维码签到
        } else if text.contains("手势") {
            // ?
            Enum签到类型::手势签到
        } else if text.contains("签到码") {
            // ?
            Enum签到类型::签到码签到
        } else {
            Enum签到类型::普通签到
        }
    }
    pub fn get_sign_result_by_text(text: &str) -> SignState {
        match text {
            "success" => SignState::Success,
            msg => SignState::Fail(
                if msg.is_empty() {
                    "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。"
                } else {
                    msg
                }
                .into(),
            ),
        }
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
        let CheckR { result } = utils::query::check_signcode(session, active_id, signcode)
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
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SignDetail {
    is_photo: bool,
    is_refresh_qrcode: bool,
    c: String,
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
                self.course.get_course_id(),
                self.course.get_课程名(),
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

    fn is_photo(&self) -> bool {
        self.detail.is_photo
    }

    pub fn is_refresh_qrcode(&self) -> bool {
        self.detail.is_refresh_qrcode
    }

    pub fn get_c_of_qrcode_sign(&self) -> &str {
        &self.detail.c
    }

    pub async fn get_sign_detial_by_active_id(
        active_id: &str,
        session: &SignSession,
    ) -> Result<SignDetail, reqwest::Error> {
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct SignDetailRaw {
            ifPhoto: i64,
            ifRefreshEwm: i64,
            signCode: Option<String>,
        }
        let r = utils::query::sign_detail(session, active_id).await?;
        let SignDetailRaw {
            ifPhoto,
            ifRefreshEwm,
            signCode,
        } = r.json().await?;
        Ok(SignDetail {
            is_photo: ifPhoto > 0,
            is_refresh_qrcode: ifRefreshEwm > 0,
            c: if let Some(c) = signCode { c } else { "".into() },
        })
    }
    async fn pre_sign_internal(
        &self,
        active_id: &str,
        session: &SignSession,
        response_of_presign: reqwest::Response,
    ) -> Result<SignState, reqwest::Error> {
        let response_of_analysis = utils::query::analysis(session, active_id).await?;
        let data = response_of_analysis.text().await.unwrap();
        let code = {
            let start_of_code = data.find("code='+'").unwrap() + 8;
            let data = &data[start_of_code..data.len()];
            let end_of_code = data.find('\'').unwrap();
            &data[0..end_of_code]
        };
        #[cfg(debug_assertions)]
        println!("code: {code:?}");
        let response_of_analysis2 = utils::query::analysis2(session, code).await?;
        #[cfg(debug_assertions)]
        println!(
            "analysis 结果：{}",
            response_of_analysis2.text().await.unwrap()
        );
        let presign_status = {
            let html = response_of_presign.text().await.unwrap();
            #[cfg(debug_assertions)]
            println!("预签到请求结果：{html}");
            if let Some(start_of_statuscontent_h1) = html.find("id=\"statuscontent\"") {
                let html = &html[start_of_statuscontent_h1 + 19..html.len()];
                let end_of_statuscontent_h1 = html.find('<').unwrap();
                let statuscontent_h1_content = html[0..end_of_statuscontent_h1].trim();
                if statuscontent_h1_content == "签到成功" {
                    SignState::Success
                } else {
                    SignState::Fail(statuscontent_h1_content.into())
                }
            } else {
                SignState::Fail(html.into())
            }
        };
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(presign_status)
    }
    pub async fn pre_sign(&self, session: &SignSession) -> Result<SignState, reqwest::Error> {
        let active_id = self.id.as_str();
        let uid = session.get_uid();
        let response_of_presign =
            utils::query::pre_sign(session, self.course.clone(), active_id, uid).await?;
        println!("用户[{}]预签到已请求。", session.get_用户真名());
        self.pre_sign_internal(active_id, session, response_of_presign)
            .await
    }
    pub async fn pre_sign_for_refresh_qrcode_sign(
        &self,
        c: &str,
        enc: &str,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let active_id = self.id.as_str();
        let uid = session.get_uid();
        let response_of_presign = utils::query::pre_sign_for_qrcode_sign(
            session,
            self.course.clone(),
            active_id,
            uid,
            c,
            enc,
        )
        .await?;
        println!("用户[{}]预签到已请求。", session.get_用户真名());
        self.pre_sign_internal(active_id, session, response_of_presign)
            .await
    }
    pub fn get_sign_type(&self) -> Enum签到类型 {
        match self.other_id.parse::<u8>().unwrap_or_else(|e| {
            eprintln!("{}", self.other_id);
            eprintln!("{}", self.course.get_课程名());
            panic!("{e}")
        }) {
            0 => {
                if self.is_photo() {
                    Enum签到类型::拍照签到
                } else {
                    Enum签到类型::普通签到
                }
            }
            1 => Enum签到类型::非已知签到,
            2 => Enum签到类型::二维码签到,
            3 => Enum签到类型::手势签到,
            4 => Enum签到类型::位置签到,
            5 => Enum签到类型::签到码签到,
            _ => Enum签到类型::非已知签到,
        }
    }
}
impl SignActivity {
    pub async fn sign_common(&self, session: &SignSession) -> Result<SignState, reqwest::Error> {
        let r = utils::query::general_sign(session, self.id.as_str()).await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn sign_by_signcode(
        &self,
        session: &SignSession,
        signcode: &str,
    ) -> Result<SignState, reqwest::Error> {
        if Self::check_signcode(session, &self.id, signcode).await? {
            let r = utils::query::signcode_sign(session, self.id.as_str(), signcode).await?;
            Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
        } else {
            Ok(SignState::Fail("签到码或手势不正确".into()))
        }
    }
    pub async fn sign_by_location(
        &self,
        address: &Struct位置,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::query::location_sign(session, address, self.id.as_str()).await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn sign_by_photo(
        &self,
        photo: &Photo,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::query::photo_sign(session, self.id.as_str(), photo.get_object_id()).await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
    }
    pub async fn sign_by_qrcode(
        &self,
        enc: &str,
        address: &Struct位置,
        session: &SignSession,
    ) -> Result<SignState, reqwest::Error> {
        let r = utils::query::qrcode_sign(session, enc, self.id.as_str(), address).await?;
        Ok(Self::get_sign_result_by_text(&r.text().await.unwrap()))
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
        let _r = utils::query::chat_group_pre_sign(session, id, uid, chat_id, tuid).await?;

        Ok(())
    }
    pub async fn chat_group_general_sign(
        &self,
        session: &SignSession,
    ) -> Result<(), reqwest::Error> {
        let r = utils::query::chat_group_general_sign(session, self.id.as_str(), session.get_uid())
            .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    pub async fn chat_group_signcode_sign(
        &self,
        session: &SignSession,
        signcode: &str,
    ) -> Result<(), reqwest::Error> {
        let r = utils::query::chat_group_signcode_sign(
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
        address: &Struct位置,
        session: &SignSession,
    ) -> Result<(), reqwest::Error> {
        let r = utils::query::chat_group_location_sign(
            session,
            address.get_地址(),
            self.id.as_str(),
            session.get_uid(),
            address.get_纬度(),
            address.get_经度(),
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
        let r = utils::query::chat_group_photo_sign(
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
