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
mod tools;

// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use crate::cli::location::database_add_location;
use cli::arg::{AccountSubCommand, Args, MainCommand};
use cxsign::{
    store::{
        tables::{AccountTable, AliasTable, CourseTable, ExcludeTable, LocationTable},
        DataBase, DataBaseTableTrait,
    },
    utils::DIR,
    Activity, Course, SignTrait,
};
use log::{error, info, warn};
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
    builder.target(env_logger::Target::Stdout);
    builder.init();
    cxsign::utils::set_boxed_location_preprocessor(Box::new(LocationPreprocessor));
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
    db.add_table::<CourseTable>();
    db.add_table::<ExcludeTable>();
    db.add_table::<AliasTable>();
    db.add_table::<LocationTable>();
    for (s, l) in xdsign_data::LOCATIONS.iter() {
        let alias_table = AliasTable::from_ref(&db);
        let location_table = LocationTable::from_ref(&db);
        if !alias_table.has_alias(s) {
            let lid = database_add_location(&location_table, -1, l);
            alias_table.add_alias_or(s, lid, |_, _, _| {})
        }
    }
    if let Some(sub_cmd) = command {
        match sub_cmd {
            MainCommand::Account { command } => {
                match command {
                    AccountSubCommand::Add { uname, passwd } => {
                        // 添加账号。
                        tools::inquire_pwd_and_add_account(&db, uname, passwd);
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
                    for (uname, (ref enc_pwd, _)) in accounts {
                        table.delete_account(&uname);
                        tools::add_account_by_enc_pwd_when_fresh(&db, uname, enc_pwd);
                    }
                }
                // 列出所有账号。
                let accounts = table.get_accounts();
                for a in accounts {
                    info!("{}, {}", a.0, a.1 .1);
                }
            }
            MainCommand::Courses { fresh } => {
                let table = CourseTable::from_ref(&db);
                if fresh {
                    // 重新获取课程信息并缓存。
                    let account_table = AccountTable::from_ref(&db);
                    let sessions = account_table.get_sessions();
                    CourseTable::delete(&db);
                    for (_, session) in sessions {
                        let courses = Course::get_courses(&session).unwrap_or_else(|e| {
                            warn!(
                                "未能获取到用户[{}]的课程，错误信息：{e}.",
                                session.get_stu_name()
                            );
                            Vec::default()
                        });
                        for c in courses {
                            table.add_course_or(&c, |_, _| {});
                        }
                    }
                }
                // 列出所有课程。
                let courses = table.get_courses();
                for c in courses {
                    info!("{}", c.1);
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
                    let (a, n) = if let Some(course) =
                        CourseTable::from_ref(&db).get_courses().get(&course)
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
