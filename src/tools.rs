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

use cxsign::{
    store::{
        tables::{AccountTable, CourseTable},
        DataBase, DataBaseTableTrait,
    },
    utils::DIR,
    Course, Session,
};
use log::warn;

// 添加账号。
pub fn inquire_pwd_and_add_account(db: &DataBase, uname: String, pwd: Option<String>) {
    let pwd = if let Some(pwd) = pwd {
        pwd
    } else {
        match inquire::Password::new("密码：")
            .without_confirmation()
            .prompt()
        {
            Ok(pwd) => pwd,
            Err(e) => {
                warn!("输入的密码无法解析：{e}.");
                return;
            }
        }
    };
    let enc_pwd = cxsign::utils::des_enc(&pwd);
    let session = match Session::login(&DIR, &uname, &enc_pwd) {
        Ok(s) => s,
        Err(e) => {
            warn!("账号[{uname}]登录失败：{e}.");
            return;
        }
    };
    let table = AccountTable::from_ref(db);
    let name = session.get_stu_name();
    table.add_account_or(&uname, &enc_pwd, name, AccountTable::update_account);
    let courses = Course::get_courses(&session).unwrap_or_else(|e| {
        warn!(
            "用户[{}]获取课程列表失败，错误信息：{e}.",
            session.get_stu_name()
        );
        Default::default()
    });
    let table = CourseTable::from_ref(db);
    for c in courses {
        table.add_course_or(&c, |_, _| {});
    }
}
pub fn add_account_by_enc_pwd_when_fresh(db: &DataBase, uname: String, enc_pwd: &str) {
    let session = match Session::login(&DIR, &uname, enc_pwd) {
        Ok(s) => s,
        Err(e) => {
            warn!("账号[{uname}]登录失败：{e}.");
            return;
        }
    };
    let name = session.get_stu_name();
    let table = AccountTable::from_ref(db);
    table.add_account_or(&uname, enc_pwd, name, AccountTable::update_account);
    let courses = Course::get_courses(&session).unwrap_or_else(|e| {
        warn!(
            "用户[{}]获取课程列表失败，错误信息：{e}.",
            session.get_stu_name()
        );
        Default::default()
    });
    for c in courses {
        let table = CourseTable::from_ref(db);
        table.add_course_or(&c, |_, _| {});
    }
}
