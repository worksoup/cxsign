use crate::{session::SignSession, utils};
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
// TODO: 删除 unwrap
pub struct Photo {
    object_id: String,
}

impl Photo {
    async fn get_object_id_from_cx_pan(
        session: &SignSession,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<String>, reqwest::Error> {
        let r = utils::query::pan_chaoxing(session).await?;
        let data = r.text().await.unwrap();
        let start_of_enc = data.find("enc =\"").unwrap() + 6;
        let end_of_enc = data[start_of_enc..data.len()].find("\"").unwrap() + start_of_enc;
        let enc = &data[start_of_enc..end_of_enc];
        let start_of_root_dir = data.find("_rootdir = \"").unwrap() + 12;
        let end_of_root_dir =
            data[start_of_root_dir..data.len()].find("\"").unwrap() + start_of_root_dir;
        let parent_id = &data[start_of_root_dir..end_of_root_dir];
        let r = utils::query::pan_list(session, parent_id, enc).await?;
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct CloudFile {
            name: String,
            objectId: Option<String>,
        }
        #[derive(Deserialize)]
        struct TmpR {
            list: Vec<CloudFile>,
        }
        let r: TmpR = r.json().await?;
        for item in r.list {
            if p(&item.name) {
                return Ok(item.objectId);
            }
        }
        Ok(None)
    }
    pub fn get_object_id(&self) -> &str {
        &self.object_id
    }
    pub async fn new(session: &SignSession, buf: Vec<u8>, file_name: &str) -> Self {
        let object_id = session.upload_photo(buf, file_name).await.unwrap();
        Self { object_id }
    }
    pub async fn default(session: &SignSession) -> Self {
        Self::from_pan(session, |a| a == "1.png" || a == "1.jpg").await
    }
    pub async fn from_pan(session: &SignSession, p: impl Fn(&str) -> bool) -> Self {
        let object_id = Self::get_object_id_from_cx_pan(session, p)
            .await
            .unwrap()
            .unwrap();
        Self { object_id }
    }
    pub async fn from_file(session: &SignSession, path_buf: &PathBuf) -> Self {
        let mut f = File::open(path_buf).unwrap();
        let file_name = path_buf.file_name().unwrap().to_str().unwrap();
        let size = f.metadata().unwrap().len() as usize;
        let mut buffer = vec![0u8; size];
        let _ = f.read(&mut buffer).unwrap();
        Self::new(session, buffer, file_name).await
    }
}
