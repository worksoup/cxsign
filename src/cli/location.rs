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

use clap::Subcommand;
use cxsign::{
    store::{
        tables::{AccountTable, LocationTable},
        DataBase, DataBaseTableTrait,
    },
    LocationWithRange,
};
use log::{info, warn};
use std::{collections::HashMap, path::PathBuf};

#[derive(Subcommand, Debug)]
pub enum LocationSubCommand {
    /// 导出位置。
    Export {
        /// 导出位置。
        /// 每行一个位置。课程号在前，位置在后，最后是别名。它们由字符 `$` 隔开。
        /// 其中位置的格式为 `地址,经度,纬度,海拔`, 别名的格式为以 `/` 分隔的字符串数组。
        /// 无法解析的行将会被跳过。
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 从班级历史签到中获取位置并导出。格式同上。
        #[arg(short, long)]
        course: Option<i64>,
    },
}

pub fn parse_location_sub_command(db: &DataBase, sub_command: LocationSubCommand) {
    let location_table = LocationTable::from_ref(db);
    match sub_command {
        LocationSubCommand::Export { output, course } => {
            let mut contents = location_table.export();
            for content in {
                course
                    .and_then(|course_id| {
                        let account_table = AccountTable::from_ref(db);
                        let sessions = account_table.get_sessions();
                        let courses = cxsign::Course::get_courses(sessions.values())
                            .unwrap_or_default()
                            .into_keys()
                            .map(|c| (c.get_id(), c))
                            .collect::<HashMap<_, _>>();
                        courses.get(&course_id).and_then(|course| {
                            sessions.values().next().map(|session| {
                                let mut contents = Vec::new();
                                match LocationWithRange::from_log(session, course) {
                                    Ok(locations) => {
                                        if locations.is_empty() {
                                            warn!("没有从该课程中获取到位置信息。");
                                        }
                                        for (_, l) in locations {
                                            contents.push(format!(
                                                "{}${}${}\n",
                                                course_id,
                                                l.to_shifted_location(),
                                                ""
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        warn!("遇到了问题：{e}");
                                    }
                                }
                                contents.into_iter()
                            })
                        })
                    })
                    .into_iter()
                    .flatten()
            } {
                contents += content.as_str()
            }
            if contents.is_empty() {
                warn!("没有获取到位置，不做任何事情。")
            } else if let Some(output) = output {
                let _ = std::fs::write(output, contents)
                    .map_err(|e| warn!("文件写入出错，请检查路径是否正确！错误信息：{e}"));
            } else {
                println!("{contents}")
            }
        }
    }
}
