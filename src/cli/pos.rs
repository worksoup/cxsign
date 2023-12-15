use std::path::PathBuf;

use crate::utils::{
    address::{Struct位置, 为数据库添加位置},
    sql::DataBase,
};

pub struct CliPosArgs {
    pub posid: Option<i64>,
    pub list: bool,
    pub new: Option<String>,
    pub import: Option<PathBuf>,
    pub export: Option<PathBuf>,
    pub alias: Option<String>,
    pub remove: bool,
    pub remove_all: bool,
    pub course: Option<i64>,
    pub global: bool,
    pub yes: bool,
}

pub fn pos(db: &DataBase, cli_pos_args: CliPosArgs) {
    let CliPosArgs {
        posid,
        list,
        new,
        import,
        export,
        alias,
        remove,
        remove_all,
        course,
        global,
        yes,
    } = cli_pos_args;
    fn confirm(msg: &str) -> bool {
        inquire::Confirm::new(msg)
            .with_default(false)
            .prompt()
            .unwrap()
    }
    if list {
        if global {
            // 列出所有全局位置。
            let poss = db.get_poss();
            for pos in poss {
                if pos.1 .0 == -1 {
                    println!(
                        "posid: {}, course_id: {}, addr: {},\n\talias: {:?}",
                        pos.0,
                        pos.1 .0,
                        pos.1 .1,
                        db.get_aliases(pos.0)
                    )
                }
            }
        } else if let Some(course_id) = course {
            // 列出指定课程的位置。
            let poss = db.get_course_positions_and_posid(course_id);
            for pos in poss {
                println!(
                    "posid: {}, addr: {},\n\talias: {:?}",
                    pos.0,
                    pos.1,
                    db.get_aliases(pos.0)
                )
            }
        } else {
            // 列出所有位置。
            let poss = db.get_poss();
            for pos in poss {
                println!(
                    "posid: {}, course_id: {}, addr: {},\n\talias: {:?}",
                    pos.0,
                    pos.1 .0,
                    pos.1 .1,
                    db.get_aliases(pos.0)
                )
            }
        }
    } else if let Some(new) = new {
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
        为数据库添加位置(
            &db,
            course_id,
            &Struct位置::从字符串解析(&new).unwrap_or_else(|e| panic!("{}", e)),
        );
    } else if let Some(import) = import {
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
                    if let Ok(pos) = Struct位置::从字符串解析(data[1]) {
                        let posid = 为数据库添加位置(&db, course_id, &pos);
                        if data.len() > 2 {
                            let aliases: Vec<_> = data[2].split('/').map(|s| s.trim()).collect();
                            for alias in aliases {
                                if !alias.is_empty() {
                                    db.add_alias_or(alias, posid, |db, alias, posid| {
                                        db.update_alias(alias, posid);
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
        let positions = db.get_poss();
        let mut contents = String::new();
        for (posid, pos) in positions {
            let aliases = db.get_aliases(posid);
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
            contents += format!("{}${}${}\n", pos.0, pos.1, aliases_contents).as_str()
        }
        std::fs::write(export, contents).expect("文件写入出错，请检查路径是否正确！");
    } else if let Some(ref alias) = alias
        && let Some(posid) = posid
    {
        if posid < 0 || db.has_pos(posid) {
            db.add_alias_or(alias, posid, |db, alias, posid| {
                db.update_alias(alias, posid);
            });
        } else {
            eprintln!("警告：不能为不存在的位置添加别名！将不做任何事。")
        }
    } else if let Some(alias) = alias {
        if remove {
            if !yes {
                let ans = confirm("警告：是否删除？");
                if !ans {
                    return;
                }
            }
            if db.has_alias(&alias) {
                db.delete_alias(&alias);
            } else {
                eprintln!("警告：该别名并不存在，将不做任何事情。");
            }
        } else if remove_all {
            if !yes {
                let ans = confirm("警告：是否删除？");
                if !ans {
                    return;
                }
            }
            if !yes {
                let ans = confirm("警告：请再次确认，是否删除？");
                if !ans {
                    return;
                }
            }
            db.delete_all_alias();
        }
    } else if remove && let Some(posid) = posid {
        if !yes {
            let ans = confirm("警告：是否删除？");
            if !ans {
                return;
            }
        }
        // 删除指定位置。
        db.delete_pos(posid);
    } else if remove_all {
        if !yes {
            let ans = confirm("警告：是否删除？");
            if !ans {
                return;
            }
        }
        if !yes {
            let ans = confirm("警告：请再次确认，是否删除？");
            if !ans {
                return;
            }
        }
        db.delete_all_pos();
    }
}
