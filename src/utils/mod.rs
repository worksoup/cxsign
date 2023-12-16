pub mod account;
pub mod address;
pub mod photo;
pub mod query;
pub mod sign;
pub mod sql;

use des::{
    cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit},
    Des,
};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;
lazy_static! {
    pub static ref 配置文件夹: PathBuf = {
        let is_testing = std::env::var("TEST_CXSIGN").is_ok();
        let binding = ProjectDirs::from("rt.lea", "worksoup", "cxsign").unwrap();
        let dir = if is_testing {
            binding.config_dir().join("test").to_owned()
        } else {
            binding.config_dir().to_owned()
        };
        let _ = std::fs::create_dir_all(dir.clone());
        dir
    };
}

pub fn 打印当前时间() {
    let str = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now())
        .format("%+")
        .to_string();
    println!("{str}");
}

pub fn 请求确认(询问文本: &str, 提示文本: &str) -> bool {
    inquire::Confirm::new(询问文本)
        .with_help_message(提示文本)
        .with_default_value_formatter(&|v| if v { "是[默认]" } else { "否[默认]" }.into())
        .with_formatter(&|v| if v { "是" } else { "否" }.into())
        .with_parser(&|s| match inquire::Confirm::DEFAULT_PARSER(s) {
            r @ Ok(_) => r,
            Err(_) => {
                if s == "是" {
                    Ok(true)
                } else if s == "否" {
                    Ok(false)
                } else {
                    Err(())
                }
            }
        })
        .with_error_message("请以\"y\", \"yes\"等表示“是”，\"n\", \"no\"等表示“否”。")
        .with_default(true)
        .prompt()
        .unwrap()
}

pub fn des加密(密码文本: &str) -> String {
    fn pkcs7填充(密码文本: &str) -> Vec<[u8; 8]> {
        assert!(密码文本.len() > 7);
        assert!(密码文本.len() < 17);
        let mut r = Vec::new();
        let pwd = 密码文本.as_bytes();
        let len = pwd.len();
        let batch = len / 8;
        let m = len % 8;
        for i in 0..batch {
            let mut a = [0u8; 8];
            a.copy_from_slice(&pwd[i * 8..8 + i * 8]);
            r.push(a);
        }
        let mut b = [0u8; 8];
        for i in 0..m {
            b[i] = pwd[8 * batch + i];
        }
        for item in b.iter_mut().skip(m) {
            *item = (8 - m) as u8;
        }
        r.push(b);
        // #[cfg(debug_assertions)]
        // println!("{r:?}");
        r
    }
    let key = b"u2oh6Vu^".to_owned();
    let key = GenericArray::from(key);
    let des = Des::new(&key);
    let 填充分块后的密码 = pkcs7填充(密码文本);
    let mut 加密后的数据块 = Vec::new();
    for 块 in 填充分块后的密码 {
        let mut 块 = GenericArray::from(块);
        des.encrypt_block(&mut 块);
        let mut 块 = 块.to_vec();
        加密后的数据块.append(&mut 块);
    }
    hex::encode(加密后的数据块)
}

pub fn 获取unicode字符串定宽显示时应当设置的宽度(
    s: &str,
    希望显示的宽度: usize,
) -> usize {
    if UnicodeWidthStr::width(s) > 希望显示的宽度 {
        希望显示的宽度
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
