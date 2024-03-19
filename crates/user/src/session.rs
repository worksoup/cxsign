use crate::{cookies::UserCookies, protocol};
use dir::Dir;
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
    pub fn load_json(dir: &Dir, account: &str) -> Result<Self, ureq::Error> {
        let client = login::load_json(dir.get_json_file_path(account));
        let cookies = UserCookies::new(&client);
        let stu_name = Self::find_stu_name_in_html(&client)?;
        println!("用户[{}]加载 Cookies 成功！", stu_name);
        Ok(Session {
            agent: client,
            stu_name,
            cookies,
        })
    }

    pub fn login(dir: &Dir, account: &str, enc_passwd: &str) -> Result<Session, ureq::Error> {
        let client = login::login_enc(account, enc_passwd, Some(dir.get_json_file_path(account)));
        let cookies = UserCookies::new(&client);
        let stu_name = Self::find_stu_name_in_html(&client)?;
        println!("用户[{}]登录成功！", stu_name);
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
}

impl Deref for Session {
    type Target = Agent;
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
