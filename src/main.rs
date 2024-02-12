#![feature(ascii_char)]
#![feature(lint_reasons)]
#![feature(async_closure)]
#![feature(hash_set_entry)]
#![feature(map_try_insert)]
#![feature(let_chains)]

mod activity;
mod cli;
mod session;
mod utils;
mod protocol;

use cli::{
    arg::{AccCmds, Args, MainCmds},
    location::Struct位置操作使用的信息,
};
use utils::{sql::DataBase, 配置文件夹};

#[tokio::main]
async fn main() {
    let args = <Args as clap::Parser>::parse();
    let Args {
        command,
        active_id,
        accounts,
        location,
        image,
        signcode,
        precisely,
        no_random_shift,
    } = args;
    let db = DataBase::new();
    if let Some(sub_cmd) = command {
        match sub_cmd {
            MainCmds::Account { command, fresh } => {
                if let Some(acc_sub_cmd) = command {
                    match acc_sub_cmd {
                        AccCmds::Add { uname } => {
                            // 添加账号。
                            utils::account::添加账号(&db, uname, None).await;
                        }
                        AccCmds::Remove { uname, yes } => {
                            if !yes {
                                let ans = inquire::Confirm::new("是否删除？")
                                    .with_default(false)
                                    .prompt()
                                    .unwrap();
                                if !ans {
                                    return;
                                }
                            }
                            // 删除指定账号。
                            db.delete_account(&uname);
                        }
                    }
                } else {
                    let accounts = db.get_accounts();
                    if fresh {
                        for (uname, (ref enc_pwd, _)) in accounts {
                            db.delete_account(&uname);
                            utils::account::添加账号_使用加密过的密码_刷新时用_此时密码一定是存在的且为加密后的密码(&db, uname, enc_pwd).await;
                        }
                    }
                    // 列出所有账号。
                    let accounts = db.get_accounts();
                    for a in accounts {
                        println!("{}, {}", a.0, a.1 .1);
                    }
                }
            }
            MainCmds::Course { fresh } => {
                if fresh {
                    // 重新获取课程信息并缓存。
                    let sessions = utils::account::通过账号获取签到会话(
                        &db,
                        &db.get_accounts().keys().map(|s| s.as_str()).collect(),
                    )
                    .await;
                    db.delete_all_course();
                    for (_, session) in sessions {
                        let courses = session.获取课程列表().await.unwrap();
                        for c in courses {
                            db.add_course_or(&c, |_, _| {});
                        }
                    }
                }
                // 列出所有课程。
                let courses = db.get_courses();
                for c in courses {
                    println!("{}", c.1);
                }
            }
            MainCmds::Location {
                lication_id,
                list,
                new,
                import,
                export,
                alias,
                remove,
                remove_locations,
                remove_aliases,
                course,
                global,
                yes,
            } => {
                let args = Struct位置操作使用的信息 {
                    lication_id,
                    list,
                    new,
                    import,
                    export,
                    alias,
                    remove,
                    remove_locations,
                    remove_aliases,
                    course,
                    global,
                    yes,
                };
                cli::location::location(&db, args)
            }
            MainCmds::List { course, all } => {
                let sessions = utils::account::通过账号获取签到会话(
                    &db,
                    &db.get_accounts().keys().map(|s| s.as_str()).collect(),
                )
                .await;
                let (available_sign_activities, other_sign_activities) =
                    utils::sign::获取所有签到(&sessions).await;
                if let Some(course) = course {
                    // 列出指定课程的有效签到。
                    for a in available_sign_activities {
                        if a.0.课程.get_课程号() == course {
                            a.0.display(true);
                        }
                    }
                    if all {
                        // 列出指定课程的所有签到。
                        for a in other_sign_activities {
                            if a.0.课程.get_课程号() == course {
                                a.0.display(true);
                            }
                        }
                    }
                } else {
                    // 列出所有有效签到。
                    for a in available_sign_activities {
                        a.0.display(false);
                    }
                    if all {
                        // 列出所有签到。
                        for a in other_sign_activities {
                            a.0.display(false);
                        }
                    }
                }
            }
            MainCmds::WhereIsConfig => {
                println!("{:?}", std::ops::Deref::deref(&配置文件夹));
            }
        }
    } else {
        let 签到可能使用的信息 = cli::arg::CliArgs {
            位置字符串: location,
            图片或图片路径: image,
            签到码: signcode,
            是否精确识别二维码: precisely,
            是否禁用随机偏移: no_random_shift,
        };
        cli::签到(&db, active_id, accounts, 签到可能使用的信息)
            .await
            .unwrap();
    }
    utils::打印当前时间();
}
