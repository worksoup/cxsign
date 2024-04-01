use cxsign::{
    store::{
        tables::{AccountTable, CourseTable},
        DataBase, DataBaseTableTrait,
    },
    utils::DIR,
    Course, Session,
};

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
