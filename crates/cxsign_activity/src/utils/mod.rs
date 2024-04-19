use crate::protocol;
use crate::sign::{CaptchaId, SignResult, SignTrait};
use cxsign_captcha::protocol::CAPTCHA_ID;
use cxsign_types::{Location, LocationWithRange};
use cxsign_user::Session;
use log::{debug, warn};

pub fn secondary_verification(
    agent: &ureq::Agent,
    url: String,
    msg: String,
    captcha_id: &Option<CaptchaId>,
) -> Result<SignResult, Box<ureq::Error>> {
    let enc2 = &msg[9..msg.len()];
    debug!("enc2: {enc2:?}");
    let url = url + "&enc2=" + enc2;
    let captcha_id = if let Some(captcha_id) = captcha_id {
        captcha_id
    } else {
        warn!("未找到滑块 ID, 使用内建值。");
        CAPTCHA_ID
    };
    let url_param = cxsign_captcha::utils::captcha_solver(agent, captcha_id)?.get_validate_info();
    let r = if let Some(url_param) = url_param {
        let url = url + "&validate=" + &url_param;
        let r = protocol::ureq_get(agent, &url)?;
        guess_sign_result_by_text(&r.into_string().unwrap())
    } else {
        SignResult::Fail {
            msg: "滑块验证失败，请重试。".to_string(),
        }
    };
    Ok(r)
}

pub fn guess_sign_result_by_text(text: &str) -> SignResult {
    match text {
        "success" => SignResult::Susses,
        msg => {
            if msg.is_empty() {
                SignResult::Fail { msg:
                "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。".into() }
            } else {
                if msg == "您已签到过了" {
                    SignResult::Susses
                } else {
                    SignResult::Fail { msg: msg.into() }
                }
            }
        }
    }
}

pub(crate) fn sign_unchecked_with_location<T: SignTrait>(
    sign: &T,
    url_getter: impl Fn(&Location) -> String,
    location: &Location,
    preset_location: &Option<LocationWithRange>,
    captcha_id: Option<CaptchaId>,
    session: &Session,
) -> Result<SignResult, Box<ureq::Error>> {
    let mut locations = Vec::new();
    let addr = location.get_addr();
    locations.push(location.clone());
    if let Some(location) = preset_location {
        let mut location = location.to_location();
        if !addr.is_empty() {
            location.set_addr(addr);
        }
        locations.push(location);
    }
    if locations.is_empty() {
        return Ok(SignResult::Fail {
            msg: "没有可供签到的位置！".to_string(),
        });
    }
    for location in locations {
        let url = url_getter(&location);
        let r = protocol::ureq_get(session, &url)?;
        match sign.guess_sign_result_by_text(&r.into_string().unwrap()) {
            SignResult::Susses => return Ok(SignResult::Susses),
            SignResult::Fail { msg } => {
                if msg.starts_with("validate_") {
                    // 这里假设了二次验证只有在“签到成功”的情况下出现。
                    return secondary_verification(session, url, msg, &captcha_id);
                } else if msg.contains("位置") || msg.contains("Location") || msg.contains("范围")
                {
                    continue;
                } else {
                    return Ok(SignResult::Fail { msg });
                }
            }
        };
    }
    warn!("BUG: 请保留现场联系开发者处理。");
    Ok(SignResult::Fail {
        msg: "所有位置均不可用。".to_string(),
    })
}
