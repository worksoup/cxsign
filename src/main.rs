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
#![feature(lint_reasons)]
#![feature(async_closure)]
#![feature(hash_set_entry)]
#![feature(map_try_insert)]
#![feature(let_chains)]

mod cli;

// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use cli::arg::{AccountSubCommand, Args, MainCommand};
use cxsign::{
    store::{
        tables::{AccountTable, AliasTable, ExcludeTable, LocationTable},
        DataBase, DataBaseTableTrait,
    },
    utils::DIR,
    Activity, SignTrait,
};
use log::{error, info, warn};
use std::collections::HashMap;
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
    builder.target(env_logger::Target::Stdout);
    builder.init();
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
    db.add_table::<AliasTable>();
    db.add_table::<LocationTable>();
    if let Some(sub_cmd) = command {
        match sub_cmd {
            MainCommand::Account { command } => {
                match command {
                    AccountSubCommand::Add { uname, passwd } => {
                        let table = AccountTable::from_ref(&db);
                        let pwd = cxsign::utils::inquire_pwd(passwd);
                        let session = table.login(uname.clone(), pwd);
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
                        AccountTable::from_ref(&db).delete_account(&uname);
                    }
                }
            }
            MainCommand::Accounts { fresh } => {
                let table = AccountTable::from_ref(&db);
                let accounts = table.get_accounts();
                if fresh {
                    for (cxsign::UnameAndEncPwdPair { uname, enc_pwd }, _) in accounts {
                        let session = table.relogin(uname.clone(), &enc_pwd);
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
                let accounts = table.get_accounts();
                for a in accounts {
                    info!("{}, {}", a.0.uname, a.1);
                }
            }
            MainCommand::Courses { accounts } => {
                let account_table = AccountTable::from_ref(&db);
                let (sessions, _) = if let Some(accounts_str) = &accounts {
                    (
                        account_table.get_sessions_by_accounts_str(accounts_str),
                        true,
                    )
                } else {
                    (account_table.get_sessions(), false)
                };
                // 获取课程信息。
                let courses = cxsign::Course::get_courses(sessions.values()).unwrap_or_default();
                // 列出所有课程。
                for (c, _) in courses {
                    info!("{}", c);
                }
            }
            MainCommand::Location { command } => {
                cli::location::parse_location_sub_command(&db, command)
            }
            MainCommand::Locations { global, course } => {
                let location_table = LocationTable::from_ref(&db);
                let alias_table = AliasTable::from_ref(&db);
                let course_id = course.or(if global { Some(-1) } else { None });
                if let Some(course_id) = course_id {
                    // 列出指定课程的位置。
                    let locations = location_table.get_location_map_by_course(course_id);
                    for (location_id, location) in locations {
                        info!(
                            "位置id: {}, 位置: {},\n\t别名: {:?}",
                            location_id,
                            location,
                            alias_table.get_aliases(location_id)
                        )
                    }
                } else {
                    // 列出所有位置。
                    let locations = location_table.get_locations();
                    for (location_id, (course_id, location)) in locations {
                        info!(
                            "位置id: {}, 课程号: {}, 位置: {},\n\t别名: {:?}",
                            location_id,
                            course_id,
                            location,
                            alias_table.get_aliases(location_id)
                        )
                    }
                }
            }
            MainCommand::List { course, all } => {
                let sessions = AccountTable::from_ref(&db).get_sessions();
                if let Some(course) = course {
                    let courses = cxsign::Course::get_courses(sessions.values())
                        .unwrap_or_default()
                        .into_keys()
                        .map(|c| (c.get_id(), c))
                        .collect::<HashMap<_, _>>();
                    let (a, n) = if let Some(course) = courses.get(&course)
                        && let Some(session) = sessions.values().next()
                        && let Ok((a, n, _)) = Activity::get_course_activities(
                            ExcludeTable::from_ref(&db),
                            session,
                            course,
                        ) {
                        (a, n)
                    } else {
                        (vec![], vec![])
                    };
                    // 列出指定课程的有效签到。
                    for a in a {
                        if a.course.get_id() == course {
                            info!("{}", a.fmt_without_course_info());
                        }
                    }
                    if all {
                        // 列出指定课程的所有签到。
                        for a in n {
                            if a.course.get_id() == course {
                                info!("{}", a.fmt_without_course_info());
                            }
                        }
                    }
                } else {
                    let (available_sign_activities, other_sign_activities, _) =
                        Activity::get_all_activities(
                            ExcludeTable::from_ref(&db),
                            sessions.values(),
                            all,
                        )
                        .unwrap_or_else(|e| {
                            warn!("未能获取签到列表，错误信息：{e}.",);
                            Default::default()
                        });
                    // 列出所有有效签到。
                    for a in available_sign_activities {
                        info!("{}", a.0.as_inner());
                    }
                    if all {
                        // 列出所有签到。
                        for a in other_sign_activities {
                            info!("{}", a.0.as_inner());
                        }
                    } else {
                        warn!("{NOTICE}");
                    }
                }
            }
            MainCommand::WhereIsConfig => {
                info!(
                    "{}",
                    &DIR.get_config_dir()
                        .into_os_string()
                        .to_string_lossy()
                        .to_string()
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
