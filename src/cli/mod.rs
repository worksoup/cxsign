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
pub mod location;

use self::arg::CliArgs;
use cxlib::{
    activity::{Activity, RawSign},
    default_impl::{
        sign::Sign,
        signner::{
            DefaultGestureOrSigncodeSignner, DefaultLocationInfoGetter, DefaultLocationSignner,
            DefaultNormalOrRawSignner, DefaultPhotoSignner, DefaultQrCodeSignner,
        },
        store::{AccountTable, DataBase},
    },
    error::Error,
    sign::{SignResult, SignTrait, SignnerTrait},
    user::Session,
};
use log::{info, warn};
use std::collections::HashMap;

fn match_signs(
    raw_sign: RawSign,
    db: &DataBase,
    sessions: &[Session],
    cli_args: &CliArgs,
) -> Result<(), Error> {
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
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
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
                DefaultLocationInfoGetter::from(db),
                location_str,
                image,
                &None,
                #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
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
            sign_results =
                DefaultLocationSignner::new(DefaultLocationInfoGetter::from(db), location_str)
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
        info!("签到活动[{}]签到结果：", sign.as_raw().name);
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
    uid_list_str: Option<String>,
    cli_args: CliArgs,
) -> Result<(), Error> {
    let (sessions, has_uid_arg) = if let Some(uid_list_str) = &uid_list_str {
        (
            AccountTable::get_sessions_by_uid_list_str(&db, uid_list_str),
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
            } else if has_uid_arg {
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
            chrono::DateTime::from_timestamp_millis(sign.start_time_mills as i64)
                .unwrap()
                .naive_local()
                .to_string(),
            sign.course.get_class_id(),
            sign.course.get_id(),
            sign.course.get_name()
        );
        let mut names = Vec::new();
        for s in sessions.iter() {
            names.push(s.get_stu_name().to_string())
        }
        info!("签到者：{names:?}");
        match_signs(sign, &db, &sessions, &cli_args).unwrap_or_else(|e| warn!("{e}"));
    }
    Ok(())
}
