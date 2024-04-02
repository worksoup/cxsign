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
    let table = AccountTable::from_ref(db);
    let name = session.get_stu_name();
    table.add_account_or(&uname, &enc_pwd, name, AccountTable::update_account);
    let courses = Course::get_courses(&session).unwrap();
    for c in courses {
        let table = CourseTable::from_ref(db);
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
    let table = AccountTable::from_ref(db);
    table.add_account_or(&uname, 加密过的密码, name, AccountTable::update_account);
    let courses = Course::get_courses(&session).unwrap();
    for c in courses {
        let table = CourseTable::from_ref(db);
        table.add_course_or(&c, |_, _| {});
    }
}
