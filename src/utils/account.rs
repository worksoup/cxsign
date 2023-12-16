use std::collections::HashMap;

use crate::session::Struct签到会话;

use super::sql::DataBase;

// 添加账号。TODO: 跳过输入密码阶段
pub async fn 添加账号(db: &DataBase, uname: String, pwd: Option<String>) {
    let pwd = if let Some(pwd) = pwd {
        pwd
    } else {
        inquire::Password::new("密码：")
            .without_confirmation()
            .prompt()
            .unwrap()
    };
    let enc_pwd = crate::utils::des加密(&pwd);
    let session = Struct签到会话::通过账号密码登录(&uname, &enc_pwd)
        .await
        .unwrap();
    let name = session.get_用户真名();
    db.add_account_or(&uname, &enc_pwd, name, DataBase::update_account);
    let courses = session.获取课程列表().await.unwrap();
    for c in courses {
        db.add_course_or(&c, |_, _| {});
    }
}
pub async fn 添加账号_使用加密过的密码_刷新时用_此时密码一定是存在的且为加密后的密码(
    db: &DataBase,
    uname: String,
    加密过的密码: &str,
) {
    let session = Struct签到会话::通过账号密码登录(&uname, 加密过的密码)
        .await
        .unwrap();
    let name = session.get_用户真名();
    db.add_account_or(&uname, 加密过的密码, name, DataBase::update_account);
    let courses = session.获取课程列表().await.unwrap();
    for c in courses {
        db.add_course_or(&c, |_, _| {});
    }
}

pub async fn 通过账号获取签到会话(
    db: &DataBase,
    账号列表: &Vec<&str>,
) -> HashMap<String, Struct签到会话> {
    // let accounts = db.get_accounts();
    let 配置文件夹 = crate::utils::配置文件夹.clone();
    let mut s = HashMap::new();
    for 账号 in 账号列表 {
        if db.has_account(账号) {
            let cookies文件路径 = 配置文件夹.join(账号.to_string() + ".json");
            let 签到会话 = Struct签到会话::从cookies文件加载(cookies文件路径)
                .await
                .unwrap();
            s.insert(账号.to_string(), 签到会话);
        }
    }
    s
}
