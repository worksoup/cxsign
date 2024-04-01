pub mod cookies;
pub mod course;

use crate::activity::{
    sign::Struct签到,
    {Enum活动, Struct非签到活动},
};
use crate::protocol;
use crate::protocol::UA;
use crate::session::course::{GetCoursesR, Struct课程};
use crate::utils::配置文件夹;
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
pub struct Struct签到会话 {
    client: Client,
    用户真名: String,
    cookies: UserCookies,
}

impl PartialEq for Struct签到会话 {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Struct签到会话 {}

impl Hash for Struct签到会话 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_uid().hash(state);
        self.get_fid().hash(state);
        self.get_用户真名().hash(state);
    }
}

impl Struct签到会话 {
    pub async fn 从cookies文件加载<P: AsRef<std::path::Path>>(
        cookies路径: P,
    ) -> Result<Self, reqwest::Error> {
        let cookie_store = {
            let cookies文件 = std::fs::File::open(cookies路径)
                .map(std::io::BufReader::new)
                .unwrap();
            reqwest_cookie_store::CookieStore::load_json(cookies文件).unwrap()
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
        let 用户真名 = Self::获取用户真名(&client).await?;
        println!("用户[{}]加载 Cookies 成功！", 用户真名);
        Ok(Struct签到会话 {
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

    pub async fn 通过账号密码登录(
        账号: &str,
        加密过的密码: &str,
    ) -> Result<Struct签到会话, reqwest::Error> {
        let cookie_store = reqwest_cookie_store::CookieStore::new(None);
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = ClientBuilder::new()
            .user_agent(UA)
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()?;
        let response = protocol::login_enc(&client, 账号, 加密过的密码).await?;
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
            let mut writer = std::fs::File::create(配置文件夹.join(账号.to_string() + ".json"))
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
        let 用户真名 = Self::获取用户真名(&client).await?;
        println!("用户[{}]登录成功！", 用户真名);
        Ok(Struct签到会话 {
            client,
            用户真名,
            cookies,
        })
    }

    pub async fn 获取课程列表(&self) -> Result<Vec<Struct课程>, reqwest::Error> {
        // let r = protocol::back_clazz_data(self.deref()).await?;
        // std::fs::write("C:\\Users\\15102\\Desktop\\1.json", format!("{}",r.text().await.unwrap())).unwrap();
        let r = protocol::back_clazz_data(self.deref()).await?;
        let r: GetCoursesR = r.json().await.unwrap();
        let mut arr = Vec::new();
        for c in r.channel_list {
            if let Some(data) = c.content.课程 {
                for course in data.data {
                    if c.班级号.is_i64() {
                        arr.push(Struct课程::new(
                            course.课程号,
                            c.班级号.as_i64().unwrap(),
                            course.任课教师.as_str(),
                            course.封面图url.unwrap_or("".into()).as_str(),
                            course.课程名.as_str(),
                        ))
                    }
                }
            }
        }
        println!("用户[{}]已获取课程列表。", self.用户真名);
        Ok(arr)
    }
    async fn 获取用户真名(client: &Client) -> Result<String, reqwest::Error> {
        let r = protocol::account_manage(client).await?;
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
    pub async fn 获取网盘token(&self) -> Result<String, reqwest::Error> {
        let r = protocol::pan_token(self).await?;
        #[derive(Deserialize)]
        struct Tmp {
            _token: String,
        }
        let r: Tmp = r.json().await.unwrap();
        Ok(r._token)
    }

    pub async fn 上传在线图片(
        &self,
        buffer: Vec<u8>,
        file_name: &str,
    ) -> Result<String, reqwest::Error> {
        let token = self.获取网盘token().await?;
        let r = protocol::pan_upload(self, buffer, self.get_uid(), &token, file_name).await?;
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct Tmp {
            objectId: String,
        }
        let tmp: Tmp = r.json().await.unwrap();
        Ok(tmp.objectId)
    }
}

impl Struct签到会话 {
    pub async fn 遍历课程以获取所有活动(
        &self,
    ) -> Result<(Vec<Struct签到>, Vec<Struct签到>, Vec<Struct非签到活动>), reqwest::Error> {
        let mut 有效签到列表 = Vec::new();
        let mut 其他签到列表 = Vec::new();
        let mut 非签到活动列表 = Vec::new();
        let mut tasks = FuturesUnordered::new();
        let 课程列表 = self.获取课程列表().await?;
        for c in 课程列表 {
            tasks.push(Enum活动::获取课程的所有活动(self, c));
        }
        while let Some(item) = tasks.next().await {
            for a in item? {
                if let Enum活动::签到(签到) = a {
                    if 签到.是否有效() {
                        有效签到列表.push(签到);
                    } else {
                        其他签到列表.push(签到);
                    }
                } else if let Enum活动::非签到活动(非签到活动) = a {
                    非签到活动列表.push(非签到活动);
                }
            }
        }
        有效签到列表.sort_by(|a1, a2| -> Ordering { a1.开始时间戳.cmp(&a2.开始时间戳) });
        Ok((有效签到列表, 其他签到列表, 非签到活动列表))
    }
}

impl Deref for Struct签到会话 {
    type Target = Client;
    fn deref(&self) -> &Client {
        &self.client
    }
}
