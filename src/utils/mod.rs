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

pub fn inquire_confirm(message: &str, help_message: &str) -> bool {
    inquire::Confirm::new(message)
        .with_help_message(help_message)
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

pub fn encrypto_pwd(pwd: &str) -> String {
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
