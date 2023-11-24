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
use std::{
    collections::{hash_map::OccupiedError, HashMap},
    fs::DirEntry,
    path::PathBuf,
};
use unicode_width::UnicodeWidthStr;

use crate::sign_session::{activity::sign::SignActivity, session::SignSession};

use self::sql::DataBase;
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

// 添加账号。TODO: 跳过输入密码阶段
pub async fn add_account(db: &DataBase, uname: String, pwd: Option<String>) {
    let pwd = if let Some(pwd) = pwd {
        pwd
    } else {
        inquire::Password::new("密码：")
            .without_confirmation()
            .prompt()
            .unwrap()
    };
    let enc_pwd = crate::utils::encrypto_pwd(&pwd);
    let session = SignSession::login_enc(&uname, &enc_pwd).await.unwrap();
    let name = session.get_stu_name();
    db.add_account_or(&uname, &enc_pwd, name, DataBase::update_account);
    let courses = session.get_courses().await.unwrap();
    for c in courses {
        db.add_course_or(&c, |_, _| {});
    }
}
// 添加账号（刷新时用，此时密码一定是存在的且为加密后的密码）。
pub async fn add_account_enc(db: &DataBase, uname: String, enc_pwd: &str) {
    let session = SignSession::login_enc(&uname, &enc_pwd).await.unwrap();
    let name = session.get_stu_name();
    db.add_account_or(&uname, &enc_pwd, name, DataBase::update_account);
    let courses = session.get_courses().await.unwrap();
    for c in courses {
        db.add_course_or(&c, |_, _| {});
    }
}

pub async fn get_sessions(db: &DataBase) -> HashMap<String, SignSession> {
    let accounts = db.get_accounts();
    let config_dir = crate::utils::CONFIG_DIR.clone();
    let mut s = HashMap::new();
    for a in accounts {
        let cookies_dir = config_dir.join(a.0.to_string() + ".json");
        let session = SignSession::load(cookies_dir).await.unwrap();
        s.insert(a.0, session);
    }
    s
}

pub async fn get_signs<'a>(
    sessions: &'a HashMap<String, SignSession>,
) -> (
    HashMap<SignActivity, Vec<&'a SignSession>>,
    HashMap<SignActivity, Vec<&'a SignSession>>,
) {
    let mut asigns = HashMap::new();
    let mut osigns = HashMap::new();
    for (_, session) in sessions {
        let (available_sign_activities, other_sign_activities, _) =
            session.traverse_activities().await.unwrap();
        for sa in available_sign_activities {
            let vec = vec![session];
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = asigns.try_insert(sa, vec)
            {
                entry.get_mut().push(session);
            }
        }
        for sa in other_sign_activities {
            let vec = vec![session];
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = osigns.try_insert(sa, vec)
            {
                entry.get_mut().push(session);
            }
        }
    }
    (asigns, osigns)
}
pub fn handle_qrcode_pic_path(pic_path: &str) -> String {
    let results = rxing::helpers::detect_multiple_in_file(pic_path).expect("decodes");
    let r = &results[0];
    let r = r.getText();
    let beg = r.find("&enc=").unwrap();
    let enc = &r[beg + 5..beg + 37];
    // 在二维码图片中会有一个参数 `c`, 二维码预签到时需要。
    // 但是该参数似乎暂时可以从 `signDetail` 接口获取到。所以此处先注释掉。
    // let beg = r.find("&c=").unwrap();
    // let c = &r[beg + 3..beg + 9];
    // (c.to_owned(), enc.to_owned())
    enc.to_owned()
}
// mod test {
//     #[test]
//     fn test_des() {
//         println!("{}", crate::utils::pwd_des("0123456789."));
//     }
// }
