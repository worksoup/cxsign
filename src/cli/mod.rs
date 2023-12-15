pub mod arg;
mod sign;
mod single_sign;
pub mod pos;

use crate::activity::sign::{Enum签到类型, Enum签到结果, Struct签到};
use crate::utils;
use crate::utils::sign::截屏获取二维码签到所需参数;
use crate::{
    session::Struct签到会话,
    utils::{address::Struct位置, sql::DataBase},
};
use std::fs::DirEntry;
use std::{collections::HashMap, path::PathBuf};

use self::arg::CliArgs;

pub fn 通过目录决定图片路径(图片所在目录: &PathBuf) -> Option<PathBuf> {
    loop {
        let 答案 = utils::请求确认("二维码图片是否就绪？","本程序会读取 `--pic` 参数所指定的路径下最新修改的图片。你可以趁现在获取这张图片，然后按下回车进行签到。",);
        if 答案 {
            break;
        }
    }
    let 图片路径 = if let Ok(图片所在目录) = std::fs::read_dir(图片所在目录) {
        let mut 目录下所有文件: Vec<DirEntry> = 图片所在目录
            .filter_map(|k| {
                let r = k.as_ref().is_ok_and(|k| {
                    k.file_type().is_ok_and(|t| {
                        t.is_file() && {
                            let 文件名 = k.file_name();
                            let 文件后缀名 = 文件名.to_str().unwrap().split('.').last().unwrap();
                            文件后缀名 == "png" || 文件后缀名 == "jpg"
                        }
                    })
                });
                if r {
                    Some(unsafe { k.unwrap_unchecked() })
                } else {
                    None
                }
            })
            .collect();
        if 目录下所有文件.is_empty() {
            eprintln!("文件夹下没有图片！（只支持 `*.png` 文件或 `*.jpg` 文件。）");
            None
        } else {
            目录下所有文件.sort_by(|a, b| {
                b.metadata()
                    .unwrap()
                    .modified()
                    .unwrap()
                    .cmp(&a.metadata().unwrap().modified().unwrap())
            });
            Some(目录下所有文件[0].path())
        }
    } else {
        eprintln!("遍历文件夹失败！");
        None
    };
    图片路径
}
async fn 通过位置id和位置字符串决定位置(
    db: &DataBase,
    位置id: &Option<i64>,
    位置字符串: &Option<String>,
) -> Option<Struct位置> {
    if let Some(ref 位置字符串) = 位置字符串 {
        Some(Struct位置::从字符串解析(&位置字符串).unwrap_or_else(|e| panic!("{}", e)))
    } else if let Some(位置id) = 位置id {
        let (_, 位置) = db.get_pos(*位置id);
        Some(位置)
    } else {
        None
    }
}

fn 打印对于sign无法获取二维码时的错误信息(sign: &Struct签到) {
    eprintln!(
        "所有用户在二维码签到[{}]中签到失败！二维码签到需要提供签到二维码！",
        sign.签到名
    );
}

async fn qrcode_sign_by_pic_arg<'a>(
    签到: &Struct签到,
    pic: &PathBuf,
    location: &Option<i64>,
    db: &DataBase,
    pos: &Option<String>,
    sessions: &'a Vec<&Struct签到会话>,
) -> Result<HashMap<&'a str, Enum签到结果>, reqwest::Error> {
    let pos_vec =
        if let Some(pos) = 通过位置id和位置字符串决定位置(db, &location, pos).await {
            vec![pos]
        } else {
            let mut positions = db.get_course_positions(签到.课程.get_课程号());
            let mut other = db.get_course_positions(-1);
            positions.append(&mut other);
            positions
        };
    let mut states = HashMap::new();
    if std::fs::metadata(pic).unwrap().is_dir() {
        if let Some(pic) = 通过目录决定图片路径(pic) {
            let enc = utils::sign::扫描路径中二维码并获取签到所需参数(
                pic.to_str().unwrap(),
            );
            states = sign::二维码签到(
                签到,
                签到.get_二维码签到时的c参数(),
                &enc,
                &pos_vec,
                sessions,
            )
            .await?;
        } else {
            打印对于sign无法获取二维码时的错误信息(签到);
        }
    } else {
        let enc =
            utils::sign::扫描路径中二维码并获取签到所需参数(pic.to_str().unwrap());
        states = sign::二维码签到(
            签到,
            签到.get_二维码签到时的c参数(),
            &enc,
            &pos_vec,
            sessions,
        )
        .await?;
    }
    Ok(states)
}

async fn 区分签到类型并进行签到<'a>(
    sign: &Struct签到,
    db: &DataBase,
    sessions: &'a Vec<&Struct签到会话>,
    cli_args: &CliArgs,
) -> Result<(), reqwest::Error> {
    let CliArgs {
        位置id: location,
        位置字符串: pos,
        图片或图片路径: pic,
        签到码: signcode,
        // capture,
        是否精确识别二维码: precise,
        是否禁用随机偏移: no_random_shift,
    } = cli_args;
    let sign_type = sign.get_sign_type();
    let mut 签到结果列表 = HashMap::new();

    match sign_type {
        Enum签到类型::拍照签到 => {
            let pic = if let Some(pic) = pic
                && let Ok(metadata) = std::fs::metadata(pic)
            {
                if metadata.is_dir() {
                    通过目录决定图片路径(pic)
                } else {
                    Some(pic.to_owned())
                }
            } else {
                None
            };
            签到结果列表 = sign::拍照签到(sign, &pic, sessions).await?;
        }
        Enum签到类型::普通签到 => {
            签到结果列表 = sign::普通签到(sign, sessions).await?;
        }
        Enum签到类型::二维码签到 => {
            let pos_vec = if let Some(pos) =
                通过位置id和位置字符串决定位置(db, location, pos).await
            {
                vec![pos]
            } else {
                let mut pos_vec = db.get_course_positions(sign.课程.get_课程号());
                let mut other = db.get_course_positions(-1);
                pos_vec.append(&mut other);
                pos_vec
            };
            //  如果有 pic 参数，那么使用它。
            if let Some(pic) = pic {
                签到结果列表 =
                    qrcode_sign_by_pic_arg(sign, pic, location, db, pos, sessions).await?;
            }
            // 如果没有则试图截屏。
            else if let Some(enc) =
                截屏获取二维码签到所需参数(sign.二维码是否刷新(), *precise)
            {
                签到结果列表 = sign::二维码签到(
                    sign,
                    sign.get_二维码签到时的c参数(),
                    &enc,
                    &pos_vec,
                    sessions,
                )
                .await?;
            }
            // 这下是真没有了。
            else {
                打印对于sign无法获取二维码时的错误信息(sign);
            }
        }
        Enum签到类型::位置签到 => {
            if let Some(pos) = 通过位置id和位置字符串决定位置(db, location, pos).await
            {
                签到结果列表 =
                    sign::位置签到(sign, &vec![pos], false, sessions, *no_random_shift).await?;
            } else {
                let mut poss = db.get_course_positions(sign.课程.get_课程号());
                let mut other = db.get_course_positions(-1);
                poss.append(&mut other);
                签到结果列表 =
                    sign::位置签到(sign, &poss, true, sessions, *no_random_shift).await?;
            };
        }
        Enum签到类型::非已知签到 => {
            eprintln!("签到活动[{}]为无效签到类型！", sign.签到名);
        }
        signcode_sign_type => {
            if let Some(signcode) = signcode {
                签到结果列表 = sign::签到码签到(sign, signcode, sessions).await?;
            } else {
                let sign_type_str = match signcode_sign_type {
                    Enum签到类型::手势签到 => "手势",
                    Enum签到类型::签到码签到 => "签到码",
                    _ => unreachable!(),
                };
                eprintln!(
                    "所有用户在{sign_type_str}签到[{}]中签到失败！需要提供签到码！",
                    sign.签到名
                )
            }
        }
    };
    if !签到结果列表.is_empty() {
        println!("签到活动[{}]签到结果：", sign.签到名);
        for (用户真名, 签到结果) in 签到结果列表 {
            if let Enum签到结果::失败 { 失败信息 } = 签到结果 {
                eprintln!("\t用户[{}]签到失败！失败信息：[{:?}]", 用户真名, 失败信息);
            } else {
                println!("\t用户[{}]签到成功！", 用户真名,);
            }
        }
    }
    Ok(())
}

pub async fn 签到(
    db: &DataBase,
    active_id: Option<i64>,
    账号列表字符串: Option<String>,
    签到可能使用的信息: CliArgs,
) -> Result<(), reqwest::Error> {
    let mut 是否指定accounts参数 = false;
    let 数据库完整账号列表 = db.get_accounts();
    let 签到所需的账号列表: Vec<&str> = if let Some(账号列表字符串) = &账号列表字符串
    {
        是否指定accounts参数 = true;
        账号列表字符串.split(",").map(|a| a.trim()).collect()
    } else {
        数据库完整账号列表.keys().map(|s| s.as_str()).collect()
    };
    let sessions = utils::account::通过账号获取签到会话(&db, &签到所需的账号列表).await;
    let (有效签到列表, 其他签到列表) = utils::sign::获取所有签到(&sessions).await;
    let signs = if let Some(active_id) = active_id {
        let s1 = 有效签到列表
            .iter()
            .find(|kv| kv.0.活动id == active_id.to_string());
        let s2 = 其他签到列表
            .iter()
            .find(|kv| kv.0.活动id == active_id.to_string());
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
                账号对象_签到所需的_vec.push(*session)
            }
        }
        let mut map = HashMap::new();
        map.insert(签到_需要处理的, 账号对象_签到所需的_vec);
        map
    } else {
        let mut signs = HashMap::new();
        for (sign, full_sessions) in &有效签到列表 {
            let mut 账号对象_签到所需的_vec = Vec::new();
            for (uname, session) in full_sessions {
                if 签到所需的账号列表.contains(&uname.as_str()) {
                    账号对象_签到所需的_vec.push(*session)
                }
            }
            signs.insert(sign, 账号对象_签到所需的_vec);
        }
        signs
    };

    for (sign, sessions) in signs {
        区分签到类型并进行签到(sign, db, &sessions, &签到可能使用的信息).await?;
    }
    Ok(())
}