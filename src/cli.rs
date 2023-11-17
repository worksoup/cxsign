use crate::{
    sign_session::{
        activity::sign::{SignActivity, SignState, SignType},
        session::SignSession,
    },
    utils::{address::Address, photo::Photo, sql::DataBase},
};
use std::{
    collections::{hash_map::OccupiedError, HashMap},
    path::PathBuf,
};

// 添加账号。
pub async fn add_account(db: &DataBase, uname: String, pwd: Option<String>) {
    let pwd = if let Some(pwd) = pwd {
        pwd
    } else {
        inquire::Password::new("密码：")
            .without_confirmation()
            .prompt()
            .unwrap()
    };
    let enc_pwd = crate::utils::pwd_des(&pwd);
    let session = SignSession::login_enc(&uname, &enc_pwd).await.unwrap();
    let name = session.get_stu_name();
    db.add_account_or(&uname, &enc_pwd, name, DataBase::update_account);
    let courses = session.get_courses().await.unwrap();
    for c in courses {
        db.add_course_or(&c, |_, _| {});
    }
}

pub async fn get_sessions(db: &DataBase) -> HashMap<String, SignSession> {
    let accounts = db.get_accounts();
    let config_dir = crate::utils::CONFIG_DIR.clone();
    let mut s = HashMap::new();
    for a in accounts {
        let cookies_dir = config_dir.join(a.0.to_string() + ".json");
        let session = SignSession::load(cookies_dir).await.unwrap();
        s.insert(a.0, session);
    }
    s
}

pub async fn get_signs<'a>(
    sessions: &'a HashMap<String, SignSession>,
) -> (
    HashMap<SignActivity, Vec<&'a SignSession>>,
    HashMap<SignActivity, Vec<&'a SignSession>>,
) {
    let mut asigns = HashMap::new();
    let mut osigns = HashMap::new();
    for (_, session) in sessions {
        let (available_sign_activities, other_sign_activities, _) =
            session.traverse_activities().await.unwrap();
        for sa in available_sign_activities {
            let vec = vec![session];
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = asigns.try_insert(sa, vec)
            {
                entry.get_mut().push(session);
            }
        }
        for sa in other_sign_activities {
            let vec = vec![session];
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = osigns.try_insert(sa, vec)
            {
                entry.get_mut().push(session);
            }
        }
    }
    (asigns, osigns)
}
async fn photo_sign_(
    s: &SignActivity,
    session: &SignSession,
    pic: &Option<PathBuf>,
) -> Result<SignState, reqwest::Error> {
    if let Some(pic) = pic {
        let photo = Photo::from_file(session, &pic).await;
        s.photo_sign(&photo, session).await
    } else {
        let photo = Photo::default(session).await;
        s.photo_sign(&photo, session).await
    }
}
async fn general_sign_(
    s: &SignActivity,
    session: &SignSession,
) -> Result<SignState, reqwest::Error> {
    s.general_sign(session).await
}
async fn qrcode_sign_(
    s: &SignActivity,
    session: &SignSession,
    pic: &Option<PathBuf>,
    db: &DataBase,
    enc: &Option<String>,
    location: Option<i64>,
    pos: &Option<String>,
) -> Result<SignState, reqwest::Error> {
    let enc = {
        if let Some(enc) = enc {
            enc.to_owned()
        } else {
            if let Some(pic) = pic {
                let results = rxing::helpers::detect_multiple_in_file(pic.to_str().unwrap())
                    .expect("decodes");
                let r = &results[0];
                let r = r.getText();
                let beg = r.find("&enc=").unwrap();
                let s = &r[beg + 5..beg + 37];
                // println!("enc: {s}");
                s.to_owned()
            } else {
                panic!("二维码签到需要提供 `enc` 参数或签到二维码！")
            }
        }
    };
    if let Some(ref pos) = pos {
        let pos = Address::parse_str(pos);
        s.qrcode_sign(&enc, &pos, session).await
    } else if let Some(addr) = location {
        let poss = db.get_poss();
        let pos = &poss[&addr].1;
        s.qrcode_sign(&enc, &pos, session).await
    } else {
        let mut fail_msg = String::from("未查询到有效的位置信息，二维码签到失败！");
        let poss = db.get_course_poss(s.course.get_id());
        for (_, ref pos) in poss {
            match s.qrcode_sign(&enc, &pos, session).await? {
                r @ SignState::Success => return Ok(r),
                SignState::Fail(msg) => {
                    fail_msg.push_str("\n\t");
                    fail_msg.push_str(msg.as_str());
                    continue;
                }
            };
        }
        let poss = db.get_course_poss(-1);
        for (_, ref pos) in poss {
            match s.qrcode_sign(&enc, &pos, session).await? {
                r @ SignState::Success => return Ok(r),
                SignState::Fail(msg) => {
                    fail_msg.push_str("\n\t");
                    fail_msg.push_str(msg.as_str());
                    continue;
                }
            };
        }
        Ok(SignState::Fail(fail_msg))
    }
}
async fn location_sign_(
    s: &SignActivity,
    session: &SignSession,
    db: &DataBase,
    location: Option<i64>,
    pos: &Option<String>,
) -> Result<SignState, reqwest::Error> {
    if let Some(ref pos) = pos {
        let pos = Address::parse_str(pos);
        s.location_sign(&pos, session).await
    } else if let Some(addr) = location {
        let poss = db.get_poss();
        let pos = &poss[&addr].1;
        s.location_sign(&pos, session).await
    } else {
        let mut fail_msg = String::new();
        let poss = db.get_course_poss(s.course.get_id());
        for (_, ref pos) in poss {
            match s.location_sign(&pos, session).await? {
                r @ SignState::Success => return Ok(r),
                SignState::Fail(msg) => {
                    fail_msg.push_str("\n\t");
                    fail_msg.push_str(msg.as_str());
                    continue;
                }
            };
        }
        let poss = db.get_course_poss(-1);
        for (_, ref pos) in poss {
            match s.location_sign(&pos, session).await? {
                r @ SignState::Success => return Ok(r),
                SignState::Fail(msg) => {
                    fail_msg.push_str("\n\t");
                    fail_msg.push_str(msg.as_str());
                    continue;
                }
            };
        }
        Ok(SignState::Fail(fail_msg))
    }
}
async fn signcode_sign_(
    s: &SignActivity,
    signcode_sign_type: &SignType,
    session: &SignSession,
    signcode: &Option<String>,
) -> Result<SignState, reqwest::Error> {
    if let Some(signcode) = signcode {
        s.signcode_sign(session, signcode).await
    } else {
        match signcode_sign_type {
            SignType::Gesture => panic!("未提供手势，手势签到失败！"),
            SignType::SignCode => panic!("未提供签到码，签到码签到失败！"),
            _ => unreachable!(),
        }
    }
}
async fn pre_sign_and_handle_sign_type_sign(
    s: &SignActivity,
    sign_type: &SignType,
    session: &SignSession,
    pic: &Option<PathBuf>,
    location: Option<i64>,
    db: &DataBase,
    pos: &Option<String>,
    enc: &Option<String>,
    signcode: &Option<String>,
) -> Result<SignState, reqwest::Error> {
    s.pre_sign(session).await?;
    match sign_type {
        SignType::Photo => photo_sign_(s, session, &pic).await,
        SignType::Common => general_sign_(s, session).await,
        SignType::QrCode => qrcode_sign_(s, session, &pic, db, enc, location, pos).await,
        SignType::Location => location_sign_(s, session, db, location, pos).await,
        SignType::Unknown => {
            panic!("无效签到类型！");
        }
        signcode_sign_type => signcode_sign_(s, signcode_sign_type, session, signcode).await,
    }
}

async fn handle_account_sign<'a>(
    s: &SignActivity,
    picdir: &Option<PathBuf>,
    location: Option<i64>,
    db: &DataBase,
    pos: &Option<String>,
    enc: &Option<String>,
    signcode: &Option<String>,
    local_sessions: &'a Vec<&SignSession>,
) -> Result<(), reqwest::Error> {
    let sign_type = s.get_sign_type(local_sessions[0]).await?;
    let pic = if let Some(pic) = picdir {
        let metadata = std::fs::metadata(&pic).unwrap();
        if metadata.is_dir() {
            crate::utils::picdir_to_pic(&pic)
        } else {
            Some(pic.to_owned())
        }
    } else {
        None
    };
    for session in local_sessions {
        let state = pre_sign_and_handle_sign_type_sign(
            &s, &sign_type, session, &pic, location, db, &pos, &enc, &signcode,
        )
        .await?;
        match state {
            SignState::Success => println!("用户[{}]签到成功！", session.get_stu_name()),
            SignState::Fail(msg) => eprintln!("{msg}"),
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
    enc: Option<String>,
    picdir: Option<PathBuf>,
    signcode: Option<String>,
) -> Result<(), reqwest::Error> {
    if let Some(active_id) = activity {
        let s1 = asigns.iter().find(|kv| kv.0.id == active_id.to_string());
        let s2 = osigns.iter().find(|kv| kv.0.id == active_id.to_string());
        let (s, mut local_sessions) = {
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
            local_sessions = &accounts;
        }
        handle_account_sign(
            s,
            &picdir,
            location,
            db,
            &pos,
            &enc,
            &signcode,
            local_sessions,
        )
        .await?;
    } else {
        for (s, mut local_sessions) in &asigns {
            let mut accounts = Vec::new();
            if let Some(ref account) = &account {
                accounts.push(&sessions[account]);
                local_sessions = &accounts;
            }
            handle_account_sign(
                s,
                &picdir,
                location,
                db,
                &pos,
                &enc,
                &signcode,
                local_sessions,
            )
            .await?;
        }
    }
    Ok(())
}
