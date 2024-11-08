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
mod xddcc;

// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use cli::arg::{AccountSubCommand, Args, MainCommand};
use cxsign::{
    activity::{Activity, RawSign},
    default_impl::store::{AccountTable, DataBase, ExcludeTable, UnameAndEncPwdPair},
    dir::Dir,
    sign::SignTrait,
    types::Location,
    user::Session,
};
use log::{error, info, warn};
use std::collections::HashMap;
use xdsign_data::LocationPreprocessor;

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
fn main() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
    let mut builder = env_logger::Builder::from_env(env);
    // builder.target(env_logger::Target::Stdout);
    let logger = builder.build();
    let multi = indicatif::MultiProgress::new();
    indicatif_log_bridge::LogWrapper::new(multi.clone(), logger)
        .try_init()
        .unwrap_or_else(|e| {
            error!("日志初始化失败。错误信息：{e}.");
            panic!()
        });
    Dir::set_config_dir_info("TEST_XDSIGN", "rt.lea", "Leart", "xdsign");
    let _ = cxsign::default_impl::init_all();
    Location::set_boxed_location_preprocessor(Box::new(LocationPreprocessor))
        .unwrap_or_else(|e| error!("{e}"));
    let args = <Args as clap::Parser>::parse();
    let Args {
        command,
        active_id,
        accounts,
        location,
        image,
        signcode,
        precisely,
    } = args;
    let db = DataBase::default();
    db.add_table::<AccountTable>();
    db.add_table::<ExcludeTable>();
    if let Some(sub_cmd) = command {
        match sub_cmd {
            MainCommand::Account { command } => {
                match command {
                    AccountSubCommand::Add { uname, passwd } => {
                        let pwd = cxsign::utils::inquire_pwd(passwd);
                        let session = AccountTable::login(&db, uname.clone(), pwd);
                        // 添加账号。
                        match session {
                            Ok(session) => info!(
                                "添加账号 [{uname}]（用户名：{}）成功！",
                                session.get_stu_name()
                            ),
                            Err(e) => warn!("添加账号 [{uname}] 失败：{e}."),
                        };
                    }
                    AccountSubCommand::Remove { uname, yes } => {
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
                        AccountTable::delete_account(&db, &uname);
                    }
                }
            }
            MainCommand::Accounts { fresh } => {
                let accounts = AccountTable::get_accounts(&db);
                if fresh {
                    for (UnameAndEncPwdPair { uname, enc_pwd }, _) in accounts {
                        let session = Session::relogin(&uname, &enc_pwd);
                        match session {
                            Ok(session) => info!(
                                "刷新账号 [{uname}]（用户名：{}）成功！",
                                session.get_stu_name()
                            ),
                            Err(e) => warn!("刷新账号 [{uname}] 失败：{e}."),
                        };
                    }
                }
                // 列出所有账号。
                let accounts = AccountTable::get_accounts(&db);
                for a in accounts {
                    println!("{}, {}", a.0.uname, a.1);
                }
            }
            MainCommand::Courses { accounts } => {
                let (sessions, _) = if let Some(accounts_str) = &accounts {
                    (
                        AccountTable::get_sessions_by_accounts_str(&db, accounts_str),
                        true,
                    )
                } else {
                    (AccountTable::get_sessions(&db), false)
                };
                // 获取课程信息。
                let courses =
                    cxsign::types::Course::get_courses(sessions.values()).unwrap_or_default();
                // 列出所有课程。
                for (c, _) in courses {
                    println!("{}", c);
                }
            }
            MainCommand::Locations { pretty, short } => {
                if short {
                    let locations = xdsign_data::LOCATIONS.iter();
                    for (_, location) in locations {
                        println!("{}", location,)
                    }
                } else {
                    // 列出所有位置。
                    let locations = xdsign_data::LOCATIONS.iter();
                    if pretty {
                        for (alias, location) in locations {
                            println!("位置: {}, 别名: {}", location, alias)
                        }
                    } else {
                        for (alias, location) in locations {
                            println!("{}${}", location, alias)
                        }
                    }
                }
            }
            MainCommand::List { course, all } => {
                let sessions = AccountTable::get_sessions(&db);
                if let Some(course) = course {
                    let courses = cxsign::types::Course::get_courses(sessions.values())
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
                    &cxsign::dir::Dir::get_config_dir()
                        .into_os_string()
                        .to_string_lossy()
                        .to_string()
                );
            }
            MainCommand::Xddcc {
                accounts,
                this,
                device_code,
                output,
                list,
                id,
                just_id,
            } => {
                xddcc::xddcc(
                    (accounts, device_code, id),
                    output,
                    (&db, &multi),
                    (this, just_id, list),
                );
            }
        }
    } else {
        let cli_args = cli::arg::CliArgs {
            location_str: location,
            image,
            signcode,
            precisely,
        };
        warn!("{NOTICE}");
        cli::do_sign(db, active_id, accounts, cli_args)
            .unwrap_or_else(|e| error!("签到失败！错误信息：{e}."));
    }
}
