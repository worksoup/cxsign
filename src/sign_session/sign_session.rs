use crate::sign_session::activity::activity::{
    Activity, GetActivityR, OtherActivity, SignActivity,
};
use crate::sign_session::course::{Course, GetCoursesR};
use crate::utils::{self, CONFIG_DIR};
use cookie_store::Cookie;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::{Client, ClientBuilder};
use serde_derive::Deserialize;
use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::{Deref, Index};

#[allow(non_snake_case, unused)]
#[derive(Debug)]
pub struct UserCookies {
    JSESSIONID: String,
    lv: String,
    fid: String,
    _uid: String,
    uf: String,
    _d: String,
    UID: String,
    vc: String,
    vc2: String,
    vc3: String,
    cx_p_token: String,
    p_auth_token: String,
    xxtenc: String,
    DSSTASH_LOG: String,
    route: String,
}

impl UserCookies {
    #[allow(non_snake_case)]
    fn new_(
        JSESSIONID: &str,
        lv: &str,
        fid: &str,
        _uid: &str,
        uf: &str,
        _d: &str,
        UID: &str,
        vc: &str,
        vc2: &str,
        vc3: &str,
        cx_p_token: &str,
        p_auth_token: &str,
        xxtenc: &str,
        DSSTASH_LOG: &str,
        route: &str,
    ) -> Self {
        UserCookies {
            JSESSIONID: JSESSIONID.into(),
            lv: lv.into(),
            fid: fid.into(),
            _uid: _uid.into(),
            uf: uf.into(),
            _d: _d.into(),
            UID: UID.into(),
            vc: vc.into(),
            vc2: vc2.into(),
            vc3: vc3.into(),
            cx_p_token: cx_p_token.into(),
            p_auth_token: p_auth_token.into(),
            xxtenc: xxtenc.into(),
            DSSTASH_LOG: DSSTASH_LOG.into(),
            route: route.into(),
        }
    }
    #[allow(non_snake_case)]
    pub fn new<'a>(cookies: Vec<Cookie<'a>>) -> Self {
        let mut JSESSIONID = String::new();
        let mut lv = String::new();
        let mut fid = String::new();
        let mut _uid = String::new();
        let mut uf = String::new();
        let mut _d = String::new();
        let mut UID = String::new();
        let mut vc = String::new();
        let mut vc2 = String::new();
        let mut vc3 = String::new();
        let mut cx_p_token = String::new();
        let mut p_auth_token = String::new();
        let mut xxtenc = String::new();
        let mut DSSTASH_LOG = String::new();
        let mut route = String::new();
        for c in cookies {
            match c.name() {
                "JSESSIONID" => {
                    JSESSIONID = c.value().into();
                }
                "lv" => {
                    lv = c.value().into();
                }
                "fid" => {
                    fid = c.value().into();
                }
                "_uid" => {
                    _uid = c.value().into();
                }
                "uf" => {
                    uf = c.value().into();
                }
                "_d" => {
                    _d = c.value().into();
                }
                "UID" => {
                    UID = c.value().into();
                }
                "vc" => {
                    vc = c.value().into();
                }
                "vc2" => {
                    vc2 = c.value().into();
                }
                "vc3" => {
                    vc3 = c.value().into();
                }
                "cx_p_token" => {
                    cx_p_token = c.value().into();
                }
                "p_auth_token" => {
                    p_auth_token = c.value().into();
                }
                "xxtenc" => {
                    xxtenc = c.value().into();
                }
                "DSSTASH_LOG" => {
                    DSSTASH_LOG = c.value().into();
                }
                "route" => {
                    route = c.value().into();
                }
                _ => {
                    JSESSIONID = c.value().into();
                }
            }
        }
        UserCookies {
            JSESSIONID,
            lv,
            fid,
            _uid,
            uf,
            _d,
            UID,
            vc,
            vc2,
            vc3,
            cx_p_token,
            p_auth_token,
            xxtenc,
            DSSTASH_LOG,
            route,
        }
    }
}

impl Default for UserCookies {
    fn default() -> Self {
        Self::new_("", "", "-1", "", "", "", "", "", "", "", "", "", "", "", "")
    }
}

#[derive(Debug)]
pub struct SignSession {
    client: Client,
    stu_name: String,
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
        self.get_stu_name().hash(state);
    }
}
#[derive(Deserialize)]
struct LoginR {
    mes: String,
    status: bool,
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
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()
            .unwrap();
        let stu_name = Self::get_stu_name_(&client).await?;
        Ok(SignSession {
            client,
            stu_name,
            cookies,
        })
    }
    pub fn get_uid(&self) -> &str {
        &self.cookies._uid
    }
    pub fn get_fid(&self) -> &str {
        &self.cookies.fid
    }

    pub fn get_stu_name(&self) -> &str {
        &self.stu_name
    }

    pub async fn login(uname: &str, pwd: &str) -> Result<SignSession, reqwest::Error> {
        let cookie_store = reqwest_cookie_store::CookieStore::new(None);
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = ClientBuilder::new()
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()?;
        let response = utils::api::login(&client, uname, pwd).await?;
        let LoginR { mes, status } = response.json().await.unwrap();
        if status{
            println!("{mes}");
        }else{
            panic!("{mes}");
        }
        {
            // Write store back to disk
            let mut writer = std::fs::File::create(CONFIG_DIR.join(uname.to_string() + ".json"))
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
        // println!("{:?}", response.text().await.unwrap());
        let stu_name = Self::get_stu_name_(&client).await?;
        Ok(SignSession {
            client,
            stu_name,
            cookies,
        })
    }

    #[allow(unused)]
    pub async fn login_enc(uname: &str, enc_pwd: &str) -> Result<SignSession, reqwest::Error> {
        let cookie_store = reqwest_cookie_store::CookieStore::new(None);
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = ClientBuilder::new()
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()?;
        let response = utils::api::login_enc(&client, uname, enc_pwd).await?;
        /// TODO: 存疑
        let LoginR { mes, status } = response.json().await.unwrap();
        if status{
            println!("{mes}");
        }else{
            panic!("{mes}");
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
        let stu_name = Self::get_stu_name_(&client).await?;
        Ok(SignSession {
            client,
            stu_name,
            cookies,
        })
    }

    pub async fn get_courses(&self) -> Result<Vec<Course>, reqwest::Error> {
        let r = utils::api::back_clazz_data(self.deref()).await?;
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
        Ok(arr)
    }
    async fn get_stu_name_(client: &Client) -> Result<String, reqwest::Error> {
        let r = utils::api::account_manage(client).await?;
        let html_content = r.text().await?;
        let e = html_content.find("messageName").unwrap() + 20;
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let name = html_content.index(0..html_content.find('"').unwrap());
        Ok(name.to_owned())
    }
    pub async fn get_pan_token(&self) -> Result<String, reqwest::Error> {
        let r = utils::api::pan_token(self).await?;
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
        let r = utils::api::pan_upload(self, buffer, self.get_uid(), &token, file_name).await?;
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct Tmp {
            objectId: String,
        }
        let tmp: Tmp = r.json().await.unwrap();
        Ok(tmp.objectId)
    }
    pub async fn get_object_id_from_cx_pan(
        &self,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<String>, reqwest::Error> {
        let r = utils::api::pan_chaoxing(self).await?;
        let data = r.text().await.unwrap();
        let start_of_enc = data.find("enc =\"").unwrap() + 6;
        let end_of_enc = data[start_of_enc..data.len()].find("\"").unwrap() + start_of_enc;
        let enc = &data[start_of_enc..end_of_enc];
        let start_of_root_dir = data.find("_rootdir = \"").unwrap() + 12;
        let end_of_root_dir =
            data[start_of_root_dir..data.len()].find("\"").unwrap() + start_of_root_dir;
        let parent_id = &data[start_of_root_dir..end_of_root_dir];
        let r = utils::api::pan_list(self, parent_id, enc).await?;
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
}

impl SignSession {
    async fn get_all_activities(&self, c: Course) -> Result<Vec<Activity>, reqwest::Error> {
        let r = utils::api::active_list(self, c.clone()).await?;
        let r: GetActivityR = r.json().await.unwrap();
        let mut arr = Vec::new();
        if let Some(data) = r.data {
            for ar in data.activeList {
                if let Some(other_id) = ar.otherId {
                    let other_id_i64: i64 = other_id.parse().unwrap();
                    if other_id_i64 >= 0 && other_id_i64 <= 5 {
                        arr.push(Activity::Sign(SignActivity {
                            id: ar.id.to_string(),
                            name: ar.nameOne,
                            course: c.clone(),
                            other_id,
                            status: ar.status,
                            start_time_secs: (ar.startTime / 1000) as i64,
                        }))
                    } else {
                        arr.push(Activity::Other(OtherActivity {
                            id: ar.id.to_string(),
                            name: ar.nameOne,
                            course: c.clone(),
                            status: ar.status,
                            start_time_secs: (ar.startTime / 1000) as i64,
                        }))
                    }
                } else {
                    arr.push(Activity::Other(OtherActivity {
                        id: ar.id.to_string(),
                        name: ar.nameOne,
                        course: c.clone(),
                        status: ar.status,
                        start_time_secs: (ar.startTime / 1000) as i64,
                    }))
                }
            }
        }
        Ok(arr)
    }
    pub async fn traverse_activities(
        &self,
    ) -> Result<(Vec<SignActivity>, Vec<SignActivity>, Vec<OtherActivity>), reqwest::Error> {
        let courses = self.get_courses().await?;
        let mut available_sign_activities = Vec::new();
        let mut other_sign_activities = Vec::new();
        let mut other_activities = Vec::new();
        let mut tasks = FuturesUnordered::new();
        for c in courses {
            tasks.push(self.get_all_activities(c));
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
