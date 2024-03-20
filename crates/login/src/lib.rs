use anyhow::Result;
use std::path::Path;
use ureq::{Agent, AgentBuilder};

pub mod protocol;
pub mod utils;
pub static UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 (schild:eaf4fb193ec970c0a9775e2a27b0232b) (device:iPhone11,2) Language/zh-Hans com.ssreader.ChaoXingStudy/ChaoXingStudy_3_6.0.2_ios_phone_202209281930_99 (@Kalimdor)_1665876591620212942";

pub fn login_enc<P: AsRef<Path>>(
    account: &str,
    enc_passwd: &str,
    store_path: Option<P>,
) -> Result<Agent> {
    let cookie_store = cookie_store::CookieStore::new(None);
    let client = AgentBuilder::new()
        .user_agent(UA)
        .cookie_store(cookie_store)
        .build();
    let response = crate::protocol::login_enc(&client, account, enc_passwd).unwrap();
    /// TODO: 存疑
    #[derive(serde::Deserialize)]
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
    } = response.into_json().unwrap();
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
    if let Some(store_path) = store_path {
        // Write store back to disk
        let mut writer = std::fs::File::create(store_path)
            .map(std::io::BufWriter::new)
            .unwrap();
        client.cookie_store().save_json(&mut writer).unwrap();
    }
    Ok(client)
}

pub fn load_json<P: AsRef<std::path::Path>>(cookies_file: P) -> Agent {
    let cookie_store = {
        let file = std::fs::File::open(cookies_file)
            .map(std::io::BufReader::new)
            .unwrap();
        cookie_store::CookieStore::load_json(file).unwrap()
    };
    AgentBuilder::new()
        .user_agent(UA)
        .cookie_store(cookie_store)
        .build()
}

#[cfg(test)]
mod tests {}
