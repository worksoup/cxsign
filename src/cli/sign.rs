use std::{collections::HashMap, path::PathBuf};

use futures::{stream::FuturesUnordered, StreamExt};

use crate::{
    activity::sign::{Struct签到, SignState},
    session::SignSession,
    utils::{address::Struct位置, photo::Photo},
};

use super::single_sign;

pub async fn 普通签到<'a>(
    sign: &Struct签到,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::通用签到_单个账号(sign, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}

pub async fn 拍照签到<'a>(
    sign: &Struct签到,
    pic: &Option<PathBuf>,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let photo = if let Some(pic) = &pic {
        Photo::from_file(sessions[0], pic).await
    } else {
        Photo::default(sessions[0]).await
    };
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::拍照签到_单个账号(sign, &photo, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}
pub async fn 二维码签到<'a>(
    sign: &Struct签到,
    c: &str,
    enc: &str,
    pos_vec: &Vec<Struct位置>,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::二维码签到_单个账号(sign, c, enc, pos_vec, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}

pub async fn 位置签到<'a>(
    sign: &Struct签到,
    poss: &Vec<Struct位置>,
    auto_fetch_pos: bool,
    sessions: &'a Vec<&SignSession>,
    no_random_shift: bool,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::位置签到_单个账号(
            sign,
            poss,
            auto_fetch_pos,
            session,
            no_random_shift,
        ));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}

pub async fn 签到码签到<'a>(
    sign: &Struct签到,
    signcode: &str,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::签到码签到_单个账号(sign, signcode, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}
