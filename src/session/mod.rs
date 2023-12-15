pub mod cookies;
pub mod course;

use crate::activity::{
    sign::SignActivity,
    {Activity, OtherActivity},
};
use crate::session::course::{Course, GetCoursesR};
use crate::utils::{self, query::UA, 配置文件夹};
use cookies::UserCookies;
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::{Client, ClientBuilder};
use serde_derive::Deserialize;
use std::{
    cmp::Ordering,
    hash::Hash,
    ops::{Deref, Index},
};

#[derive(Debug)]
pub struct SignSession {
    client: Client,
    用户真名: String,
    cookies: UserCookies,
}

impl PartialEq for SignSession {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}
impl Eq for SignSession {}

impl Hash for SignSession {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_uid().hash(state);
        self.get_fid().hash(state);
        self.get_用户真名().hash(state);
    }
}

impl SignSession {
    pub async fn load<P: AsRef<std::path::Path>>(cookies_dir: P) -> Result<Self, reqwest::Error> {
        let cookie_store = {
            let file = std::fs::File::open(cookies_dir)
                .map(std::io::BufReader::new)
                .unwrap();
            // use re-exported version of `CookieStore` for crate compatibility
            reqwest_cookie_store::CookieStore::load_json(file).unwrap()
        };
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let cookies = {
            let store = cookie_store.lock().unwrap();
            let mut cookies = Vec::new();
            for c in store.iter_any() {
                cookies.push(c.to_owned())
            }
            cookies
        };
        let cookies = UserCookies::new(cookies);
        let client = Client::builder()
            .user_agent(UA)
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()
            .unwrap();
        let 用户真名 = Self::get_stu_name_(&client).await?;
        println!("用户[{用户真名}]加载 Cookies 成功！");
        Ok(SignSession {
            client,
            用户真名,
            cookies,
        })
    }
    pub fn get_uid(&self) -> &str {
        self.cookies.get_uid()
    }
    pub fn get_fid(&self) -> &str {
        self.cookies.get_fid()
    }

    pub fn get_用户真名(&self) -> &str {
        &self.用户真名
    }

    pub async fn login(uname: &str, enc_pwd: &str) -> Result<SignSession, reqwest::Error> {
        let cookie_store = reqwest_cookie_store::CookieStore::new(None);
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = ClientBuilder::new()
            .user_agent(UA)
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()?;
        let response = utils::query::login_enc(&client, uname, enc_pwd).await?;
        /// TODO: 存疑
        #[derive(Deserialize)]
        struct LoginR {
            url: Option<String>,
            msg1: Option<String>,
            msg2: Option<String>,
            status: bool,
        }
        let LoginR {
            status,
            url,
            msg1,
            msg2,
        } = response.json().await.unwrap();
        let mut mes = Vec::new();
        if let Some(url) = url {
            mes.push(url);
        }
        if let Some(msg1) = msg1 {
            mes.push(msg1);
        }
        if let Some(msg2) = msg2 {
            mes.push(msg2);
        }
        if !status {
            for mes in mes {
                eprintln!("{mes:?}");
            }
            panic!("登录失败！");
        }
        {
            // Write store back to disk
            let mut writer = std::fs::File::create(配置文件夹.join(uname.to_string() + ".json"))
                .map(std::io::BufWriter::new)
                .unwrap();
            let store = cookie_store.lock().unwrap();
            store.save_json(&mut writer).unwrap();
        }
        let store = {
            let s = cookie_store.clone();
            let s = s.lock().unwrap();
            let mut r = Vec::new();
            for s in s.iter_any() {
                r.push(s.to_owned());
            }
            r
        };
        let cookies = UserCookies::new(store);
        let 用户真名 = Self::get_stu_name_(&client).await?;
        println!("用户[{用户真名}]登录成功！");
        Ok(SignSession {
            client,
            用户真名,
            cookies,
        })
    }

    pub async fn get_courses(&self) -> Result<Vec<Course>, reqwest::Error> {
        let r = utils::query::back_clazz_data(self.deref()).await?;
        let r: GetCoursesR = r.json().await.unwrap();
        let mut arr = Vec::new();
        for c in r.channelList {
            if let Some(data) = c.content.course {
                for course in data.data {
                    if c.key.is_i64() {
                        arr.push(Course::new(
                            course.id,
                            c.key.as_i64().unwrap(),
                            course.teacherfactor.as_str(),
                            course.imageurl.as_str(),
                            course.name.as_str(),
                        ))
                    }
                }
            }
        }
        println!("用户[{}]已获取课程列表。", self.用户真名);
        Ok(arr)
    }
    async fn get_stu_name_(client: &Client) -> Result<String, reqwest::Error> {
        let r = utils::query::account_manage(client).await?;
        let html_content = r.text().await?;
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
    pub async fn get_pan_token(&self) -> Result<String, reqwest::Error> {
        let r = utils::query::pan_token(self).await?;
        #[derive(Deserialize)]
        struct Tmp {
            _token: String,
        }
        let r: Tmp = r.json().await.unwrap();
        Ok(r._token)
    }

    pub async fn upload_photo(
        &self,
        buffer: Vec<u8>,
        file_name: &str,
    ) -> Result<String, reqwest::Error> {
        let token = self.get_pan_token().await?;
        let r = utils::query::pan_upload(self, buffer, self.get_uid(), &token, file_name).await?;
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct Tmp {
            objectId: String,
        }
        let tmp: Tmp = r.json().await.unwrap();
        Ok(tmp.objectId)
    }
}

impl SignSession {
    pub async fn traverse_activities(
        &self,
    ) -> Result<(Vec<SignActivity>, Vec<SignActivity>, Vec<OtherActivity>), reqwest::Error> {
        let mut available_sign_activities = Vec::new();
        let mut other_sign_activities = Vec::new();
        let mut other_activities = Vec::new();
        let mut tasks = FuturesUnordered::new();
        let courses = self.get_courses().await?;
        for c in courses {
            tasks.push(Activity::from_course(self, c));
        }
        while let Some(item) = tasks.next().await {
            for a in item? {
                if let Activity::Sign(sa) = a {
                    if sa.is_available() {
                        available_sign_activities.push(sa);
                    } else {
                        other_sign_activities.push(sa);
                    }
                } else if let Activity::Other(oa) = a {
                    other_activities.push(oa);
                }
            }
        }
        available_sign_activities
            .sort_by(|a1, a2| -> Ordering { a1.start_time_secs.cmp(&a2.start_time_secs) });
        Ok((
            available_sign_activities,
            other_sign_activities,
            other_activities,
        ))
    }
}

impl Deref for SignSession {
    type Target = Client;
    fn deref(&self) -> &Client {
        &self.client
    }
}
