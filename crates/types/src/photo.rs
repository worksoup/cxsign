use serde::Deserialize;
use std::fs::File;
use std::path::Path;
use user::session::Session;

// TODO: 删除 unwrap
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Photo {
    object_id: String,
}

impl Photo {
    pub fn get_pan_token(session: &Session) -> Result<String, ureq::Error> {
        let r = pan::protocol::pan_token(session)?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(alias = "_token")]
            token: String,
        }
        let r: Tmp = r.into_json().unwrap();
        Ok(r.token)
    }

    pub fn new(session: &Session, file: &File, file_name: &str) -> Result<Self, ureq::Error> {
        let token = Self::get_pan_token(session)?;
        let r = pan::protocol::pan_upload(session, file, session.get_uid(), &token, file_name)?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(alias = "objectId")]
            object_id: String,
        }
        let tmp: Tmp = r.into_json().unwrap();
        Ok(Self {
            object_id: tmp.object_id,
        })
    }
    pub fn get_object_id(&self) -> &str {
        &self.object_id
    }
    pub fn default(session: &Session) -> Option<Self> {
        Self::find_in_cxpan(session, |a| a == "1.png" || a == "1.jpg").unwrap()
    }
    pub fn find_in_cxpan(
        session: &Session,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<Self>, ureq::Error> {
        let r = pan::protocol::pan_chaoxing(session)?;
        let r_text = r.into_string().unwrap();
        let start_of_enc = r_text.find("enc =\"").unwrap() + 6;
        let end_of_enc = r_text[start_of_enc..r_text.len()].find('"').unwrap() + start_of_enc;
        let enc = &r_text[start_of_enc..end_of_enc];
        let start_of_root_dir = r_text.find("_rootdir = \"").unwrap() + 12;
        let end_of_root_dir =
            r_text[start_of_root_dir..r_text.len()].find('"').unwrap() + start_of_root_dir;
        let parent_id = &r_text[start_of_root_dir..end_of_root_dir];
        let r = pan::protocol::pan_list(session, parent_id, enc)?;
        #[derive(Deserialize)]
        struct CloudFile {
            name: String,
            #[serde(alias = "objectId")]
            object_id: Option<String>,
        }
        #[derive(Deserialize)]
        struct TmpR {
            list: Vec<CloudFile>,
        }
        let r: TmpR = r.into_json()?;
        for item in r.list {
            if p(&item.name) {
                return Ok(item.object_id.map(|object_id| Self { object_id }));
            }
        }
        Ok(None)
    }
    pub fn get_from_file(session: &Session, file_path: impl AsRef<Path>) -> Self {
        let f = File::open(&file_path).unwrap();
        let file_name = file_path.as_ref().file_name().unwrap().to_str().unwrap();
        Self::new(session, &f, file_name).unwrap()
    }
}
