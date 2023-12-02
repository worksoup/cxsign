pub mod arg;
mod sign;
mod single_sign;

use crate::activity::sign::{SignActivity, SignState, SignType};
use crate::utils::sign::get_refresh_qrcode_sign_params_on_screen;
use crate::{
    session::SignSession,
    utils::{address::Address, sql::DataBase},
};
use std::fs::DirEntry;
use std::{collections::HashMap, path::PathBuf};

pub fn picdir_to_pic(picdir: &PathBuf) -> Option<PathBuf> {
    loop {
        let ans = crate::utils::inquire_confirm("二维码图片是否就绪？","本程序会读取 `--pic` 参数所指定的路径下最新修改的图片。你可以趁现在获取这张图片，然后按下回车进行签到。",);
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
    sign: &SignActivity,
    db: &DataBase,
    location: Option<i64>,
    pos: &Option<String>,
) -> Vec<Address> {
    if let Some(ref pos) = pos {
        vec![Address::parse_str(&pos).unwrap_or_else(|e| panic!("{}", e))]
    } else if let Some(addr) = location {
        let poss = db.get_pos(addr);
        vec![poss.1]
    } else {
        let mut poss = db.get_course_poss_without_posid(sign.course.get_id());
        let mut other = db.get_course_poss_without_posid(-1);
        poss.append(&mut other);
        poss
    }
}
async fn handle_account_sign<'a>(
    sign: &SignActivity,
    pic: &Option<PathBuf>,
    location: Option<i64>,
    db: &DataBase,
    pos: &Option<String>,
    signcode: &Option<String>,
    sessions: &'a Vec<&SignSession>,
    capture: bool,
) -> Result<(), reqwest::Error> {
    let sign_type = sign.get_sign_type();
    let mut states = HashMap::new();

    match sign_type {
        SignType::Photo => {
            if let Some(pic) = pic {
                if let Ok(metadata) = std::fs::metadata(pic) {
                    let pic = if metadata.is_dir() {
                        picdir_to_pic(pic)
                    } else {
                        Some(pic.to_owned())
                    };
                    states = sign::photo_sign_(sign, &pic, sessions).await?;
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
        SignType::Common => {
            states = sign::general_sign_(sign, sessions).await?;
        }
        SignType::QrCode => {
            let poss = location_and_pos_to_poss(sign, db, location, pos).await;
            fn print_err_msg(sign: &SignActivity) {
                eprintln!(
                    "所有用户在二维码签到[{}]中签到失败！二维码签到需要提供签到二维码！",
                    sign.name
                );
            }
            async fn try_by_pic_arg<'a>(
                sign: &SignActivity,
                pic: &Option<PathBuf>,
                location: Option<i64>,
                db: &DataBase,
                pos: &Option<String>,
                sessions: &'a Vec<&SignSession>,
            ) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
                let poss = location_and_pos_to_poss(sign, db, location, pos).await;
                let mut states = HashMap::new();
                if let Some(pic) = pic {
                    if std::fs::metadata(pic).unwrap().is_dir() {
                        if let Some(pic) = picdir_to_pic(pic) {
                            let enc =
                                crate::utils::sign::handle_qrcode_pic_path(pic.to_str().unwrap());
                            states = sign::qrcode_sign_(
                                sign,
                                sign.get_c_of_qrcode_sign(),
                                &enc,
                                &poss,
                                sessions,
                            )
                            .await?;
                        } else {
                            print_err_msg(sign);
                        }
                    } else {
                        let enc = crate::utils::sign::handle_qrcode_pic_path(pic.to_str().unwrap());
                        states = sign::qrcode_sign_(
                            sign,
                            sign.get_c_of_qrcode_sign(),
                            &enc,
                            &poss,
                            sessions,
                        )
                        .await?;
                    }
                } else {
                    print_err_msg(sign);
                };
                Ok(states)
            }

            if capture {
                if let Some(enc) =
                    get_refresh_qrcode_sign_params_on_screen(sign.is_refresh_qrcode())
                {
                    states = sign::qrcode_sign_(
                        sign,
                        sign.get_c_of_qrcode_sign(),
                        &enc,
                        &poss,
                        sessions,
                    )
                    .await?;
                } else {
                    states = try_by_pic_arg(sign, pic, location, db, pos, sessions).await?;
                }
            } else {
                states = try_by_pic_arg(sign, pic, location, db, pos, sessions).await?;
            }
        }
        SignType::Location => {
            let poss = location_and_pos_to_poss(sign, db, location, pos).await;
            states = sign::location_sign_(sign, &poss, sessions).await?;
        }
        SignType::Unknown => {
            eprintln!("签到活动[{}]为无效签到类型！", sign.name);
        }
        signcode_sign_type => {
            if let Some(signcode) = signcode {
                states = sign::signcode_sign_(sign, signcode, sessions).await?;
            } else {
                let sign_type_str = match signcode_sign_type {
                    SignType::Gesture => "手势",
                    SignType::SignCode => "签到码",
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

pub async fn sign(
    db: &DataBase,
    sessions: &HashMap<String, SignSession>,
    asigns: HashMap<SignActivity, Vec<&SignSession>>,
    osigns: HashMap<SignActivity, Vec<&SignSession>>,
    activity: Option<i64>,
    account: Option<String>,
    location: Option<i64>,
    pos: Option<String>,
    pic: Option<PathBuf>,
    signcode: Option<String>,
    capture: bool,
) -> Result<(), reqwest::Error> {
    if let Some(active_id) = activity {
        let s1 = asigns.iter().find(|kv| kv.0.id == active_id.to_string());
        let s2 = osigns.iter().find(|kv| kv.0.id == active_id.to_string());
        let (sign, mut full_sessions) = {
            if let Some(s1) = s1 {
                s1
            } else if let Some(s2) = s2 {
                s2
            } else {
                panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
            }
        };
        let mut accounts = Vec::new();
        if let Some(ref account) = &account {
            accounts.push(&sessions[account]);
            full_sessions = &accounts;
        }
        handle_account_sign(
            sign,
            &pic,
            location,
            db,
            &pos,
            &signcode,
            full_sessions,
            capture,
        )
        .await?;
    } else {
        for (sign, mut full_sessions) in &asigns {
            let mut accounts = Vec::new();
            if let Some(ref account) = &account {
                accounts.push(&sessions[account]);
                full_sessions = &accounts;
            }
            handle_account_sign(
                sign,
                &pic,
                location,
                db,
                &pos,
                &signcode,
                full_sessions,
                capture,
            )
            .await?;
        }
    }
    Ok(())
}
