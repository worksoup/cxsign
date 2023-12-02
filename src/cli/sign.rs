use std::{collections::HashMap, path::PathBuf};

use futures::{stream::FuturesUnordered, StreamExt};

use crate::{
    activity::sign::{SignActivity, SignState},
    session::SignSession,
    utils::{address::Address, photo::Photo},
};

use super::single_sign;

pub async fn general_sign_<'a>(
    sign: &SignActivity,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::general_sign_single(sign, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}

pub async fn photo_sign_<'a>(
    sign: &SignActivity,
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
        tasks.push(single_sign::photo_sign_single(sign, &photo, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}
pub async fn qrcode_sign_<'a>(
    sign: &SignActivity,
    c: &str,
    enc: &str,
    poss: &Vec<Address>,
    sessions: &'a Vec<&SignSession>,
    no_random_shift: bool,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::qrcode_sign_single(
            sign,
            c,
            enc,
            poss,
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

pub async fn location_sign_<'a>(
    sign: &SignActivity,
    poss: &Vec<Address>,
    sessions: &'a Vec<&SignSession>,
    no_random_shift: bool,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::location_sign_single(
            sign,
            poss,
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

pub async fn signcode_sign_<'a>(
    sign: &SignActivity,
    signcode: &str,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::signcode_sign_single(sign, signcode, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}
