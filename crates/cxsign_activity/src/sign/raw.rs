use crate::protocol;
use crate::sign::{
    GestureSign, LocationSign, NormalQrCodeSign, NormalSign, PhotoSign, QrCodeSign,
    RefreshQrCodeSign, Sign, SignDetail, SignResult, SignTrait, SigncodeSign,
};
use cxsign_types::Course;
use cxsign_user::Session;
use cxsign_utils::get_width_str_should_be;
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct RawSign {
    pub start_timestamp: i64,
    pub active_id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status_code: i32,
}
impl SignTrait for RawSign {
    fn as_inner(&self) -> &RawSign {
        self
    }
    fn pre_sign(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let active_id = self.active_id.as_str();
        let uid = session.get_uid();
        let response_of_pre_sign =
            protocol::pre_sign(session, self.course.clone(), active_id, uid)?;
        info!("用户[{}]预签到已请求。", session.get_stu_name());
        self.analysis_after_presign(active_id, session, response_of_pre_sign)
    }
    unsafe fn sign_unchecked(&self, session: &Session) -> Result<SignResult, Box<ureq::Error>> {
        let r = protocol::general_sign(session, self.active_id.as_str())?;
        Ok(self.guess_sign_result_by_text(&r.into_string().unwrap()))
    }
}
impl Display for RawSign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name_width = get_width_str_should_be(self.name.as_str(), 12);
        write!(
            f,
            "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}, course: {}/{}",
            self.active_id,
            self.name,
            self.status_code,
            chrono::DateTime::from_timestamp(self.start_timestamp, 0).unwrap(),
            self.is_valid(),
            self.course.get_id(),
            self.course.get_name(),
            width = name_width,
        )
    }
}
impl RawSign {
    pub(crate) fn check_signcode(
        session: &Session,
        active_id: &str,
        signcode: &str,
    ) -> Result<bool, Box<ureq::Error>> {
        #[derive(Deserialize)]
        struct CheckR {
            #[allow(unused)]
            result: i64,
        }
        let CheckR { result } = crate::protocol::check_signcode(session, active_id, signcode)?
            .into_json()
            .unwrap();
        Ok(result == 1)
    }
    pub(crate) fn get_sign_detail(
        active_id: &str,
        session: &Session,
    ) -> Result<SignDetail, Box<ureq::Error>> {
        #[derive(Deserialize)]
        struct GetSignDetailR {
            #[serde(rename = "ifPhoto")]
            is_photo_sign: i64,
            #[serde(rename = "ifRefreshEwm")]
            is_refresh_qrcode: i64,
            #[serde(rename = "signCode")]
            sign_code: Option<String>,
        }
        let r = protocol::sign_detail(session, active_id)?;
        let GetSignDetailR {
            is_photo_sign,
            is_refresh_qrcode,
            sign_code,
        } = r.into_json().unwrap();
        Ok(SignDetail {
            is_photo: is_photo_sign > 0,
            is_refresh_qrcode: is_refresh_qrcode > 0,
            c: if let Some(c) = sign_code {
                c
            } else {
                "".into()
            },
        })
    }
}

impl RawSign {
    pub fn to_sign(self, session: &Session) -> Sign {
        if let Ok(sign_detail) = RawSign::get_sign_detail(self.active_id.as_str(), session) {
            let r#else = |e| {
                error!("{}", self.other_id);
                error!("{}", self.course.get_name());
                panic!("{e}")
            };
            match self.other_id.parse::<u8>().unwrap_or_else(r#else) {
                0 => {
                    if sign_detail.is_photo {
                        Sign::Photo(PhotoSign {
                            raw_sign: self,
                            photo: None,
                        })
                    } else {
                        Sign::Normal(NormalSign { raw_sign: self })
                    }
                }
                1 => Sign::Unknown(self),
                2 => {
                    if sign_detail.is_refresh_qrcode {
                        Sign::QrCode(QrCodeSign::RefreshQrCodeSign(RefreshQrCodeSign {
                            enc: None,
                            c: sign_detail.c.clone(),
                            raw_sign: self,
                            location: None,
                        }))
                    } else {
                        Sign::QrCode(QrCodeSign::NormalQrCodeSign(NormalQrCodeSign {
                            enc: None,
                            c: sign_detail.c.clone(),
                            raw_sign: self,
                            location: None,
                        }))
                    }
                }
                3 => Sign::Gesture(GestureSign {
                    raw_sign: self,
                    gesture: None,
                }),
                4 => Sign::Location(LocationSign {
                    raw_sign: self,
                    location: None,
                    has_range: false,
                }),
                5 => Sign::Signcode(SigncodeSign {
                    signcode: None,
                    raw_sign: self,
                }),
                _ => Sign::Unknown(self),
            }
        } else {
            Sign::Unknown(self)
        }
    }
    pub fn fmt_without_course_info(&self) -> String {
        let name_width = get_width_str_should_be(self.name.as_str(), 12);
        format!(
            "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}",
            self.active_id,
            self.name,
            self.status_code,
            chrono::DateTime::from_timestamp(self.start_timestamp, 0).unwrap(),
            self.is_valid(),
            width = name_width,
        )
    }

    pub(crate) fn analysis_after_presign(
        &self,
        active_id: &str,
        session: &Session,
        response_of_presign: ureq::Response,
    ) -> Result<SignResult, Box<ureq::Error>> {
        let response_of_analysis = protocol::analysis(session, active_id)?;
        let data = response_of_analysis.into_string().unwrap();
        let code = {
            let start_of_code = data.find("code='+'").unwrap() + 8;
            let data = &data[start_of_code..data.len()];
            let end_of_code = data.find('\'').unwrap();
            &data[0..end_of_code]
        };
        debug!("code: {code:?}");
        let _response_of_analysis2 = protocol::analysis2(session, code)?;
        debug!(
            "analysis 结果：{}",
            _response_of_analysis2.into_string().unwrap()
        );
        let pre_sign_status = {
            let html = response_of_presign.into_string().unwrap();
            trace!("预签到请求结果：{html}");
            if let Some(start_of_statuscontent_h1) = html.find("id=\"statuscontent\"") {
                let html = &html[start_of_statuscontent_h1 + 19..html.len()];
                let end_of_statuscontent_h1 = html.find("</").unwrap();
                let content_of_statuscontent_h1 = html[0..end_of_statuscontent_h1].trim();
                debug!("content_of_statuscontent_h1: {content_of_statuscontent_h1:?}.");
                if content_of_statuscontent_h1.contains("签到成功") {
                    SignResult::Susses
                } else {
                    SignResult::Fail {
                        msg: content_of_statuscontent_h1.into(),
                    }
                }
            } else {
                SignResult::Fail { msg: html }
            }
        };
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(pre_sign_status)
    }
}

impl RawSign {
    pub fn sign_with_signcode(
        &self,
        session: &Session,
        signcode: &str,
    ) -> Result<SignResult, Box<ureq::Error>> {
        if Self::check_signcode(session, &self.active_id, signcode)? {
            let r = protocol::signcode_sign(session, self.active_id.as_str(), signcode)?;
            Ok(self.guess_sign_result_by_text(&r.into_string().unwrap()))
        } else {
            Ok(SignResult::Fail {
                msg: "签到码或手势不正确".into(),
            })
        }
    }
}

impl RawSign {
    // pub fn speculate_type_by_text(text: &str) -> Sign {
    //     if text.contains("拍照") {
    //         Sign::Photo
    //     } else if text.contains("位置") {
    //         Sign::Location
    //     } else if text.contains("二维码") {
    //         Sign::QrCode
    //     } else if text.contains("手势") {
    //         // ?
    //         Sign::Gesture
    //     } else if text.contains("签到码") {
    //         // ?
    //         Sign::SignCode
    //     } else {
    //         Sign::Normal
    //     }
    // }

    // pub async fn chat_group_pre_sign(
    //     &self,
    //     chat_id: &str,
    //     tuid: &str,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let id = self.活动id.as_str();
    //     let uid = session.get_uid();
    //     let _r = protocol::chat_group_pre_sign(session, id, uid, chat_id, tuid).await?;
    //     Ok(())
    // }

    // pub async fn chat_group_general_sign(
    //     &self,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let r =
    //         protocol::chat_group_general_sign(session, self.活动id.as_str(), session.get_uid())
    //             .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }

    // pub async fn chat_group_signcode_sign(
    //     &self,
    //     session: &Struct签到会话,
    //     signcode: &str,
    // ) -> Result<(), ureq::Error> {
    //     let r = protocol::chat_group_signcode_sign(
    //         session,
    //         self.活动id.as_str(),
    //         session.get_uid(),
    //         signcode,
    //     )
    //     .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }

    // pub async fn chat_group_location_sign(
    //     &self,
    //     address: &Struct位置,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let r = protocol::chat_group_location_sign(
    //         session,
    //         address.get_地址(),
    //         self.活动id.as_str(),
    //         session.get_uid(),
    //         address.get_纬度(),
    //         address.get_经度(),
    //     )
    //     .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }

    // pub async fn chat_group_photo_sign(
    //     &self,
    //     photo: &Struct在线图片,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let r = protocol::chat_group_photo_sign(
    //         session,
    //         self.活动id.as_str(),
    //         session.get_uid(),
    //         photo.get_object_id(),
    //     )
    //     .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }
}
