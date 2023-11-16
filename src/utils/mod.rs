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
use std::path::PathBuf;
lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = {
        let binding = ProjectDirs::from("rt.lea", "worksoup", "cxsign").unwrap();
        let dir = binding.config_dir().to_owned();
        let _ = std::fs::create_dir_all(dir.clone());
        dir
    };
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

// mod test {
//     #[test]
//     fn test_des() {
//         println!("{}", crate::utils::pwd_des("0123456789."));
//     }
// }
