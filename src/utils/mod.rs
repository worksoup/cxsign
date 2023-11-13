use std::path::PathBuf;

use directories::ProjectDirs;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = {
        let binding = ProjectDirs::from("rt.lea", "worksoup", "cxsign").unwrap();
        let dir = binding.config_dir().to_owned();
        let _ = std::fs::create_dir_all(dir.clone());
        dir
    };
}
pub mod api;
pub mod sql;
