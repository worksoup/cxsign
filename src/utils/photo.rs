use crate::{session::Struct签到会话, utils};
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
// TODO: 删除 unwrap
pub struct Struct在线图片 {
    object_id: String,
}

impl Struct在线图片 {
    async fn 通过某种规则从网盘获取object_id(
        session: &Struct签到会话,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<String>, reqwest::Error> {
        let 响应 = utils::query::pan_chaoxing(session).await?;
        let 响应的文本 = 响应.text().await.unwrap();
        let start_of_enc = 响应的文本.find("enc =\"").unwrap() + 6;
        let end_of_enc = 响应的文本[start_of_enc..响应的文本.len()]
            .find('"')
            .unwrap()
            + start_of_enc;
        let enc = &响应的文本[start_of_enc..end_of_enc];
        let start_of_root_dir = 响应的文本.find("_rootdir = \"").unwrap() + 12;
        let end_of_root_dir = 响应的文本[start_of_root_dir..响应的文本.len()]
            .find('"')
            .unwrap()
            + start_of_root_dir;
        let parent_id = &响应的文本[start_of_root_dir..end_of_root_dir];
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
    pub async fn new(session: &Struct签到会话, buf: Vec<u8>, file_name: &str) -> Self {
        let object_id = session.上传在线图片(buf, file_name).await.unwrap();
        Self { object_id }
    }
    pub async fn 默认(session: &Struct签到会话) -> Option<Self> {
        Self::从网盘获取(session, |a| a == "1.png" || a == "1.jpg").await
    }
    pub async fn 从网盘获取(
        session: &Struct签到会话,
        p: impl Fn(&str) -> bool,
    ) -> Option<Self> {
        let object_id = Self::通过某种规则从网盘获取object_id(session, p)
            .await
            .unwrap();
        if let Some(object_id) = object_id {
            Some(Self { object_id })
        } else {
            None
        }
    }
    pub async fn 上传文件获取(session: &Struct签到会话, path_buf: &PathBuf) -> Self {
        let mut f = File::open(path_buf).unwrap();
        let file_name = path_buf.file_name().unwrap().to_str().unwrap();
        let size = f.metadata().unwrap().len() as usize;
        let mut buffer = vec![0u8; size];
        let _ = f.read(&mut buffer).unwrap();
        Self::new(session, buffer, file_name).await
    }
}
