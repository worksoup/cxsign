#![feature(ascii_char)]
#![feature(thread_local_internals)]
#![feature(lint_reasons)]
#![feature(async_closure)]
#![feature(hash_set_entry)]
#![feature(map_try_insert)]

mod activity;
mod cli;
mod session;
mod utils;

use cli::arg::{AccCmds, Args, MainCmds, PosCmds};
use utils::{
    address::{add_pos, Address},
    sql::DataBase,
    CONFIG_DIR,
};

#[tokio::main]
async fn main() {
    let args = <Args as clap::Parser>::parse();
    let Args {
        command,
        activity,
        account,
        location,
        pos,
        pic,
        signcode,
        capture,
    } = args;
    let db = DataBase::new();
    if let Some(sub) = command {
        match sub {
            MainCmds::Account { command, fresh } => {
                if let Some(acc_sub) = command {
                    match acc_sub {
                        AccCmds::Add { uname } => {
                            // 添加账号。
                            utils::account::add_account(&db, uname, None).await;
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
                            utils::account::add_account_enc(&db, uname, enc_pwd).await;
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
                    let sessions = utils::account::get_sessions(&db).await;
                    db.delete_all_course();
                    for (_, session) in sessions {
                        let courses = session.get_courses().await.unwrap();
                        for c in courses {
                            db.add_course_or(&c, |_, _| {});
                        }
                    }
                }
                // 列出所有课程。
                let courses = db.get_courses();
                for c in courses {
                    c.1.display();
                }
            }
            MainCmds::Pos {
                command,
                course,
                global,
            } => {
                if let Some(pos_sub) = command {
                    match pos_sub {
                        PosCmds::Add { course, pos } => {
                            let mut course_id = -1_i64;
                            if let Some(id) = course {
                                if id < 0 {
                                    if id == -1 {
                                        eprintln!("警告：为课程号为 -1 的课程设置的位置将被视为全局位置！");
                                    } else {
                                        panic!("警告：课程号小于 0! 请检查是否正确！");
                                    }
                                } else {
                                    course_id = id;
                                }
                            }
                            add_pos(
                                &db,
                                course_id,
                                &Address::parse_str(&pos).unwrap_or_else(|e| panic!("{}", e)),
                            )
                        }
                        PosCmds::Remove { posid, yes, all } => {
                            fn confirm(msg: &str) -> bool {
                                inquire::Confirm::new(msg)
                                    .with_default(false)
                                    .prompt()
                                    .unwrap()
                            }
                            if let Some(posid) = posid {
                                if !yes {
                                    let ans = confirm("是否删除？");
                                    if !ans {
                                        return;
                                    }
                                }
                                // 删除指定位置。
                                db.delete_pos(posid);
                            } else if all {
                                if !yes {
                                    let ans = confirm("是否删除？");
                                    if !ans {
                                        return;
                                    }
                                }
                                if !yes {
                                    let ans = confirm("请再次确认，是否删除？");
                                    if !ans {
                                        return;
                                    }
                                }
                                db.delete_all_pos();
                            } else {
                                panic!("请提供要删除位置的 posid!")
                            }
                        }
                        PosCmds::Export { output } => {
                            // 列出所有位置。
                            let poss = db.get_poss();
                            let mut contents = String::new();
                            for pos in poss.values() {
                                contents += format!("{}${}\n", pos.0, pos.1).as_str()
                            }
                            std::fs::write(output, contents)
                                .expect("文件写入出错，请检查路径是否正确！");
                        }
                        PosCmds::Import { input } => {
                            let contents = std::fs::read_to_string(input)
                                .expect("文件读取失败，请检查路径是否正确！");
                            let contents = contents.split('\n');
                            let mut line_count = 1_i64;
                            for line in contents {
                                if !line.is_empty() {
                                    let data: Vec<&str> = line.split('$').collect();
                                    if data.len() > 1 {
                                        let mut course_id = -1_i64;
                                        if let Ok(id) = data[0].trim().parse::<i64>() {
                                            course_id = id;
                                        } else {
                                            eprintln!("警告：第 {line_count} 行课程号解析出错，该位置将尝试添加为全局位置！");
                                        }
                                        if let Ok(pos) = Address::parse_str(data[1]) {
                                            add_pos(&db, course_id, &pos)
                                        } else {
                                            eprintln!("错误：第 {line_count} 行位置解析出错, 该行将被跳过！格式应为 `addr,lon,lat,alt`");
                                        }
                                    } else {
                                        eprintln!("错误：第 {line_count} 行解析出错, 该行将被跳过！格式应为 `course_id$addr,lon,lat,alt`");
                                    }
                                }
                                line_count += 1;
                            }
                        }
                    }
                } else if global {
                    // 列出所有全局位置。
                    let poss = db.get_poss();
                    for pos in poss {
                        if pos.1 .0 == -1 {
                            println!(
                                "posid: {}, course_id: {}, addr: {:?}",
                                pos.0, pos.1 .0, pos.1 .1
                            )
                        }
                    }
                } else if let Some(course_id) = course {
                    // 列出指定课程的位置。
                    let poss = db.get_course_poss(course_id);
                    for pos in poss {
                        println!("posid: {}, addr: {:?}", pos.0, pos.1)
                    }
                } else {
                    // 列出所有位置。
                    let poss = db.get_poss();
                    for pos in poss {
                        println!(
                            "posid: {}, course_id: {}, addr: {}",
                            pos.0, pos.1 .0, pos.1 .1
                        )
                    }
                }
            }
            MainCmds::List { course, all } => {
                let sessions = utils::account::get_sessions(&db).await;
                let (available_sign_activities, other_sign_activities) =
                    utils::sign::get_signs(&sessions).await;
                if let Some(course) = course {
                    // 列出指定课程的有效签到。
                    for a in available_sign_activities {
                        if a.0.course.get_id() == course {
                            a.0.display(true);
                        }
                    }
                    if all {
                        // 列出指定课程的所有签到。
                        for a in other_sign_activities {
                            if a.0.course.get_id() == course {
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
                println!("{:?}", std::ops::Deref::deref(&CONFIG_DIR));
            }
        }
    } else {
        let sessions = utils::account::get_sessions(&db).await;
        let (asigns, osigns) = utils::sign::get_signs(&sessions).await;
        cli::sign(
            &db, &sessions, asigns, osigns, activity, account, location, pos, pic, signcode,
            capture,
        )
        .await
        .unwrap();
    }
    utils::print_now();
}
