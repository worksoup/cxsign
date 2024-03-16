use crate::protocol;
use crate::user::session::Session;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

// TODO: 删除 unwrap
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Photo {
    object_id: String,
}

impl Photo {
    fn find_object_id(
        session: &Session,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<String>, ureq::Error> {
        let r = protocol::pan_chaoxing(session)?;
        let r_text = r.into_string().unwrap();
        let start_of_enc = r_text.find("enc =\"").unwrap() + 6;
        let end_of_enc = r_text[start_of_enc..r_text.len()].find('"').unwrap() + start_of_enc;
        let enc = &r_text[start_of_enc..end_of_enc];
        let start_of_root_dir = r_text.find("_rootdir = \"").unwrap() + 12;
        let end_of_root_dir =
            r_text[start_of_root_dir..r_text.len()].find('"').unwrap() + start_of_root_dir;
        let parent_id = &r_text[start_of_root_dir..end_of_root_dir];
        let r = protocol::pan_list(session, parent_id, enc)?;
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
        let r: TmpR = r.into_json()?;
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
    pub fn new(session: &Session, file: &File, file_name: &str) -> Self {
        let object_id = session.upload_image(file, file_name).unwrap();
        Self { object_id }
    }
    pub fn default(session: &Session) -> Option<Self> {
        Self::get_from_cxpan(session, |a| a == "1.png" || a == "1.jpg")
    }
    pub fn get_from_cxpan(session: &Session, p: impl Fn(&str) -> bool) -> Option<Self> {
        let object_id = Self::find_object_id(session, p).unwrap();
        if let Some(object_id) = object_id {
            Some(Self { object_id })
        } else {
            None
        }
    }
    pub fn get_from_file(session: &Session, file_path: impl AsRef<Path>) -> Self {
        let f = File::open(&file_path).unwrap();
        let file_name = file_path.as_ref().file_name().unwrap().to_str().unwrap();
        Self::new(session, &f, file_name)
    }
}
