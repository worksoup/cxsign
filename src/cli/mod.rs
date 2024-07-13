// Copyright (C) 2024 worksoup <https://github.com/worksoup/>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod arg;

use cxsign::{
    activity::{Activity, RawSign},
    default_impl::{
        sign::{LocationSign, Sign},
        signner::{
            DefaultGestureOrSigncodeSignner, DefaultLocationSignner, DefaultNormalOrRawSignner,
            DefaultPhotoSignner, DefaultQrCodeSignner, LocationInfoGetterTrait,
        },
        store::{AccountTable, DataBase},
    },
    error::Error,
    sign::{SignResult, SignTrait},
    signner::SignnerTrait,
    types::Location,
    user::Session,
};
use log::{info, warn};
use std::collections::HashMap;
use xdsign_data::LOCATIONS;

use self::arg::CliArgs;

pub struct XdsignLocationInfoGetter;

impl LocationInfoGetterTrait for XdsignLocationInfoGetter {
    fn map_location_str(&self, location_str: &str) -> Option<Location> {
        let location_str = location_str.trim();
        location_str
            .parse()
            .ok()
            .or_else(|| LOCATIONS.get(location_str).cloned())
    }
    fn get_fallback_location(&self, _: &LocationSign) -> Option<Location> {
        LOCATIONS.values().next().cloned()
    }
}

fn match_signs(
    raw_sign: RawSign,
    sessions: &[Session],
    cli_args: &CliArgs,
) -> Result<(), Box<Error>> {
    let sign_name = raw_sign.name.clone();
    let mut sign = if sessions.is_empty() {
        warn!("无法判断签到[{sign_name}]的签到类型。");
        Sign::Unknown(raw_sign)
    } else {
        info!("成功判断签到[{sign_name}]的签到类型。");
        Sign::from_raw(raw_sign, &sessions[0])
    };
    let sign = &mut sign;
    let CliArgs {
        location_str,
        image,
        signcode,
        precisely,
    } = cli_args;
    #[allow(clippy::mutable_key_type)]
    let mut sign_results = HashMap::new();
    let sessions = sessions.iter();
    match sign {
        Sign::Photo(ps) => {
            info!("签到[{sign_name}]为拍照签到。");
            sign_results = DefaultPhotoSignner::new(image).sign(ps, sessions)?;
        }
        Sign::Normal(ns) => {
            info!("签到[{sign_name}]为普通签到。");
            sign_results = DefaultNormalOrRawSignner.sign(ns, sessions)?;
        }
        Sign::QrCode(qs) => {
            info!("签到[{sign_name}]为二维码签到。");
            sign_results = DefaultQrCodeSignner::new(
                XdsignLocationInfoGetter,
                location_str,
                image,
                &None,
                *precisely,
            )
            .sign(qs, sessions)?;
        }
        Sign::Gesture(gs) => {
            info!("签到[{sign_name}]为手势签到。");
            if let Some(signcode) = signcode {
                sign_results = DefaultGestureOrSigncodeSignner::new(signcode).sign(gs, sessions)?;
            } else {
                warn!(
                    "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                    gs.as_inner().name
                )
            }
        }
        Sign::Location(ls) => {
            info!("签到[{sign_name}]为位置签到。");
            sign_results = DefaultLocationSignner::new(XdsignLocationInfoGetter, location_str)
                .sign(ls, sessions)?;
        }
        Sign::Signcode(ss) => {
            info!("签到[{sign_name}]为签到码签到。");
            if let Some(signcode) = signcode {
                sign_results = DefaultGestureOrSigncodeSignner::new(signcode).sign(ss, sessions)?;
            } else {
                warn!(
                    "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                    ss.as_inner().name
                )
            }
        }
        Sign::Unknown(us) => {
            warn!("签到[{}]为无效签到类型！", us.name);
            sign_results = DefaultNormalOrRawSignner.sign(us, sessions)?;
        }
    }
    if !sign_results.is_empty() {
        info!("签到活动[{}]签到结果：", sign.as_inner().name);
        for (session, sign_result) in sign_results {
            if let SignResult::Fail { msg } = sign_result {
                warn!(
                    "\t用户[{}]签到失败！失败信息：[{:?}]",
                    session.get_stu_name(),
                    msg
                );
            } else {
                info!("\t用户[{}]签到成功！", session.get_stu_name(),);
            }
        }
    }
    Ok(())
}

pub fn do_sign(
    db: DataBase,
    active_id: Option<i64>,
    accounts_str: Option<String>,
    cli_args: CliArgs,
) -> Result<(), Error> {
    let (sessions, has_accounts_arg) = if let Some(accounts_str) = &accounts_str {
        (
            AccountTable::get_sessions_by_accounts_str(&db, accounts_str),
            true,
        )
    } else {
        (AccountTable::get_sessions(&db), false)
    };
    let activities =
        Activity::get_all_activities(&db, sessions.values(), false).map_err(Error::from)?;
    let (valid_signs, other_signs): (
        HashMap<RawSign, Vec<Session>>,
        HashMap<RawSign, Vec<Session>>,
    ) = activities
        .into_iter()
        .filter_map(|(k, v)| match k {
            Activity::RawSign(k) => Some((k, v)),
            Activity::Other(_) => None,
        })
        .partition(|(k, _)| k.is_valid());
    let signs = if let Some(active_id) = active_id {
        let (sign, sessions) = {
            if let Some(s1) = valid_signs
                .into_iter()
                .find(|kv| kv.0.as_inner().active_id == active_id.to_string())
            {
                s1
            } else if let Some(s2) = other_signs
                .into_iter()
                .find(|kv| kv.0.as_inner().active_id == active_id.to_string())
            {
                s2
            } else if has_accounts_arg {
                panic!(
                    "没有该签到活动！请检查签到活动 ID 是否正确或所指定的账号是否存在该签到活动！"
                );
            } else {
                panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
            }
        };
        let mut map = HashMap::new();
        map.insert(sign, sessions);
        map
    } else {
        let mut signs = HashMap::new();
        for (sign, sessions) in valid_signs {
            signs.insert(sign, sessions);
        }
        signs
    };
    if signs.is_empty() {
        warn!("签到列表为空。");
    }
    for (sign, sessions) in signs {
        info!(
            "即将处理签到：[{}], id 为 {}, 开始时间为 {}, 课程为 {} / {} / {}",
            sign.name,
            sign.active_id,
            cxsign::utils::time_string_from_mills(sign.start_time_mills),
            sign.course.get_class_id(),
            sign.course.get_id(),
            sign.course.get_name()
        );
        let mut names = Vec::new();
        for s in sessions.iter() {
            names.push(s.get_stu_name().to_string())
        }
        info!("签到者：{names:?}");
        match_signs(sign, &sessions, &cli_args).unwrap_or_else(|e| warn!("{e}"));
    }
    Ok(())
}
