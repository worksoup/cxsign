pub mod arg;
mod sign;
mod single_sign;

use crate::activity::sign::{Struct签到, SignState, Enum签到类型};
use crate::utils;
use crate::utils::sign::截屏获取二维码签到所需参数;
use crate::{
    session::SignSession,
    utils::{address::Struct位置, sql::DataBase},
};
use std::fs::DirEntry;
use std::{collections::HashMap, path::PathBuf};

use self::arg::CliArgs;

pub fn picdir_to_pic(picdir: &PathBuf) -> Option<PathBuf> {
    loop {
        let ans = utils::inquire_confirm("二维码图片是否就绪？","本程序会读取 `--pic` 参数所指定的路径下最新修改的图片。你可以趁现在获取这张图片，然后按下回车进行签到。",);
        if ans {
            break;
        }
    }
    let pic = if let Ok(pic_dir) = std::fs::read_dir(picdir) {
        let mut files: Vec<DirEntry> = pic_dir
            .filter_map(|k| {
                let r = k.as_ref().is_ok_and(|k| {
                    k.file_type().is_ok_and(|t| {
                        t.is_file() && {
                            let file_name = k.file_name();
                            let ext = file_name.to_str().unwrap().split('.').last().unwrap();
                            ext == "png" || ext == "jpg"
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
        if files.is_empty() {
            eprintln!("文件夹下没有图片！（只支持 `*.png` 文件或 `*.jpg` 文件。）");
            None
        } else {
            files.sort_by(|a, b| {
                b.metadata()
                    .unwrap()
                    .modified()
                    .unwrap()
                    .cmp(&a.metadata().unwrap().modified().unwrap())
            });
            Some(files[0].path())
        }
    } else {
        eprintln!("遍历文件夹失败！");
        None
    };
    pic
}
async fn location_and_pos_to_poss(
    db: &DataBase,
    location: &Option<i64>,
    pos: &Option<String>,
) -> Option<Struct位置> {
    if let Some(ref pos) = pos {
        Some(Struct位置::parse_str(&pos).unwrap_or_else(|e| panic!("{}", e)))
    } else if let Some(addr) = location {
        let poss = db.get_pos(*addr);
        Some(poss.1)
    } else {
        None
    }
}

fn 打印对于sign无法获取二维码时的错误信息(sign: &Struct签到) {
    eprintln!(
        "所有用户在二维码签到[{}]中签到失败！二维码签到需要提供签到二维码！",
        sign.name
    );
}

async fn qrcode_sign_by_pic_arg<'a>(
    sign: &Struct签到,
    pic: &PathBuf,
    location: &Option<i64>,
    db: &DataBase,
    pos: &Option<String>,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let pos_vec = if let Some(pos) = location_and_pos_to_poss(db, &location, pos).await {
        vec![pos]
    } else {
        let mut poss = db.get_course_poss_without_posid(sign.course.get_course_id());
        let mut other = db.get_course_poss_without_posid(-1);
        poss.append(&mut other);
        poss
    };
    let mut states = HashMap::new();
    if std::fs::metadata(pic).unwrap().is_dir() {
        if let Some(pic) = picdir_to_pic(pic) {
            let enc = utils::sign::handle_qrcode_pic_path(pic.to_str().unwrap());
            states = sign::二维码签到(sign, sign.get_c_of_qrcode_sign(), &enc, &pos_vec, sessions)
                .await?;
        } else {
            打印对于sign无法获取二维码时的错误信息(sign);
        }
    } else {
        let enc = utils::sign::handle_qrcode_pic_path(pic.to_str().unwrap());
        states =
            sign::二维码签到(sign, sign.get_c_of_qrcode_sign(), &enc, &pos_vec, sessions).await?;
    }
    Ok(states)
}
async fn 区分签到类型并进行签到<'a>(
    sign: &Struct签到,
    db: &DataBase,
    sessions: &'a Vec<&SignSession>,
    cli_args: &CliArgs,
) -> Result<(), reqwest::Error> {
    let CliArgs {
        location,
        pos,
        pic,
        signcode,
        // capture,
        precise,
        no_random_shift,
    } = cli_args;
    let sign_type = sign.get_sign_type();
    let mut states = HashMap::new();

    match sign_type {
        Enum签到类型::拍照签到 => {
            if let Some(pic) = pic {
                if let Ok(metadata) = std::fs::metadata(pic) {
                    let pic = if metadata.is_dir() {
                        picdir_to_pic(pic)
                    } else {
                        Some(pic.to_owned())
                    };
                    states = sign::拍照签到(sign, &pic, sessions).await?;
                } else {
                    eprintln!(
                        "所有用户在拍照签到[{}]中签到失败！未能获取{:?}的元信息！",
                        sign.name, pic
                    );
                };
            } else {
                eprintln!(
                    "所有用户在拍照签到[{}]中签到失败！未提供照片路径！",
                    sign.name
                )
            };
        }
        Enum签到类型::普通签到 => {
            states = sign::普通签到(sign, sessions).await?;
        }
        Enum签到类型::二维码签到 => {
            let pos_vec = if let Some(pos) = location_and_pos_to_poss(db, location, pos).await {
                vec![pos]
            } else {
                let mut pos_vec = db.get_course_poss_without_posid(sign.course.get_course_id());
                let mut other = db.get_course_poss_without_posid(-1);
                pos_vec.append(&mut other);
                pos_vec
            };
            //  如果有 pic 参数，那么使用它。
            if let Some(pic) = pic {
                states = qrcode_sign_by_pic_arg(sign, pic, location, db, pos, sessions).await?;
            } 
            // 如果没有则试图截屏。
            else if let Some(enc) =
                截屏获取二维码签到所需参数(sign.is_refresh_qrcode(), *precise)
            {
                states =
                    sign::二维码签到(sign, sign.get_c_of_qrcode_sign(), &enc, &pos_vec, sessions)
                        .await?;
            }
            // 这下是真没有了。
             else {
                打印对于sign无法获取二维码时的错误信息(sign);
            }
        }
        Enum签到类型::位置签到 => {
            if let Some(pos) = location_and_pos_to_poss(db, location, pos).await {
                states = sign::位置签到(sign, &vec![pos], false, sessions, *no_random_shift)
                    .await?;
            } else {
                let mut poss = db.get_course_poss_without_posid(sign.course.get_course_id());
                let mut other = db.get_course_poss_without_posid(-1);
                poss.append(&mut other);
                states =
                    sign::位置签到(sign, &poss, true, sessions, *no_random_shift).await?;
            };
        }
        Enum签到类型::非已知签到 => {
            eprintln!("签到活动[{}]为无效签到类型！", sign.name);
        }
        signcode_sign_type => {
            if let Some(signcode) = signcode {
                states = sign::签到码签到(sign, signcode, sessions).await?;
            } else {
                let sign_type_str = match signcode_sign_type {
                    Enum签到类型::手势签到 => "手势",
                    Enum签到类型::签到码签到 => "签到码",
                    _ => unreachable!(),
                };
                eprintln!(
                    "所有用户在{sign_type_str}签到[{}]中签到失败！需要提供签到码！",
                    sign.name
                )
            }
        }
    };
    if !states.is_empty() {
        println!("签到活动[{}]签到结果：", sign.name);
        for (uname, state) in states {
            if let SignState::Fail(msg) = state {
                eprintln!("\t用户[{}]签到失败！失败信息：[{:?}]", uname, msg);
            } else {
                println!("\t用户[{}]签到成功！", uname,);
            }
        }
    }
    Ok(())
}

pub async fn 签到(
    db: &DataBase,
    activity: Option<i64>,
    accounts: Option<String>,
    cli_args: CliArgs,
) -> Result<(), reqwest::Error> {
    let mut 是否指定了accounts参数 = false;
    let 用户名_从数据库中获取的所有 = db.get_accounts();
    let 用户名_签到所需的: Vec<&str> = if let Some(account) = &accounts {
        是否指定了accounts参数 = true;
        account.split(",").map(|a| a.trim()).collect()
    } else {
        用户名_从数据库中获取的所有.keys().map(|s| s.as_str()).collect()
    };
    let sessions = utils::account::get_sessions_by_unames(&db, &用户名_签到所需的).await;
    let (有效签到, 其他签到) = utils::sign::get_signs(&sessions).await;
    let signs = if let Some(active_id) = activity {
        let s1 = 有效签到.iter().find(|kv| kv.0.id == active_id.to_string());
        let s2 = 其他签到.iter().find(|kv| kv.0.id == active_id.to_string());
        let (签到_需要处理的, 所有sessions_对应于_签到_需要处理的) = {
            if let Some(s1) = s1 {
                s1
            } else if let Some(s2) = s2 {
                s2
            } else {
                if 是否指定了accounts参数 {
                    panic!("没有该签到活动！请检查签到活动 ID 是否正确或所指定的账号是否存在该签到活动！");
                } else {
                    panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
                }
            }
        };
        let mut 账号对象_签到所需的_vec = Vec::new();
        for (uname, session) in 所有sessions_对应于_签到_需要处理的 {
            if 用户名_签到所需的.contains(&uname.as_str()) {
                账号对象_签到所需的_vec.push(*session)
            }
        }
        let mut map = HashMap::new();
        map.insert(签到_需要处理的, 账号对象_签到所需的_vec);
        map
    } else {
        let mut signs = HashMap::new();
        for (sign, full_sessions) in &有效签到 {
            let mut 账号对象_签到所需的_vec = Vec::new();
            for (uname, session) in full_sessions {
                if 用户名_签到所需的.contains(&uname.as_str()) {
                    账号对象_签到所需的_vec.push(*session)
                }
            }
            signs.insert(sign, 账号对象_签到所需的_vec);
        }
        signs
    };

    for (sign, sessions) in signs {
        区分签到类型并进行签到(sign, db, &sessions, &cli_args).await?;
    }
    Ok(())
}
