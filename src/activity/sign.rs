use serde_derive::Deserialize;

use crate::session::{course::Struct课程, Struct签到会话};
use crate::utils::address::Struct位置;
use crate::utils::photo::Struct在线图片;
use crate::utils::{self, 获取unicode字符串定宽显示时应当设置的宽度};
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
pub struct Struct签到 {
    pub 活动id: String,
    pub 签到名: String,
    pub 课程: Struct课程,
    pub other_id: String,
    pub 状态码: i32,
    pub 开始时间戳: i64,
    pub 签到信息: Struct签到信息,
}
#[derive(Debug)]
pub enum Enum签到结果 {
    成功,
    失败 { 失败信息: String },
}
impl Struct签到 {
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
    pub fn 通过文本判断签到结果(text: &str) -> Enum签到结果 {
        match text {
            "success" => Enum签到结果::成功,
            信息 => Enum签到结果::失败 {
                失败信息: if 信息.is_empty() {
                    "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。"
                } else {
                    信息
                }
                .into(),
            },
        }
    }
    async fn 检查签到码是否正确(
        session: &Struct签到会话,
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
    pub course: Struct课程,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Struct签到信息 {
    is_photo: bool,
    is_refresh_qrcode: bool,
    c: String,
}
impl Struct签到 {
    pub fn display(&self, already_course: bool) {
        let name_width = 获取unicode字符串定宽显示时应当设置的宽度(self.签到名.as_str(), 12);
        if already_course {
            println!(
                "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}",
                self.活动id,
                self.签到名,
                self.状态码,
                chrono::DateTime::from_timestamp(self.开始时间戳, 0).unwrap(),
                self.是否有效(),
                width = name_width,
            );
        } else {
            println!(
                "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}, course: {}/{}",
                self.活动id,
                self.签到名,
                self.状态码,
                chrono::DateTime::from_timestamp(self.开始时间戳, 0).unwrap(),
                self.是否有效(),
                self.课程.get_课程号(),
                self.课程.get_课程名(),
                width = name_width,
            );
        }
    }
    pub fn 是否有效(&self) -> bool {
        let time = std::time::SystemTime::from(
            chrono::DateTime::from_timestamp(self.开始时间戳, 0).unwrap(),
        );
        let one_hour = std::time::Duration::from_secs(7200);
        self.状态码 == 1 && std::time::SystemTime::now().duration_since(time).unwrap() < one_hour
    }

    fn 是否为拍照签到(&self) -> bool {
        self.签到信息.is_photo
    }

    pub fn 二维码是否刷新(&self) -> bool {
        self.签到信息.is_refresh_qrcode
    }

    pub fn get_二维码签到时的c参数(&self) -> &str {
        &self.签到信息.c
    }

    pub async fn 通过active_id获取签到信息(
        active_id: &str,
        session: &Struct签到会话,
    ) -> Result<Struct签到信息, reqwest::Error> {
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct GetSignDetailR {
            ifPhoto: i64,
            ifRefreshEwm: i64,
            signCode: Option<String>,
        }
        let r = utils::query::sign_detail(session, active_id).await?;
        let GetSignDetailR {
            ifPhoto,
            ifRefreshEwm,
            signCode,
        } = r.json().await?;
        Ok(Struct签到信息 {
            is_photo: ifPhoto > 0,
            is_refresh_qrcode: ifRefreshEwm > 0,
            c: if let Some(c) = signCode { c } else { "".into() },
        })
    }
    async fn 预签到_analysis部分(
        &self,
        active_id: &str,
        session: &Struct签到会话,
        response_of_presign: reqwest::Response,
    ) -> Result<Enum签到结果, reqwest::Error> {
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
        let _response_of_analysis2 = utils::query::analysis2(session, code).await?;
        #[cfg(debug_assertions)]
        println!(
            "analysis 结果：{}",
            _response_of_analysis2.text().await.unwrap()
        );
        let presign_status = {
            let html = response_of_presign.text().await.unwrap();
            #[cfg(debug_assertions)]
            println!("预签到请求结果：{html}");
            if let Some(start_of_statuscontent_h1) = html.find("id=\"statuscontent\"") {
                let html = &html[start_of_statuscontent_h1 + 19..html.len()];
                let end_of_statuscontent_h1 = html.find('<').unwrap();
                let id为statuscontent的h1的内容 = html[0..end_of_statuscontent_h1].trim();
                if id为statuscontent的h1的内容 == "签到成功" {
                    Enum签到结果::成功
                } else {
                    Enum签到结果::失败 {
                        失败信息: id为statuscontent的h1的内容.into(),
                    }
                }
            } else {
                Enum签到结果::失败 {
                    失败信息: html.into(),
                }
            }
        };
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(presign_status)
    }
    pub async fn 预签到(
        &self,
        session: &Struct签到会话,
    ) -> Result<Enum签到结果, reqwest::Error> {
        let active_id = self.活动id.as_str();
        let uid = session.get_uid();
        let response_of_presign =
            utils::query::pre_sign(session, self.课程.clone(), active_id, uid).await?;
        println!("用户[{}]预签到已请求。", session.get_用户真名());
        self.预签到_analysis部分(active_id, session, response_of_presign)
            .await
    }
    pub async fn 预签到_对于有刷新二维码签到(
        &self,
        c: &str,
        enc: &str,
        session: &Struct签到会话,
    ) -> Result<Enum签到结果, reqwest::Error> {
        let active_id = self.活动id.as_str();
        let uid = session.get_uid();
        let response_of_presign = utils::query::pre_sign_for_qrcode_sign(
            session,
            self.课程.clone(),
            active_id,
            uid,
            c,
            enc,
        )
        .await?;
        println!("用户[{}]预签到已请求。", session.get_用户真名());
        self.预签到_analysis部分(active_id, session, response_of_presign)
            .await
    }
    pub fn get_sign_type(&self) -> Enum签到类型 {
        match self.other_id.parse::<u8>().unwrap_or_else(|e| {
            eprintln!("{}", self.other_id);
            eprintln!("{}", self.课程.get_课程名());
            panic!("{e}")
        }) {
            0 => {
                if self.是否为拍照签到() {
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
impl Struct签到 {
    pub async fn 作为普通签到处理(
        &self,
        session: &Struct签到会话,
    ) -> Result<Enum签到结果, reqwest::Error> {
        let r = utils::query::general_sign(session, self.活动id.as_str()).await?;
        Ok(Self::通过文本判断签到结果(
            &r.text().await.unwrap(),
        ))
    }
    pub async fn 作为签到码签到处理(
        &self,
        session: &Struct签到会话,
        signcode: &str,
    ) -> Result<Enum签到结果, reqwest::Error> {
        if Self::检查签到码是否正确(session, &self.活动id, signcode).await? {
            let r = utils::query::signcode_sign(session, self.活动id.as_str(), signcode).await?;
            Ok(Self::通过文本判断签到结果(
                &r.text().await.unwrap(),
            ))
        } else {
            Ok(Enum签到结果::失败 {
                失败信息: "签到码或手势不正确".into(),
            })
        }
    }
    pub async fn 作为位置签到处理(
        &self,
        address: &Struct位置,
        session: &Struct签到会话,
    ) -> Result<Enum签到结果, reqwest::Error> {
        let r = utils::query::location_sign(session, address, self.活动id.as_str()).await?;
        Ok(Self::通过文本判断签到结果(
            &r.text().await.unwrap(),
        ))
    }
    pub async fn 作为拍照签到处理(
        &self,
        photo: &Struct在线图片,
        session: &Struct签到会话,
    ) -> Result<Enum签到结果, reqwest::Error> {
        let r = utils::query::photo_sign(session, self.活动id.as_str(), photo.get_object_id())
            .await?;
        Ok(Self::通过文本判断签到结果(
            &r.text().await.unwrap(),
        ))
    }
    pub async fn 作为二维码签到处理(
        &self,
        enc: &str,
        address: &Struct位置,
        session: &Struct签到会话,
    ) -> Result<Enum签到结果, reqwest::Error> {
        let r = utils::query::qrcode_sign(session, enc, self.活动id.as_str(), address).await?;
        Ok(Self::通过文本判断签到结果(
            &r.text().await.unwrap(),
        ))
    }
}
impl Struct签到 {
    pub async fn chat_group_pre_sign(
        &self,
        chat_id: &str,
        tuid: &str,
        session: &Struct签到会话,
    ) -> Result<(), reqwest::Error> {
        let id = self.活动id.as_str();
        let uid = session.get_uid();
        let _r = utils::query::chat_group_pre_sign(session, id, uid, chat_id, tuid).await?;

        Ok(())
    }
    pub async fn chat_group_general_sign(
        &self,
        session: &Struct签到会话,
    ) -> Result<(), reqwest::Error> {
        let r = utils::query::chat_group_general_sign(
            session,
            self.活动id.as_str(),
            session.get_uid(),
        )
        .await?;
        println!("{:?}", r.text().await.unwrap());
        Ok(())
    }
    pub async fn chat_group_signcode_sign(
        &self,
        session: &Struct签到会话,
        signcode: &str,
    ) -> Result<(), reqwest::Error> {
        let r = utils::query::chat_group_signcode_sign(
            session,
            self.活动id.as_str(),
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
        session: &Struct签到会话,
    ) -> Result<(), reqwest::Error> {
        let r = utils::query::chat_group_location_sign(
            session,
            address.get_地址(),
            self.活动id.as_str(),
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
        photo: &Struct在线图片,
        session: &Struct签到会话,
    ) -> Result<(), reqwest::Error> {
        let r = utils::query::chat_group_photo_sign(
            session,
            self.活动id.as_str(),
            session.get_uid(),
            photo.get_object_id(),
        )
        .await?;
        println!("{:?}", r.text().await.unwrap());

        Ok(())
    }
}
