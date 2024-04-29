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

use std::path::PathBuf;

use cxsign::{
    store::{
        tables::{AliasTable, LocationTable},
        DataBase, DataBaseTableTrait,
    },
    Location,
};

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

pub struct LocationCliArgs {
    pub location_id: Option<i64>,
    pub list: bool,
    pub new: Option<String>,
    pub import: Option<PathBuf>,
    pub export: Option<PathBuf>,
    pub alias: Option<String>,
    pub remove: bool,
    pub remove_locations: bool,
    pub remove_aliases: bool,
    pub course: Option<i64>,
    pub global: bool,
    pub yes: bool,
}

pub fn location(db: &DataBase, cli_args: LocationCliArgs) {
    let location_table = LocationTable::from_ref(db);
    let alias_table = AliasTable::from_ref(db);
    fn confirm(msg: &str) -> bool {
        inquire::Confirm::new(msg)
            .with_default(false)
            .prompt()
            .unwrap()
    }
    let LocationCliArgs {
        location_id,
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
    } = cli_args;
    if let Some(new) = new {
        let over_args = || {
            location_id.is_some()
                || import.is_some()
                || export.is_some()
                || remove
                || remove_locations
                || remove_aliases
                || global
                || yes
        };
        if over_args() {
            eprintln!("本行命令将被解释为添加新的位置。可使用选项有 `-c, --course`, `-a, --alias`, 可同时起效的选项有 `-l, --list`, 其余选项将不起效。")
        }
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
        let location_id = database_add_location(
            &location_table,
            course_id,
            &Location::parse(&new).unwrap_or_else(|e| panic!("{}", e)),
        );
        if let Some(alias) = alias {
            alias_table.add_alias_or(&alias, location_id, |alias_table, alias, location_id| {
                alias_table.update_alias(alias, location_id);
            })
        }
    } else if let Some(import) = import {
        let over_args = || {
            location_id.is_some()
                || export.is_some()
                || alias.is_some()
                || remove
                || remove_locations
                || remove_aliases
                || course.is_some()
                || global
                || yes
        };
        if over_args() {
            eprintln!(
                "本行命令将被解释为导入位置。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
            )
        }
        let contents = std::fs::read_to_string(import).expect("文件读取失败，请检查路径是否正确！");
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
                        eprintln!(
                            "警告：第 {line_count} 行课程号解析出错，该位置将尝试添加为全局位置！"
                        );
                    }
                    if let Ok(location) = Location::parse(data[1]) {
                        let location_id =
                            database_add_location(&location_table, course_id, &location);
                        if data.len() > 2 {
                            let aliases: Vec<_> = data[2].split('/').map(|s| s.trim()).collect();
                            for alias in aliases {
                                if !alias.is_empty() {
                                    alias_table.add_alias_or(alias, location_id, |t, a, l| {
                                        t.update_alias(a, l);
                                    })
                                }
                            }
                        }
                    } else {
                        eprintln!("错误：第 {line_count} 行位置解析出错, 该行将被跳过！格式应为 `地址,经度,纬度,海拔`");
                    }
                } else {
                    eprintln!("错误：第 {line_count} 行解析出错, 该行将被跳过！格式应为 `course_id$addr,lon,lat,alt`");
                }
            }
            line_count += 1;
        }
    } else if let Some(export) = export {
        let over_args = || {
            location_id.is_some()
                || alias.is_some()
                || remove
                || remove_locations
                || remove_aliases
                || course.is_some()
                || global
                || yes
        };
        if over_args() {
            eprintln!(
                "本行命令将被解释为导出位置。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
            )
        }
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
            #[cfg(debug_assertions)]
            println!("{aliases:?}");
            contents += format!("{}${}${}\n", course_id, location, aliases_contents).as_str()
        }
        std::fs::write(export, contents).expect("文件写入出错，请检查路径是否正确！");
    } else if let Some(ref alias) = alias
        && let Some(location_id) = location_id
    {
        let over_args =
            || remove || remove_locations || remove_aliases || course.is_some() || global || yes;
        if over_args() {
            eprintln!(
                "本行命令将被解释为设置别名。需要 `location_id` 参数。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
            )
        }
        if location_id < 0 || location_table.has_location(location_id) {
            alias_table.add_alias_or(alias, location_id, |alias_table, alias, location_id| {
                alias_table.update_alias(alias, location_id);
            });
        } else {
            eprintln!("警告：不能为不存在的位置添加别名！将不做任何事。")
        }
    } else if remove {
        let over_args = || remove_locations || remove_aliases || course.is_some() || global;
        if !yes {
            let ans = confirm("警告：是否删除？");
            if !ans {
                return;
            }
        }
        if let Some(alias) = alias {
            if over_args() || location_id.is_some() {
                eprintln!(
                    "本行命令将被解释为删除别名。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                )
            }
            if alias_table.has_alias(&alias) {
                alias_table.delete_alias(&alias);
            } else {
                eprintln!("警告：该别名并不存在，将不做任何事情。");
            }
        } else if let Some(location_id) = location_id {
            if over_args() {
                eprintln!(
                    "本行命令将被解释为删除地址。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                )
            }
            location_table.delete_location(location_id);
        }
    } else if remove_aliases || remove_locations {
        if course.is_some() && global {
            eprintln!("选项`-c, --course` 和 `-g, --global` 不会同时起效，将解释为前者。")
        }
        let locations_id: Vec<_> = if let Some(course_id) = course {
            location_table
                .get_location_map_by_course(course_id)
                .keys()
                .copied()
                .collect()
        } else if global {
            location_table
                .get_locations()
                .keys()
                .filter_map(|id| if (*id) == -1 { Some(*id) } else { None })
                .collect()
        } else {
            location_table.get_locations().keys().copied().collect()
        };
        if !yes {
            let ans = confirm("警告：是否删除？");
            if !ans {
                return;
            }
        }
        if locations_id.len() > 1 && !yes {
            let ans = confirm("警告：删除数目大于 1, 请再次确认，是否删除？");
            if !ans {
                return;
            }
        }
        // 删除指定位置。
        if remove_aliases {
            if remove_locations || alias.is_some() || location_id.is_some() {
                if alias.is_none() && location_id.is_none() {
                    eprintln!(
                        "本行命令将被解释为删除一类位置的别名。`    --remove-all` 选项将不起效。"
                    )
                } else {
                    eprintln!(
                        "本行命令将被解释为删除一类位置的别名。可使用的选项有`-c, --course`, `-g, --global`, `-y, --yes`. 可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                    )
                }
            }
            for location_id in locations_id {
                let aliases = alias_table.get_aliases(location_id);
                for alias in aliases {
                    alias_table.delete_alias(&alias);
                }
            }
        } else {
            if alias.is_some() || location_id.is_some() {
                eprintln!(
                    "本行命令将被解释为删除一类位置的别名。可使用的选项有`-c, --course`, `-g, --global`, `-y, --yes`. 可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                )
            }
            for location_id in locations_id {
                location_table.delete_location(location_id);
            }
        }
    }
    if list {
        if global {
            // 列出所有全局位置。
            let locations = location_table.get_locations();
            for (location_id, (course_id, location)) in locations {
                if course_id == -1 {
                    println!(
                        "位置id: {}, 课程号: {}, 位置: {},\n\t别名: {:?}",
                        location_id,
                        course_id,
                        location,
                        alias_table.get_aliases(location_id)
                    )
                }
            }
        } else if let Some(course_id) = course {
            // 列出指定课程的位置。
            let locations = location_table.get_location_map_by_course(course_id);
            for (location_id, location) in locations {
                println!(
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
                println!(
                    "位置id: {}, 课程号: {}, 位置: {},\n\t别名: {:?}",
                    location_id,
                    course_id,
                    location,
                    alias_table.get_aliases(location_id)
                )
            }
        }
    }
}
