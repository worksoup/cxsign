#![feature(ascii_char)]
#![feature(thread_local_internals)]
#![feature(lint_reasons)]
#![feature(async_closure)]
#![feature(hash_set_entry)]
#![feature(map_try_insert)]

mod cli;
mod gui;
mod sign_session;
mod utils;

use clap::{Parser, Subcommand};
use std::{ops::Deref, path::PathBuf};
use utils::{address::Address, sql::DataBase, CONFIG_DIR};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let Args {
        command,
        activity,
        account,
        location,
        pos,
        enc,
        pic,
        signcode,
        gui,
    } = args;

    if gui {
        gui::gui().await;
    } else {
        let db = DataBase::new();
        if let Some(sub) = command {
            match sub {
                MainCmds::Account { command, fresh } => {
                    if let Some(acc_sub) = command {
                        match acc_sub {
                            AccCmds::Add { uname } => {
                                // 添加账号。
                                cli::add_account(&db, uname, None).await;
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
                            for (uname, (pwd, _)) in accounts {
                                db.delete_account(&uname);
                                cli::add_account(&db, uname, Some(pwd)).await;
                            }
                        } else {
                            // 列出所有账号。
                            for a in accounts {
                                println!("{}, {}", a.0, a.1 .1);
                            }
                        }
                    }
                }
                MainCmds::Course { fresh } => {
                    if fresh {
                        // 重新获取课程信息并缓存。
                        let sessions = cli::get_sessions(&db).await;
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
                                if let Some(course_id) = course {
                                    if course_id < 0 {
                                        eprintln!("警告：课程号小于 0! 请检查是否正确！");
                                        if course_id == -1 {
                                            eprintln!("警告：为课程号为 -1 的课程设置的位置将被视为全局位置！");
                                        }
                                    }
                                    // 为指定课程添加位置。
                                    let mut posid = 0_i64;
                                    loop {
                                        if db.has_pos(posid) {
                                            posid = posid + 1;
                                            continue;
                                        }
                                        db.add_pos_or(
                                            posid,
                                            course_id,
                                            &Address::parse_str(&pos),
                                            |_, _, _, _| {},
                                        );
                                        break;
                                    }
                                } else {
                                    // 添加全局位置。
                                    let mut posid = 0_i64;
                                    loop {
                                        if db.has_pos(posid) {
                                            posid = posid + 1;
                                            continue;
                                        }
                                        db.add_pos_or(
                                            posid,
                                            -1,
                                            &Address::parse_str(&pos),
                                            |_, _, _, _| {},
                                        );
                                        break;
                                    }
                                }
                            }
                            PosCmds::Remove { posid, yes } => {
                                if !yes {
                                    let ans = inquire::Confirm::new("是否删除？")
                                        .with_default(false)
                                        .prompt()
                                        .unwrap();
                                    if !ans {
                                        return;
                                    }
                                }
                                // 删除指定位置。
                                db.delete_pos(posid);
                            }
                        }
                    } else {
                        if global {
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
                        } else {
                            if let Some(course_id) = course {
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
                                        "posid: {}, course_id: {}, addr: {:?}",
                                        pos.0, pos.1 .0, pos.1 .1
                                    )
                                }
                            }
                        }
                    }
                }
                MainCmds::List { course, all } => {
                    let sessions = cli::get_sessions(&db).await;
                    let (available_sign_activities, other_sign_activities) =
                        cli::get_signs(&sessions).await;
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
                    println!("{:?}", CONFIG_DIR.deref());
                }
            }
        } else {
            let sessions = cli::get_sessions(&db).await;
            let (asigns, osigns) = cli::get_signs(&sessions).await;
            let pic = if let Some(pic) = pic {
                let metadata = std::fs::metadata(&pic).unwrap();
                if metadata.is_dir() {
                    cli::picdir_to_pic(&pic)
                } else {
                    Some(pic)
                }
            } else {
                None
            };
            cli::sign(
                &db, &sessions, asigns, osigns, activity, account, location, pos, enc, pic,
                signcode,
            )
            .await
            .unwrap();
        }
        cli::print_now();
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "进行签到。", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<MainCmds>,
    /// 签到 ID.
    /// 默认以最近起对所有有效签到顺序进行签到，且缺少参数时会跳过并继续。
    activity: Option<i64>,
    /// 签到账号。
    /// 默认以一定顺序对所有用户进行签到。
    #[arg(short, long)]
    account: Option<String>,
    /// 位置 ID.
    /// 位置签到或二维码位置签到时需要提供。
    /// 也可以通过 `--pos` 选项直接指定位置，此时本选项将失效。
    /// 默认按照先课程位置后全局位置的顺序依次尝试。
    #[arg(short, long)]
    location: Option<i64>,
    /// 通过地址名称、经纬度与海拔直接指定位置。
    /// 位置签到或二维码位置签到时需要提供。
    /// 格式为：`addr/lat/lon/alt`.
    #[arg(long)]
    pos: Option<String>,
    /// `enc` 参数。
    /// 二维码签到时需要提供该参数。
    #[arg(short, long)]
    enc: Option<String>,
    /// 本地图片路径。
    /// 拍照签到或二维码时需要提供，
    /// 如果是文件，则直接使用该文件作为拍照签到图片或二维码图片文件。
    /// 如果是目录，则会选择在该目录下修改日期最新的图片作为拍照签到图片或二维码图片。
    #[arg(short, long)]
    pic: Option<PathBuf>,
    /// 签到码。
    /// 签到码签到时需要提供。
    #[arg(short, long)]
    signcode: Option<String>,
    /// 以图形界面模式启动。
    #[arg(short, long)]
    gui: bool,
}

#[derive(Subcommand, Debug)]
enum MainCmds {
    /// 账号相关操作（列出、添加、删除）。
    /// 默认列出所有账号。
    Account {
        #[command(subcommand)]
        command: Option<AccCmds>,
        /// 重新获取账号信息并缓存。
        #[arg(short, long)]
        fresh: bool,
    },
    /// 列出所有课程。
    Course {
        /// 重新获取课程信息并缓存。
        #[arg(short, long)]
        fresh: bool,
    },
    /// 列出有效签到。
    List {
        /// 列出指定课程的签到。
        #[arg(short, long)]
        course: Option<i64>,
        /// 列出所有签到（包括无效签到）。
        #[arg(short, long)]
        all: bool,
    },
    /// 位置相关操作（列出、添加、删除）。
    /// 默认列出所有位置。
    Pos {
        #[command(subcommand)]
        command: Option<PosCmds>,
        /// 列出绑定指定课程的位置。
        #[arg(short, long)]
        course: Option<i64>,
        /// 列出全局位置。
        #[arg(short, long)]
        global: bool,
    },
    /// 显示配置文件夹位置。
    WhereIsConfig,
}

#[derive(Subcommand, Debug)]
enum AccCmds {
    /// 添加账号。
    Add {
        /// 账号（手机号）。
        uname: String,
    },
    /// 删除账号。
    Remove {
        /// 账号（手机号）。
        uname: String,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
}
#[derive(Subcommand, Debug)]
enum PosCmds {
    /// 添加位置。
    Add {
        /// 绑定该位置到指定课程。
        /// 默认添加为全局位置。
        #[arg(short, long)]
        course: Option<i64>,
        /// 地址名称、经纬度与海拔。
        /// 格式为：`addr/lat/lon/alt`.
        pos: String,
    },
    /// 删除位置。
    Remove {
        /// 位置 ID.
        posid: i64,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
}
