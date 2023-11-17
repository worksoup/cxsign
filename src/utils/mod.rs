pub mod address;
pub mod api;
pub mod photo;
pub mod sql;

use des::{
    cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit},
    Des,
};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::{fs::DirEntry, path::PathBuf};
use unicode_width::UnicodeWidthStr;
lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = {
        let binding = ProjectDirs::from("rt.lea", "worksoup", "cxsign").unwrap();
        let dir = binding.config_dir().to_owned();
        let _ = std::fs::create_dir_all(dir.clone());
        dir
    };
}

pub fn print_now() {
    let str = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now())
        .format("%+")
        .to_string();
    println!("{str}");
}

pub fn picdir_to_pic(picdir: &PathBuf) -> Option<PathBuf> {
    loop {
        let ans = inquire::Confirm::new("二维码图片是否准备好了？").with_help_message("本程序会读取 `--picdir` 参数所指定的路径下最新修改的图片。你可以趁现在获取这张图片，然后按下回车进行签到。").with_default_value_formatter(&|v|{
            if v {"是[默认]"}else{"否[默认]"}.into()
        }).with_formatter(&|v|{
            if v {"是"}else{"否"}.into()
        }).with_parser(&|s|{
            match inquire::Confirm::DEFAULT_PARSER(s) {
                r@Ok(_) => r,
                Err(_) => {
                    if s == "是"{
                        Ok(true)
                    }else if s =="否"  {
                        Ok(false)
                    }else {
                        Err(())
                    }
                },
            }
        }).with_error_message("请以\"y\", \"yes\"等表示“是”，\"n\", \"no\"等表示“否”。")
        .with_default(true)
        .prompt()
        .unwrap();
        if ans {
            break;
        }
    }
    let pic = if let Ok(pic_dir) = std::fs::read_dir(picdir) {
        let mut files: Vec<DirEntry> = pic_dir
            .filter_map(|k| {
                if let Ok(k) = k {
                    if let Ok(t) = k.file_type() {
                        if t.is_file() {
                            let name = k.file_name();
                            let ext = name.to_str().unwrap().split('.').last().unwrap();
                            if ext == "png" || ext == "jpg" {
                                Some(k)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        if files.is_empty() {
            eprintln!("文件夹下没有图片！（只支持 `*.png` 文件或 `*.jpg` 文件。）");
            None
        } else {
            files.sort_by(|a, b| {
                b.metadata()
                    .unwrap()
                    .modified()
                    .unwrap()
                    .cmp(&a.metadata().unwrap().modified().unwrap())
            });
            Some(files[0].path())
        }
    } else {
        eprintln!("遍历文件夹失败！");
        None
    };
    pic
}
pub fn pwd_des(pwd: &str) -> String {
    fn pkcs7(pwd: &str) -> Vec<[u8; 8]> {
        assert!(pwd.len() > 7);
        assert!(pwd.len() < 17);
        let mut r = Vec::new();
        let pwd = pwd.as_bytes();
        let len = pwd.len();
        let batch = len / 8;
        let m = len % 8;
        for i in 0..batch {
            let mut a = [0u8; 8];
            a.copy_from_slice(&pwd[0 + i * 8..8 + i * 8]);
            r.push(a);
        }
        let mut b = [0u8; 8];
        for i in 0..m {
            b[i] = pwd[8 * batch as usize + i];
        }
        for i in m..8 {
            b[i] = (8 - m) as u8;
        }
        r.push(b);
        println!("{r:?}");
        r
    }
    let key = b"u2oh6Vu^".to_owned();
    let key = GenericArray::from(key);
    let des = Des::new(&key);
    let pwd = pkcs7(pwd);
    let mut a = Vec::new();
    for b in pwd {
        let mut b = GenericArray::from(b);
        des.encrypt_block(&mut b);
        let mut b = b.to_vec();
        a.append(&mut b);
    }
    hex::encode(a)
}

pub fn get_unicode_correct_display_width(s: &str, perfer_width: usize) -> usize {
    if UnicodeWidthStr::width(s) > perfer_width {
        perfer_width
    } else {
        UnicodeWidthStr::width(s) + 12 - s.len()
    }
}

// mod test {
//     #[test]
//     fn test_des() {
//         println!("{}", crate::utils::pwd_des("0123456789."));
//     }
// }
