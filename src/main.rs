#![feature(ascii_char)]
#![feature(lint_reasons)]
#![feature(async_closure)]
#![feature(hash_set_entry)]
#![feature(map_try_insert)]
#![feature(let_chains)]

mod cli;
mod tools;

use cli::{
    arg::{AccCmds, Args, MainCmds},
    location::Struct位置操作使用的信息,
};
use cxsign::{
    store::{
        tables::{AccountTable, CourseTable},
        DataBase, DataBaseTableTrait,
    },
    utils::DIR,
    Course, SignTrait,
};

fn main() {
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
    let db = DataBase::default();
    if let Some(sub_cmd) = command {
        match sub_cmd {
            MainCmds::Account { command, fresh } => {
                if let Some(acc_sub_cmd) = command {
                    match acc_sub_cmd {
                        AccCmds::Add { uname } => {
                            // 添加账号。
                            tools::添加账号(&db, uname, None);
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
                            AccountTable::from_ref(&db).delete_account(&uname);
                        }
                    }
                } else {
                    let table = AccountTable::from_ref(&db);
                    let accounts = table.get_accounts();
                    if fresh {
                        for (uname, (ref enc_pwd, _)) in accounts {
                            table.delete_account(&uname);
                            tools::添加账号_使用加密过的密码_刷新时用_此时密码一定是存在的且为加密后的密码(&db, uname, enc_pwd);
                        }
                    }
                    // 列出所有账号。
                    let accounts = table.get_accounts();
                    for a in accounts {
                        println!("{}, {}", a.0, a.1 .1);
                    }
                }
            }
            MainCmds::Course { fresh } => {
                let table = CourseTable::from_ref(&db);
                if fresh {
                    // 重新获取课程信息并缓存。
                    let sessions = tools::通过账号获取签到会话(
                        &db,
                        &AccountTable::from_ref(&db)
                            .get_accounts()
                            .keys()
                            .map(|s| s.as_str())
                            .collect(),
                    )
                    ;
                    CourseTable::delete(&db);
                    for (_, session) in sessions {
                        let courses = Course::get_courses(&session).unwrap();
                        for c in courses {
                            table.add_course_or(&c, |_, _| {});
                        }
                    }
                }
                // 列出所有课程。
                let courses = table.get_courses();
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
                    location_id: lication_id,
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
                let sessions = tools::通过账号获取签到会话(
                    &db,
                    &AccountTable::from_ref(&db)
                        .get_accounts()
                        .keys()
                        .map(|s| s.as_str())
                        .collect(),
                )
                ;
                let (available_sign_activities, other_sign_activities) =
                    tools::获取所有签到(&sessions);
                if let Some(course) = course {
                    // 列出指定课程的有效签到。
                    for a in available_sign_activities {
                        if a.0.as_inner().course.get_id() == course {
                            println!("{}", a.0.as_inner().fmt_without_course_info());
                        }
                    }
                    if all {
                        // 列出指定课程的所有签到。
                        for a in other_sign_activities {
                            if a.0.as_inner().course.get_id() == course {
                                println!("{}", a.0.as_inner().fmt_without_course_info());
                            }
                        }
                    }
                } else {
                    // 列出所有有效签到。
                    for a in available_sign_activities {
                        println!("{}", a.0.as_inner());
                    }
                    if all {
                        // 列出所有签到。
                        for a in other_sign_activities {
                            println!("{}", a.0.as_inner());
                        }
                    }
                }
            }
            MainCmds::WhereIsConfig => {
                println!("{}", &DIR.get_config_dir().to_str().unwrap());
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
            
            .unwrap();
    }
    cxsign::utils::print_now();
}
