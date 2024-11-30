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

#![feature(ascii_char)]
#![feature(async_closure)]
#![feature(hash_set_entry)]
#![feature(map_try_insert)]
#![feature(let_chains)]

mod cli;

// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use clap::CommandFactory;
use cli::arg::{AccountSubCommand, Args, MainCommand};
use cxlib::login::{DefaultLoginSolver, LoginSolverTrait, LoginSolverWrapper};
use cxlib::{
    activity::{Activity, RawSign},
    default_impl::store::{AccountTable, AliasTable, DataBase, ExcludeTable, LocationTable},
    sign::SignTrait,
    user::Session,
};
use log::{error, info, warn};
use std::collections::HashMap;
use std::io::stdout;

const NOTICE: &str = r#"
    
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

    列出签到时会默认排除从未发过签到或最后一次签到在 160 天
    之前的课程。

    如有需要，请使用 `cxsign list -a` 命令强制列出所有签到
    或使用 `cxsign list -c <COURSE_ID>` 列出特定课程的签
    到，此时将会刷新排除列表。

    注意，`cxsign list -a` 耗时十几秒到数分钟不等。不过后者
    耗时较短。

++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

"#;
pub fn run() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
    let mut builder = env_logger::Builder::from_env(env);
    builder.target(env_logger::Target::Stderr);
    builder.init();
    cxlib::dir::Dir::set_config_dir_info("TEST_CXSIGN", "up.workso", "Worksoup", "cxsign");
    let args = <Args as clap::Parser>::parse();
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    let Args {
        command,
        id,
        uid,
        location,
        image,
        code,
        precisely,
    } = args;
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    let Args {
        command,
        id,
        uid,
        location,
        image,
        code,
    } = args;
    let db = DataBase::default();
    db.add_table::<AccountTable>();
    db.add_table::<ExcludeTable>();
    db.add_table::<AliasTable>();
    db.add_table::<LocationTable>();
    if let Some(sub_cmd) = command {
        match sub_cmd {
            MainCommand::Account { command } => {
                match command {
                    AccountSubCommand::Add { uname, passwd } => {
                        let pwd = cxlib::utils::inquire_pwd(passwd);
                        let login_type_and_uname = uname.split_once(":");
                        let session = if let Some((login_type, uname)) = login_type_and_uname {
                            AccountTable::login(&db, uname.into(), pwd, login_type.into())
                        } else {
                            AccountTable::login(
                                &db,
                                uname.clone(),
                                pwd,
                                DefaultLoginSolver.login_type().into(),
                            )
                        };
                        // 添加账号。
                        match session {
                            Ok(session) => info!(
                                "添加账号 [{uname}]（用户名：{}）成功！",
                                session.get_stu_name()
                            ),
                            Err(e) => warn!("添加账号 [{uname}] 失败：{e}."),
                        };
                    }
                    AccountSubCommand::Remove { uid, yes } => {
                        if !yes {
                            let ans = inquire::Confirm::new("是否删除？")
                                .with_default(false)
                                .prompt()
                                .unwrap_or_else(|e| {
                                    warn!("无法识别输入：{e}.");
                                    false
                                });
                            if !ans {
                                return;
                            }
                        }
                        // 删除指定账号。
                        AccountTable::delete_account(&db, &uid);
                    }
                }
            }
            MainCommand::Accounts { fresh } => {
                let sessions: Vec<Session> = if fresh {
                    AccountTable::get_accounts(&db)
                        .into_iter()
                        .filter_map(|a| {
                            let session = Session::relogin(
                                a.uname(),
                                a.enc_pwd(),
                                &LoginSolverWrapper::new(a.login_type()),
                            );
                            session.ok()
                        })
                        .collect()
                } else {
                    // 列出所有账号。
                    AccountTable::get_sessions(&db).into_values().collect()
                };
                for session in sessions {
                    println!(
                        "{}, {}, {}",
                        session.get_uname(),
                        session.get_stu_name(),
                        session.get_uid()
                    );
                }
            }
            MainCommand::Courses { uid } => {
                let (sessions, _) = if let Some(uid_list_str) = &uid {
                    (
                        AccountTable::get_sessions_by_uid_list_str(&db, uid_list_str),
                        true,
                    )
                } else {
                    (AccountTable::get_sessions(&db), false)
                };
                // 获取课程信息。
                let courses =
                    cxlib::types::Course::get_courses(sessions.values()).unwrap_or_default();
                // 列出所有课程。
                for (c, _) in courses {
                    println!("{}", c);
                }
            }
            MainCommand::Location { command } => {
                cli::location::parse_location_sub_command(&db, command)
            }
            MainCommand::Locations {
                global,
                course,
                pretty,
                short,
            } => {
                let course_id = course.or(if global { Some(-1) } else { None });
                if short {
                    let locations = if let Some(course_id) = course_id {
                        LocationTable::get_location_map_by_course(&db, course_id)
                    } else {
                        LocationTable::get_locations(&db)
                            .into_iter()
                            .map(|(k, v)| (k, v.1))
                            .collect()
                    };
                    for (_, location) in locations {
                        println!("{}", location,)
                    }
                } else if let Some(course_id) = course_id {
                    // 列出指定课程的位置。
                    let locations = LocationTable::get_location_map_by_course(&db, course_id);
                    if pretty {
                        for (location_id, location) in locations {
                            println!(
                                "位置id: {}, 位置: {},\n\t别名: {:?}",
                                location_id,
                                location,
                                AliasTable::get_aliases(&db, location_id)
                            )
                        }
                    } else {
                        for (location_id, location) in locations {
                            println!(
                                "{}${}${:?}",
                                location_id,
                                location,
                                AliasTable::get_aliases(&db, location_id)
                            )
                        }
                    }
                } else {
                    // 列出所有位置。
                    let locations = LocationTable::get_locations(&db);
                    if pretty {
                        for (location_id, (course_id, location)) in locations {
                            println!(
                                "位置id: {}, 课程号: {}, 位置: {},\n\t别名: {:?}",
                                location_id,
                                course_id,
                                location,
                                AliasTable::get_aliases(&db, location_id)
                            )
                        }
                    } else {
                        for (location_id, (course_id, location)) in locations {
                            println!(
                                "{}${}${}${:?}",
                                location_id,
                                course_id,
                                location,
                                AliasTable::get_aliases(&db, location_id)
                            )
                        }
                    }
                }
            }
            MainCommand::List { course, all } => {
                let sessions = AccountTable::get_sessions(&db);
                if let Some(course) = course {
                    let courses = cxlib::types::Course::get_courses(sessions.values())
                        .unwrap_or_default()
                        .into_keys()
                        .map(|c| (c.get_id(), c))
                        .collect::<HashMap<_, _>>();
                    let (a, n) = if let Some(course) = courses.get(&course)
                        && let Some(session) = sessions.values().next()
                        && let Ok(a) = Activity::get_course_activities(&db, session, course)
                    {
                        a.into_iter()
                            .filter_map(|k| match k {
                                Activity::RawSign(k) => Some(k),
                                Activity::Other(_) => None,
                            })
                            .partition(|k| k.is_valid())
                    } else {
                        (vec![], vec![])
                    };
                    // 列出指定课程的有效签到。
                    for a in a {
                        if a.course.get_id() == course {
                            println!("{}", a.fmt_without_course_info());
                        }
                    }
                    if all {
                        // 列出指定课程的所有签到。
                        for a in n {
                            if a.course.get_id() == course {
                                println!("{}", a.fmt_without_course_info());
                            }
                        }
                    }
                } else {
                    let activities = Activity::get_all_activities(&db, sessions.values(), all)
                        .unwrap_or_else(|e| {
                            warn!("未能获取签到列表，错误信息：{e}.",);
                            Default::default()
                        });
                    let (available_sign_activities, other_sign_activities): (
                        Vec<RawSign>,
                        Vec<RawSign>,
                    ) = activities
                        .into_keys()
                        .filter_map(|k| match k {
                            Activity::RawSign(k) => Some(k),
                            Activity::Other(_) => None,
                        })
                        .partition(|a| a.is_valid());
                    // 列出所有有效签到。
                    for a in available_sign_activities {
                        println!("{}", a.as_inner());
                    }
                    if all {
                        // 列出所有签到。
                        for a in other_sign_activities {
                            println!("{}", a.as_inner());
                        }
                    } else {
                        warn!("{NOTICE}");
                    }
                }
            }
            MainCommand::WhereIsConfig => {
                println!(
                    "{}",
                    &cxlib::dir::Dir::get_config_dir()
                        .into_os_string()
                        .to_string_lossy()
                        .to_string()
                );
            }
            #[cfg(feature = "clap_complete_command")]
            MainCommand::Completions { shell, output } => {
                if let Some(output) = output {
                    shell
                        .generate_to(&mut Args::command(), output)
                        .map_err(|e| warn!("文件写入出错，请检查路径是否正确！错误信息：{e}"))
                        .unwrap();
                } else {
                    shell.generate(&mut Args::command(), &mut stdout());
                }
            }
        }
    } else {
        let cli_args = cli::arg::CliArgs {
            location_str: location,
            image,
            signcode: code,
            #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
            precisely,
        };
        warn!("{NOTICE}");
        cli::do_sign(db, id, uid, cli_args).unwrap_or_else(|e| error!("签到失败！错误信息：{e}."));
    }
}
