use super::cookies::UserCookies;
use crate::course::Course;
use crate::protocol;
use crate::{
    activity::{
        sign::{Sign, SignTrait},
        Activity, OtherActivity,
    },
    get_json_file_path,
};
use serde::Deserialize;
use std::fs::File;
use std::{
    hash::Hash,
    ops::{Deref, Index},
};
use ureq::Agent;

#[derive(Debug)]
pub struct Session {
    agent: Agent,
    stu_name: String,
    cookies: UserCookies,
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Session {}

impl Hash for Session {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_uid().hash(state);
        self.get_fid().hash(state);
        self.get_stu_name().hash(state);
    }
}

impl Session {
    pub fn load_json(account: &str) -> Result<Self, ureq::Error> {
        let client = login::load_json(get_json_file_path(account));
        let cookies = UserCookies::new(&client);
        let stu_name = Self::find_stu_name_in_html(&client)?;
        println!("用户[{}]加载 Cookies 成功！", stu_name);
        Ok(Session {
            agent: client,
            stu_name,
            cookies,
        })
    }
    pub fn get_uid(&self) -> &str {
        self.cookies.get_uid()
    }
    pub fn get_fid(&self) -> &str {
        self.cookies.get_fid()
    }

    pub fn get_stu_name(&self) -> &str {
        &self.stu_name
    }

    pub fn login(account: &str, enc_passwd: &str) -> Result<Session, ureq::Error> {
        let client = login::login_enc(account, enc_passwd, Some(get_json_file_path(account)));
        let cookies = UserCookies::new(&client);
        let stu_name = Self::find_stu_name_in_html(&client)?;
        println!("用户[{}]登录成功！", stu_name);
        Ok(Session {
            agent: client,
            stu_name,
            cookies,
        })
    }

    pub  fn get_courses(&self) -> Result<Vec<Course>, ureq::Error> {
        let r = protocol::back_clazz_data(self.deref())?;
        let courses = Course::get_list_from_response(r)?;
        println!("用户[{}]已获取课程列表。", self.stu_name);
        Ok(courses)
    }
    fn find_stu_name_in_html(client: &Agent) -> Result<String, ureq::Error> {
        let r = protocol::account_manage(client)?;
        let html_content = r.into_string().unwrap();
        #[cfg(debug_assertions)]
        println!("{html_content}");
        let e = html_content.find("colorBlue").unwrap();
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let e = html_content.find('>').unwrap() + 1;
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let name = html_content
            .index(0..html_content.find('<').unwrap())
            .trim();
        Ok(name.to_owned())
    }
    pub fn get_pan_token(&self) -> Result<String, ureq::Error> {
        let r = protocol::pan_token(self)?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(alias = "_token")]
            token: String,
        }
        let r: Tmp = r.into_json().unwrap();
        Ok(r.token)
    }

    pub fn upload_image(&self, file: &File, file_name: &str) -> Result<String, ureq::Error> {
        let token = self.get_pan_token()?;
        let r = protocol::pan_upload(self, file, self.get_uid(), &token, file_name)?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(alias = "objectId")]
            object_id: String,
        }
        let tmp: Tmp = r.into_json().unwrap();
        Ok(tmp.object_id)
    }
}

impl Session {
    pub fn get_all_activities(
        &self,
    ) -> Result<(Vec<Sign>, Vec<Sign>, Vec<OtherActivity>), ureq::Error> {
        let mut 有效签到列表 = Vec::new();
        let mut 其他签到列表 = Vec::new();
        let mut 非签到活动列表 = Vec::new();
        let 课程列表 = self.get_courses()?;
        for c in 课程列表 {
            let item = Activity::get_list_from_course(self, &c)?;
            for a in item {
                if let Activity::Sign(签到) = a {
                    if 签到.is_valid() {
                        有效签到列表.push(签到);
                    } else {
                        其他签到列表.push(签到);
                    }
                } else if let Activity::Other(非签到活动) = a {
                    非签到活动列表.push(非签到活动);
                }
            }
        }
        有效签到列表.sort();
        Ok((有效签到列表, 其他签到列表, 非签到活动列表))
    }
}

impl Deref for Session {
    type Target = Agent;
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
