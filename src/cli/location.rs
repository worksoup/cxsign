use std::path::PathBuf;

use crate::utils::{
    address::{Struct位置, 为数据库添加位置},
    sql::DataBase,
};

pub struct Struct位置操作使用的信息 {
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

pub fn location(db: &DataBase, 位置操作使用的信息: Struct位置操作使用的信息) {
    fn confirm(msg: &str) -> bool {
        inquire::Confirm::new(msg)
            .with_default(false)
            .prompt()
            .unwrap()
    }
    let Struct位置操作使用的信息 {
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
    } = 位置操作使用的信息;
    if let Some(new) = new {
        let over_args = || {
            lication_id.is_some()
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
        let 位置id = 为数据库添加位置(
            &db,
            course_id,
            &Struct位置::从字符串解析(&new).unwrap_or_else(|e| panic!("{}", e)),
        );
        if let Some(alias) = alias {
            db.add_alias_or(&alias, 位置id, |db, alias, 位置id| {
                db.update_alias(alias, 位置id);
            })
        }
    } else if let Some(import) = import {
        let over_args = || {
            lication_id.is_some()
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
                    if let Ok(位置) = Struct位置::从字符串解析(data[1]) {
                        let 位置id = 为数据库添加位置(&db, course_id, &位置);
                        if data.len() > 2 {
                            let 别名列表: Vec<_> = data[2].split('/').map(|s| s.trim()).collect();
                            for 别名 in 别名列表 {
                                if !别名.is_empty() {
                                    db.add_alias_or(别名, 位置id, |db, 别名, 位置id| {
                                        db.update_alias(别名, 位置id);
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
            lication_id.is_some()
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
        let 位置列表 = db.获取所有位置();
        let mut contents = String::new();
        for (位置id, 位置) in 位置列表 {
            let aliases = db.get_aliases(位置id);
            let mut aliases_contents = String::new();
            if !aliases.is_empty() {
                aliases_contents.push_str(&aliases[0]);
                for i in 1..aliases.len() {
                    aliases_contents.push('/');
                    aliases_contents.push_str(&aliases[i]);
                }
            }
            #[cfg(debug_assertions)]
            println!("{aliases:?}");
            contents += format!("{}${}${}\n", 位置.0, 位置.1, aliases_contents).as_str()
        }
        std::fs::write(export, contents).expect("文件写入出错，请检查路径是否正确！");
    } else if let Some(ref alias) = alias
        && let Some(位置id) = lication_id
    {
        let over_args =
            || remove || remove_locations || remove_aliases || course.is_some() || global || yes;
        if over_args() {
            eprintln!(
                "本行命令将被解释为设置别名。需要 `location_id` 参数。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
            )
        }
        if 位置id < 0 || db.是否存在为某id的位置(位置id) {
            db.add_alias_or(alias, 位置id, |db, alias, 位置id| {
                db.update_alias(alias, 位置id);
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
            if over_args() || lication_id.is_some() {
                eprintln!(
                    "本行命令将被解释为删除别名。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                )
            }
            if db.has_alias(&alias) {
                db.delete_alias(&alias);
            } else {
                eprintln!("警告：该别名并不存在，将不做任何事情。");
            }
        } else if let Some(位置id) = lication_id {
            if over_args() {
                eprintln!(
                    "本行命令将被解释为删除地址。可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                )
            }
            db.删除为某id的位置(位置id);
        }
    } else if remove_aliases || remove_locations {
        if course.is_some() && global {
            eprintln!("选项`-c, --course` 和 `-g, --global` 不会同时起效，将解释为前者。")
        }
        let 待操作位置列表: Vec<_> = if let Some(course_id) = course {
            db.获取特定课程的位置和其id(course_id)
                .keys()
                .map(|id| *id)
                .collect()
        } else if global {
            db.获取所有位置()
                .keys()
                .filter_map(|id| if (*id) == -1 { Some(*id) } else { None })
                .collect()
        } else {
            db.获取所有位置().keys().map(|id| *id).collect()
        };
        if !yes {
            let ans = confirm("警告：是否删除？");
            if !ans {
                return;
            }
        }
        if 待操作位置列表.len() > 1 {
            if !yes {
                let ans = confirm("警告：删除数目大于 1, 请再次确认，是否删除？");
                if !ans {
                    return;
                }
            }
        }
        // 删除指定位置。
        if remove_aliases {
            if remove_locations || alias.is_some() || lication_id.is_some() {
                if alias.is_none() && lication_id.is_none() {
                    eprintln!(
                        "本行命令将被解释为删除一类位置的别名。`    --remove-all` 选项将不起效。"
                    )
                } else {
                    eprintln!(
                        "本行命令将被解释为删除一类位置的别名。可使用的选项有`-c, --course`, `-g, --global`, `-y, --yes`. 可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                    )
                }
            }
            for 位置id in 待操作位置列表 {
                let aliases = db.get_aliases(位置id);
                for alias in aliases {
                    db.delete_alias(&alias);
                }
            }
        } else {
            if alias.is_some() || lication_id.is_some() {
                eprintln!(
                        "本行命令将被解释为删除一类位置的别名。可使用的选项有`-c, --course`, `-g, --global`, `-y, --yes`. 可同时起效的选项有 `-l, --list`, 其余选项将不起效。"
                    )
            }
            for 位置id in 待操作位置列表 {
                db.删除为某id的位置(位置id);
            }
        }
    }
    if list {
        if global {
            // 列出所有全局位置。
            let 位置列表 = db.获取所有位置();
            for 位置 in 位置列表 {
                if 位置.1 .0 == -1 {
                    println!(
                        "位置id: {}, 课程号: {}, 位置: {},\n\t别名: {:?}",
                        位置.0,
                        位置.1 .0,
                        位置.1 .1,
                        db.get_aliases(位置.0)
                    )
                }
            }
        } else if let Some(course_id) = course {
            // 列出指定课程的位置。
            let 位置列表 = db.获取特定课程的位置和其id(course_id);
            for 位置 in 位置列表 {
                println!(
                    "位置id: {}, 位置: {},\n\t别名: {:?}",
                    位置.0,
                    位置.1,
                    db.get_aliases(位置.0)
                )
            }
        } else {
            // 列出所有位置。
            let 位置列表 = db.获取所有位置();
            for 位置 in 位置列表 {
                println!(
                    "位置id: {}, 课程号: {}, 位置: {},\n\t别名: {:?}",
                    位置.0,
                    位置.1 .0,
                    位置.1 .1,
                    db.get_aliases(位置.0)
                )
            }
        }
    }
}
