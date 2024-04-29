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
use std::path::PathBuf;

use cxsign::{
    store::{
        tables::{AliasTable, LocationTable},
        DataBase, DataBaseTableTrait,
    },
    Location,
};
use log::{debug, error, warn};

fn database_add_location(table: &LocationTable, course_id: i64, location: &Location) -> i64 {
    // 为指定课程添加位置。
    let mut lid = 0_i64;
    loop {
        if table.has_location(lid) {
            lid += 1;
            continue;
        }
        table.add_location_or(lid, course_id, location, |_, _, _, _| {});
        break;
    }
    lid
}

#[derive(Subcommand, Debug)]
pub enum LocationSubCommand {
    /// 添加位置或别名。
    Add {
        /// 地址名称、经纬度与海拔。
        /// 格式为：`addr,lon,lat,alt`.
        /// 格式为：`地址,经度,纬度,海拔`.
        location_str: String,
        /// 为位置添加别名。
        alias: Option<String>,
        /// 绑定该位置到指定课程。
        /// 默认添加为全局位置。
        #[arg(short, long)]
        course: Option<i64>,
    },
    /// 删除位置。
    Remove {
        #[command(subcommand)]
        command: Remove,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
    /// 批量删除位置。
    Reduce {
        #[command(subcommand)]
        reduce_type: ReduceType,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
        /// 指定全部。
        #[arg(short, long)]
        all: bool,
        /// 指定课程号。
        #[arg(short, long)]
        course: Option<i64>,
        /// 指定全局。
        #[arg(short, long)]
        global: bool,
    },
    /// 导入位置。
    Import {
        /// 导入位置。
        /// 每行一个位置。课程号在前，位置在后，最后是别名。它们由字符 `$` 隔开。
        /// 其中位置的格式为 `地址,经度,纬度,海拔`, 别名的格式为以 `/` 分隔的字符串数组。
        input: PathBuf,
    },
    /// 导入位置。
    Export {
        /// 导出位置。
        /// 每行一个位置。课程号在前，位置在后，最后是别名。它们由字符 `$` 隔开。
        /// 其中位置的格式为 `地址,经度,纬度,海拔`, 别名的格式为以 `/` 分隔的字符串数组。
        /// 无法解析的行将会被跳过。
        output: PathBuf,
    },
}
#[derive(Subcommand, Debug, Clone)]
pub enum ReduceType {
    /// 位置。
    Locations,
    /// 位置别名。
    Aliases {
        /// 位置 ID.
        location_id: Option<i64>,
    },
}
#[derive(Subcommand, Debug)]
pub enum Remove {
    Locations {
        /// 位置 ID.
        #[arg(short, long)]
        location_id: Option<i64>,
        /// 位置别名对应的位置。
        #[arg(short, long)]
        alias: Option<String>,
    },
    Aliases {
        /// 位置别名。
        alias: String,
    },
}

pub fn parse_location_sub_command(db: &DataBase, sub_command: LocationSubCommand) {
    let location_table = LocationTable::from_ref(db);
    let alias_table = AliasTable::from_ref(db);
    fn confirm(msg: &str) -> bool {
        inquire::Confirm::new(msg)
            .with_default(false)
            .prompt()
            .unwrap()
    }
    match sub_command {
        LocationSubCommand::Add {
            location_str,
            alias,
            course,
        } => {
            let mut course_id = -1_i64;
            if let Some(id) = course {
                if id < 0 {
                    if id == -1 {
                        warn!("警告：为课程号为 -1 的课程设置的位置将被视为全局位置！");
                    } else {
                        error!("错误：课程号小于 0! 请检查是否正确！");
                        panic!()
                    }
                } else {
                    course_id = id;
                }
            }
            let location = Location::parse(&location_str);
            let location_id = if let Ok(location) = location {
                database_add_location(&location_table, course_id, &location)
            } else {
                if alias.is_none() {
                    warn!("无法确定所要操作的位置对象！");
                    return;
                }
                let location_id = location_str.trim().parse::<i64>();
                if let Ok(location_id) = location_id
                    && location_table.has_location(location_id)
                {
                    location_id
                } else if alias_table.has_alias(location_str.trim())
                    && let Some(location_id) = alias_table.get_location_id(location_str.trim())
                {
                    location_id
                } else {
                    warn!("无法确定所要操作的位置对象！");
                    return;
                }
            };
            if let Some(alias) = alias {
                alias_table.add_alias_or(&alias, location_id, |alias_table, alias, location_id| {
                    alias_table.update_alias(alias, location_id);
                })
            }
        }
        LocationSubCommand::Remove { command, yes } => {
            if !yes {
                let ans = confirm("警告：是否删除？");
                if !ans {
                    return;
                }
            }
            match command {
                Remove::Locations { location_id, alias } => {
                    let location_id = location_id
                        .or_else(|| alias.and_then(|alias| alias_table.get_location_id(&alias)));
                    if let Some(location_id) = location_id
                        && location_table.has_location(location_id)
                    {
                        location_table.delete_location(location_id);
                        let aliases = alias_table.get_aliases(location_id);
                        for alias in aliases.iter() {
                            alias_table.delete_alias(alias)
                        }
                    } else {
                        warn!("警告：未指定有效的位置，将不做任何事情。");
                    }
                }
                Remove::Aliases { alias } => {
                    if alias_table.has_alias(&alias) {
                        alias_table.delete_alias(&alias);
                    } else {
                        warn!("警告：该别名并不存在，将不做任何事情。");
                    }
                }
            }
        }
        LocationSubCommand::Reduce {
            reduce_type,
            yes,
            all,
            course,
            global,
        } => {
            if !yes {
                let ans = confirm("警告：是否删除？");
                if !ans {
                    return;
                }
            }
            let course_id = course.or(if global { Some(-1) } else { None });
            let mut locations: Vec<i64> = if let Some(course_id) = course_id {
                location_table
                    .get_location_map_by_course(course_id)
                    .into_keys()
                    .collect()
            } else if all {
                location_table.get_locations().into_keys().collect()
            } else {
                vec![]
            };
            let delete_locations = match reduce_type {
                ReduceType::Locations => true,
                ReduceType::Aliases { location_id } => {
                    if let Some(location_id) = location_id {
                        locations = vec![location_id]
                    }
                    false
                }
            };

            if locations.is_empty() {
                if delete_locations {
                    warn!("警告：未指定任何有效的位置，将不做任何事情。");
                } else {
                    warn!("警告：未指定任何有效的别名，将不做任何事情。");
                }
                return;
            }
            let mut aliases = Vec::new();
            for location_id in locations.iter() {
                let aliases_ = alias_table.get_aliases(*location_id);
                for alias in aliases_ {
                    aliases.push(alias)
                }
            }
            if !delete_locations && aliases.is_empty() {
                warn!("警告：未指定任何有效的别名，将不做任何事情。");
                return;
            }
            if !yes {
                let ans = confirm("再次警告：是否删除？");
                if !ans {
                    return;
                }
            }
            if delete_locations {
                for location_id in locations {
                    location_table.delete_location(location_id);
                }
            }
            for alias in aliases {
                alias_table.delete_alias(&alias)
            }
        }
        LocationSubCommand::Import { input } => {
            let contents =
                std::fs::read_to_string(input).expect("文件读取失败，请检查路径是否正确！");
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
                            warn!(
                            "警告：第 {line_count} 行课程号解析出错，该位置将尝试添加为全局位置！"
                        );
                        }
                        if let Ok(location) = Location::parse(data[1]) {
                            let location_id =
                                database_add_location(&location_table, course_id, &location);
                            if data.len() > 2 {
                                let aliases: Vec<_> =
                                    data[2].split('/').map(|s| s.trim()).collect();
                                for alias in aliases {
                                    if !alias.is_empty() {
                                        alias_table.add_alias_or(alias, location_id, |t, a, l| {
                                            t.update_alias(a, l);
                                        })
                                    }
                                }
                            }
                        } else {
                            warn!("错误：第 {line_count} 行位置解析出错, 该行将被跳过！格式应为 `地址,经度,纬度,海拔`");
                        }
                    } else {
                        warn!("错误：第 {line_count} 行解析出错, 该行将被跳过！格式应为 `course_id$addr,lon,lat,alt`");
                    }
                }
                line_count += 1;
            }
        }
        LocationSubCommand::Export { output } => {
            let locations = location_table.get_locations();
            let mut contents = String::new();
            for (location_id, (course_id, location)) in locations {
                let aliases = alias_table.get_aliases(location_id);
                let mut aliases_contents = String::new();
                if !aliases.is_empty() {
                    aliases_contents.push_str(&aliases[0]);
                    for alias in aliases.iter().skip(1) {
                        aliases_contents.push('/');
                        aliases_contents.push_str(alias);
                    }
                }
                debug!("{aliases:?}");
                contents += format!("{}${}${}\n", course_id, location, aliases_contents).as_str()
            }
            std::fs::write(output, contents).expect("文件写入出错，请检查路径是否正确！");
        }
    }
}
