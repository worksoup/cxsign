use cxsign::{
    store::{
        tables::{AccountTable, CourseTable},
        DataBase, DataBaseTableTrait,
    },
    utils::DIR,
    Course, Session, Sign,
};
use std::collections::{hash_map::OccupiedError, HashMap};

// 添加账号。TODO: 跳过输入密码阶段
pub fn 添加账号(db: &DataBase, uname: String, pwd: Option<String>) {
    let pwd = if let Some(pwd) = pwd {
        pwd
    } else {
        inquire::Password::new("密码：")
            .without_confirmation()
            .prompt()
            .unwrap()
    };
    let enc_pwd = cxsign::utils::des_enc(&pwd);
    let session = Session::login(&DIR, &uname, &enc_pwd).unwrap();
    let table = AccountTable::from_ref(&db);
    let name = session.get_stu_name();
    table.add_account_or(&uname, &enc_pwd, name, AccountTable::update_account);
    let courses = Course::get_courses(&session).unwrap();
    for c in courses {
        let table = CourseTable::from_ref(&db);
        table.add_course_or(&c, |_, _| {});
    }
}
pub fn 添加账号_使用加密过的密码_刷新时用_此时密码一定是存在的且为加密后的密码(
    db: &DataBase,
    uname: String,
    加密过的密码: &str,
) {
    let session = Session::login(&DIR, &uname, 加密过的密码).unwrap();
    let name = session.get_stu_name();
    let table = AccountTable::from_ref(&db);
    table.add_account_or(&uname, 加密过的密码, name, AccountTable::update_account);
    let courses = Course::get_courses(&session).unwrap();
    for c in courses {
        let table = CourseTable::from_ref(&db);
        table.add_course_or(&c, |_, _| {});
    }
}

pub fn 通过账号获取签到会话(
    db: &DataBase,
    账号列表: &Vec<&str>,
) -> HashMap<String, Session> {
    let mut s = HashMap::new();
    for 账号 in 账号列表 {
        let table = AccountTable::from_ref(&db);
        if table.has_account(账号) {
            let 签到会话 = Session::load_json(&DIR, 账号).unwrap();
            s.insert(账号.to_string(), 签到会话);
        }
    }
    s
}

pub fn 获取所有签到(
    sessions: &HashMap<String, Session>,
) -> (
    HashMap<Sign, HashMap<&String, &Session>>,
    HashMap<Sign, HashMap<&String, &Session>>,
) {
    let mut 有效签到 = HashMap::new();
    let mut 其他签到 = HashMap::new();
    for session in sessions {
        let (available_sign_activities, other_sign_activities, _) =
            cxsign::Activity::get_all_activities(session.1).unwrap();
        for sa in available_sign_activities {
            let mut map = HashMap::new();
            map.insert(session.0, session.1);
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = 有效签到.try_insert(sa, map)
            {
                entry.get_mut().insert(session.0, session.1);
            }
        }
        for sa in other_sign_activities {
            let mut map = HashMap::new();
            map.insert(session.0, session.1);
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = 其他签到.try_insert(sa, map)
            {
                entry.get_mut().insert(session.0, session.1);
            }
        }
    }
    (有效签到, 其他签到)
}
