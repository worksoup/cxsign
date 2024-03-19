use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
// 重构完成之前使用 `cxsign` 的配置文件夹。
lazy_static! {
    static ref CONFIG_DIR: PathBuf = {
        let is_testing = std::env::var("TEST_CXSIGN").is_ok();
        let binding = directories::ProjectDirs::from("rt.lea", "worksoup", "cxsign").unwrap();
        let dir = if is_testing {
            binding.config_dir().join("test").to_owned()
        } else {
            binding.config_dir().to_owned()
        };
        let _ = std::fs::create_dir_all(dir.clone());
        dir
    };
    pub static ref DIR: Dir = Dir::new(CONFIG_DIR.as_path());
}

#[derive(Clone)]
pub struct Dir {
    base_dir: PathBuf,
    database_dir: PathBuf,
}
impl Dir {
    pub fn new(base_dir: &Path) -> Self {
        let base_dir = base_dir.to_path_buf();
        let database_dir = base_dir.join("cx.db");
        Self {
            base_dir,
            database_dir,
        }
    }
    pub fn get_config_dir(&self) -> PathBuf {
        self.base_dir.to_path_buf()
    }
    pub fn get_database_dir(&self) -> PathBuf {
        self.database_dir.to_path_buf()
    }
    pub fn get_json_file_path(&self, account: &str) -> PathBuf {
        self.base_dir.join(account.to_string() + ".json")
    }
}
