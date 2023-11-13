use crate::sign_session::sign_session::SignSession;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub enum SignType {
    // 拍照签到
    Photo,
    // 普通签到
    Common,
    // 二维码签到
    QrCode,
    // 手势签到
    Gesture,
    // 位置签到
    Location,
    // 签到码签到
    SignCode,
    // 未知
    Unknown,
}


pub struct Photo {
    object_id: String,
}

impl Photo {
    pub fn get_object_id(&self) -> &str {
        &self.object_id
    }
    pub async fn new(session: &SignSession, buf: Vec<u8>, file_name: &str) -> Self {
        let object_id = session
            .upload_photo(buf, file_name)
            .await
            .unwrap();
        Self { object_id }
    }
    pub async fn default(session: &SignSession) -> Self {
        Self::from_pan(session, |a| a == "1.png" || a == "1.jpg").await
    }
    pub async fn from_pan(session: &SignSession, p: impl Fn(&str) -> bool) -> Self {
        let object_id = session
            .get_object_id_from_cx_pan(p)
            .await
            .unwrap()
            .unwrap();
        Self { object_id }
    }
    pub async fn from_file(session: &SignSession, path_buf: PathBuf) -> Self {
        let mut f = File::open(&path_buf).unwrap();
        let file_name = path_buf.file_name().unwrap().to_str().unwrap();
        let size = f.metadata().unwrap().len() as usize;
        let mut buffer = vec![0u8; size];
        let _ = f.read(&mut buffer).unwrap();
        Self::new(session, buffer, file_name).await
    }
}

pub trait SignTrait {
    async fn sign(&self) -> Result<(), reqwest::Error>;
}
