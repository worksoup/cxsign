pub mod arg;
pub mod location;

use cxsign::{
    store::{tables::AccountTable, DataBase, DataBaseTableTrait},
    DefaultGestureOrSigncodeSignner, DefaultLocationSignner, DefaultNormalOrRawSignner,
    DefaultPhotoSignner, DefaultQrCodeSignner, Session, Sign, SignResult, SignTrait, SignnerTrait,
};
use std::collections::HashMap;

use self::arg::CliArgs;
use crate::tools;

fn 区分签到类型并进行签到<'a>(
    签到: &mut Sign,
    db: &DataBase,
    签到会话列表: Vec<&Session>,
    签到可能使用的信息: &CliArgs,
) -> Result<(), cxsign::Error> {
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
            let signner = DefaultPhotoSignner::new(pic);
            签到结果列表 = signner.sign(ps, sessions)?;
        }
        Sign::Normal(ns) => {
            签到结果列表 = DefaultNormalOrRawSignner.sign(ns, sessions)?;
        }
        Sign::QrCode(qs) => {
            let signner = DefaultQrCodeSignner::new(
                db,
                位置字符串,
                pic,
                &None,
                *是否精确识别二维码,
                *是否禁用随机偏移,
            );
            签到结果列表 = signner.sign(qs, sessions)?;
        }
        Sign::Gesture(gs) => {
            if let Some(signcode) = 签到码 {
                let signner = DefaultGestureOrSigncodeSignner::new(signcode);
                签到结果列表 = signner.sign(gs, sessions)?;
            } else {
                eprintln!(
                    "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                    gs.as_inner().name
                )
            }
        }
        Sign::Location(ls) => {
            let signner = DefaultLocationSignner::new(db, 位置字符串, *是否禁用随机偏移);
            签到结果列表 = signner.sign(ls, sessions)?;
        }
        Sign::Signcode(ss) => {
            if let Some(signcode) = 签到码 {
                let signner = DefaultGestureOrSigncodeSignner::new(signcode);
                签到结果列表 = signner.sign(ss, sessions)?;
            } else {
                eprintln!(
                    "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                    ss.as_inner().name
                )
            }
        }
        Sign::Unknown(us) => {
            eprintln!("签到活动[{}]为无效签到类型！", us.name);
            签到结果列表 = DefaultNormalOrRawSignner.sign(us, sessions)?;
        }
    }
    if !签到结果列表.is_empty() {
        println!("签到活动[{}]签到结果：", 签到.as_inner().name);
        for (用户真名, 签到结果) in 签到结果列表 {
            if let SignResult::Fail { msg } = 签到结果 {
                eprintln!(
                    "\t用户[{}]签到失败！失败信息：[{:?}]",
                    用户真名.get_stu_name(),
                    msg
                );
            } else {
                println!("\t用户[{}]签到成功！", 用户真名.get_stu_name(),);
            }
        }
    }
    Ok(())
}

pub fn 签到(
    db: &DataBase,
    active_id: Option<i64>,
    账号列表字符串: Option<String>,
    签到可能使用的信息: CliArgs,
) -> Result<(), cxsign::Error> {
    let mut 是否指定accounts参数 = false;
    let 数据库完整账号列表 = AccountTable::from_ref(db).get_accounts();
    let 签到所需的账号列表: Vec<&str> = if let Some(账号列表字符串) = &账号列表字符串
    {
        是否指定accounts参数 = true;
        账号列表字符串.split(",").map(|a| a.trim()).collect()
    } else {
        数据库完整账号列表.keys().map(|s| s.as_str()).collect()
    };
    let sessions = tools::通过账号获取签到会话(&db, &签到所需的账号列表);
    let (有效签到列表, 其他签到列表) = tools::获取所有签到(&sessions);
    let signs = if let Some(active_id) = active_id {
        let s1 = 有效签到列表
            .into_iter()
            .find(|kv| kv.0.as_inner().active_id == active_id.to_string());
        let s2 = 其他签到列表
            .into_iter()
            .find(|kv| kv.0.as_inner().active_id == active_id.to_string());
        let (签到_需要处理的, 所有sessions_对应于_签到_需要处理的) = {
            if let Some(s1) = s1 {
                s1
            } else if let Some(s2) = s2 {
                s2
            } else {
                if 是否指定accounts参数 {
                    panic!("没有该签到活动！请检查签到活动 ID 是否正确或所指定的账号是否存在该签到活动！");
                } else {
                    panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
                }
            }
        };
        let mut 账号对象_签到所需的_vec = Vec::new();
        for (uname, session) in 所有sessions_对应于_签到_需要处理的 {
            if 签到所需的账号列表.contains(&uname.as_str()) {
                账号对象_签到所需的_vec.push(session)
            }
        }
        let mut map = HashMap::new();
        map.insert(签到_需要处理的, 账号对象_签到所需的_vec);
        map
    } else {
        let mut signs = HashMap::new();
        for (sign, full_sessions) in 有效签到列表 {
            let mut 账号对象_签到所需的_vec = Vec::new();
            for (uname, session) in &full_sessions {
                if 签到所需的账号列表.contains(&uname.as_str()) {
                    账号对象_签到所需的_vec.push(*session)
                }
            }
            signs.insert(sign, 账号对象_签到所需的_vec);
        }
        signs
    };
    let iter: Vec<(Sign, Vec<&Session>)> = signs.into_iter().collect();
    for (mut sign, sessions) in iter {
        区分签到类型并进行签到(&mut sign, db, sessions, &签到可能使用的信息)?;
    }
    Ok(())
}
