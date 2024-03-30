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

fn 区分签到类型并进行签到<'a>(
    签到: RawSign,
    db: &DataBase,
    签到会话列表: &Vec<Session>,
    签到可能使用的信息: &CliArgs,
) -> Result<(), cxsign::Error> {
    let sign_name = 签到.name.clone();
    let mut 签到 = if 签到会话列表.is_empty() {
        warn!("无法判断签到[{sign_name}]的签到类型。");
        Sign::Unknown(签到)
    } else {
        info!("成功判断签到[{sign_name}]的签到类型。");
        签到.to_sign(&签到会话列表[0])
    };
    let 签到 = &mut 签到;
    let CliArgs {
        位置字符串,
        图片或图片路径: pic,
        签到码,
        是否精确识别二维码,
        是否禁用随机偏移,
    } = 签到可能使用的信息;
    let mut 签到结果列表 = HashMap::new();
    let sessions = 签到会话列表.into_iter();
    match 签到 {
        Sign::Photo(ps) => {
            info!("签到[{sign_name}]为拍照签到。");
            签到结果列表 = DefaultPhotoSignner::new(&pic).sign(ps, sessions)?;
        }
        Sign::Normal(ns) => {
            info!("签到[{sign_name}]为普通签到。");
            签到结果列表 = DefaultNormalOrRawSignner.sign(ns, sessions)?;
        }
        Sign::QrCode(qs) => {
            info!("签到[{sign_name}]为二维码签到。");
            签到结果列表 = DefaultQrCodeSignner::new(
                &db,
                &位置字符串,
                &pic,
                &None,
                *是否精确识别二维码,
                *是否禁用随机偏移,
            )
            .sign(qs, sessions)?;
        }
        Sign::Gesture(gs) => {
            info!("签到[{sign_name}]为手势签到。");
            if let Some(signcode) = 签到码 {
                签到结果列表 =
                    DefaultGestureOrSigncodeSignner::new(&signcode).sign(gs, sessions)?;
            } else {
                warn!(
                    "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                    gs.as_inner().name
                )
            }
        }
        Sign::Location(ls) => {
            info!("签到[{sign_name}]为位置签到。");
            签到结果列表 = DefaultLocationSignner::new(&db, &位置字符串, *是否禁用随机偏移)
                .sign(ls, sessions)?;
        }
        Sign::Signcode(ss) => {
            info!("签到[{sign_name}]为签到码签到。");
            if let Some(signcode) = 签到码 {
                签到结果列表 =
                    DefaultGestureOrSigncodeSignner::new(&signcode).sign(ss, sessions)?;
            } else {
                warn!(
                    "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                    ss.as_inner().name
                )
            }
        }
        Sign::Unknown(us) => {
            warn!("签到[{}]为无效签到类型！", us.name);
            签到结果列表 = DefaultNormalOrRawSignner.sign(us, sessions)?;
        }
    }
    if !签到结果列表.is_empty() {
        println!("签到活动[{}]签到结果：", 签到.as_inner().name);
        for (用户真名, 签到结果) in 签到结果列表 {
            if let SignResult::Fail { msg } = 签到结果 {
                warn!(
                    "\t用户[{}]签到失败！失败信息：[{:?}]",
                    用户真名.get_stu_name(),
                    msg
                );
            } else {
                info!("\t用户[{}]签到成功！", 用户真名.get_stu_name(),);
            }
        }
    }
    Ok(())
}

pub fn 签到(
    db: DataBase,
    active_id: Option<i64>,
    账号列表字符串: Option<String>,
    签到可能使用的信息: CliArgs,
) -> Result<(), cxsign::Error> {
    let account_table = AccountTable::from_ref(&db);
    let (sessions, 是否指定accounts参数) = if let Some(账号列表字符串) = &账号列表字符串
    {
        (
            account_table.get_sessions_by_accounts_str(账号列表字符串),
            true,
        )
    } else {
        (account_table.get_sessions(), false)
    };
    let (有效签到列表, 其他签到列表, _) = Activity::get_all_activities(
        ExcludeTable::from_ref(&db),
        sessions.values().into_iter(),
        false,
    )
    .unwrap();
    let signs = if let Some(active_id) = active_id {
        let (签到_需要处理的, 账号对象_签到所需的_vec) = {
            if let Some(s1) = 有效签到列表
                .into_iter()
                .find(|kv| kv.0.as_inner().active_id == active_id.to_string())
            {
                s1
            } else if let Some(s2) = 其他签到列表
                .into_iter()
                .find(|kv| kv.0.as_inner().active_id == active_id.to_string())
            {
                s2
            } else {
                if 是否指定accounts参数 {
                    panic!("没有该签到活动！请检查签到活动 ID 是否正确或所指定的账号是否存在该签到活动！");
                } else {
                    panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
                }
            }
        };
        let mut map = HashMap::new();
        map.insert(签到_需要处理的, 账号对象_签到所需的_vec);
        map
    } else {
        let mut signs = HashMap::new();
        for (sign, 账号对象_签到所需的_vec) in 有效签到列表 {
            signs.insert(sign, 账号对象_签到所需的_vec);
        }
        signs
    };
    if signs.is_empty() {
        warn!("签到列表为空！");
    }
    for (sign, sessions) in signs.into_iter() {
        区分签到类型并进行签到(sign, &db, &sessions, &签到可能使用的信息)
            .unwrap_or_else(|e| warn!("{e}"));
    }
    Ok(())
}
