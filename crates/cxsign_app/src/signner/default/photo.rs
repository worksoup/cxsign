use crate::SignnerTrait;
use cxsign_activity::sign::{PhotoSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_types::Photo;
use cxsign_user::Session;
use log::warn;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct DefaultPhotoSignner {
    path: Option<PathBuf>,
}

impl DefaultPhotoSignner {
    pub fn new(path: &Option<PathBuf>) -> Self {
        let path = if let Some(pic) = path
            && let Ok(metadata) = std::fs::metadata(pic)
        {
            if metadata.is_dir() {
                crate::utils::pic_dir_or_path_to_pic_path(pic).unwrap_or(None)
            } else {
                Some(pic.to_owned())
            }
        } else {
            None
        };
        Self { path }
    }
}
impl SignnerTrait<PhotoSign> for DefaultPhotoSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut PhotoSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let mut pic_map = HashMap::new();
        let mut session_to_index = HashMap::new();
        if let Some(pic) = self.path.as_ref() {
            for session in sessions.clone() {
                let photo = Photo::get_from_file(session, pic);
                pic_map.insert(0, photo);
                for session in sessions.clone() {
                    session_to_index.insert(session, 0);
                }
            }
        } else {
            let mut index = 0;
            for session in sessions.clone() {
                let photo = Photo::default(session);
                session_to_index.insert(session, index);
                if let Some(photo) = photo {
                    pic_map.insert(index, photo);
                    index += 1;
                } else {
                    warn!(
                        "用户[{}]在拍照签到时未能获取到照片，将尝试使用其他用户的照片！",
                        session.get_stu_name(),
                    );
                }
            }
        }
        let mut map = HashMap::new();
        for session in sessions {
            let index = session_to_index[session];
            if let Some(photo) = pic_map.get(&index).cloned() {
                sign.set_photo(photo);
                let a = Self::sign_single(sign, session, ())?;
                map.insert(session, a);
            } else {
                map.insert(
                    session,
                    SignResult::Fail {
                        msg: format!("拍照签到[{}]没有获取到有效的照片！", sign.as_inner().name),
                    },
                );
            }
        }
        Ok(map)
    }

    fn sign_single(sign: &mut PhotoSign, session: &Session, _: ()) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session).map_err(|e| e.into())
    }
}
