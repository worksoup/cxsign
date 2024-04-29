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

use cxsign::{
    store::{
        tables::{AccountTable, ExcludeTable},
        DataBase, DataBaseTableTrait,
    },
    Activity, DefaultGestureOrSigncodeSignner, DefaultLocationSignner, DefaultNormalOrRawSignner,
    DefaultPhotoSignner, DefaultQrCodeSignner, RawSign, Session, Sign, SignResult, SignTrait,
    SignnerTrait,
};
use log::{info, warn};
use std::collections::HashMap;

use self::arg::CliArgs;

fn match_signs(
    raw_sign: RawSign,
    db: &DataBase,
    sessions: &[Session],
    cli_args: &CliArgs,
) -> Result<(), Box<cxsign::Error>> {
    let sign_name = raw_sign.name.clone();
    let mut sign = if sessions.is_empty() {
        warn!("无法判断签到[{sign_name}]的签到类型。");
        Sign::Unknown(raw_sign)
    } else {
        info!("成功判断签到[{sign_name}]的签到类型。");
        raw_sign.to_sign(&sessions[0])
    };
    let sign = &mut sign;
    let CliArgs {
        location_str,
        image,
        signcode,
        precisely,
    } = cli_args;
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
            sign_results = DefaultQrCodeSignner::new(db, location_str, image, &None, *precisely)
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
            sign_results = DefaultLocationSignner::new(db, location_str).sign(ls, sessions)?;
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
        println!("签到活动[{}]签到结果：", sign.as_inner().name);
        for (session, sign_result) in sign_results {
            if let SignResult::Fail { msg } = sign_result {
                warn!(
                    "\t用户[{}]签到失败！失败信息：[{:?}]",
                    session.get_stu_name(),
                    msg
                );
            } else {
                println!("\t用户[{}]签到成功！", session.get_stu_name(),);
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
) -> Result<(), Box<cxsign::Error>> {
    let account_table = AccountTable::from_ref(&db);
    let (sessions, has_accounts_arg) = if let Some(accounts_str) = &accounts_str {
        (
            account_table.get_sessions_by_accounts_str(accounts_str),
            true,
        )
    } else {
        (account_table.get_sessions(), false)
    };
    let (valid_signs, other_signs, _) =
        Activity::get_all_activities(ExcludeTable::from_ref(&db), sessions.values(), false)
            .unwrap();
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
        info!("即将处理签到：{sign}");
        let mut names = Vec::new();
        for s in sessions.iter() {
            names.push(s.get_stu_name().to_string())
        }
        info!("签到者：{names:?}");
        match_signs(sign, &db, &sessions, &cli_args).unwrap_or_else(|e| warn!("{e}"));
    }
    Ok(())
}
