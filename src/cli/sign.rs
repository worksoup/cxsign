use std::{collections::HashMap, path::PathBuf};

use futures::{stream::FuturesUnordered, StreamExt};

use crate::{
    activity::sign::{Enum签到结果, Struct签到},
    session::Struct签到会话,
    utils::{address::Struct位置, photo::Struct在线图片},
};

use super::single_sign;

//noinspection ALL
pub async fn 普通签到<'a>(
    签到: &Struct签到,
    签到会话列表: &'a Vec<&Struct签到会话>,
) -> Result<HashMap<&'a str, Enum签到结果>, reqwest::Error> {
    let mut 用户真名_签到结果_哈希表 = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for 签到会话 in 签到会话列表 {
        tasks.push(single_sign::通用签到_单个账号(签到, 签到会话));
    }
    while let Some(tmp) = tasks.next().await {
        let (用户真名, 签到结果) = tmp?;
        用户真名_签到结果_哈希表.insert(用户真名, 签到结果);
    }
    Ok(用户真名_签到结果_哈希表)
}

pub async fn 拍照签到<'a>(
    签到: &Struct签到,
    pic: &Option<PathBuf>,
    签到会话列表: &'a Vec<&Struct签到会话>,
) -> Result<HashMap<&'a str, Enum签到结果>, reqwest::Error> {
    let mut 用户真名_签到结果_哈希表 = HashMap::new();
    let mut 索引到在线图片 = HashMap::new();
    let mut 签到到索引 = HashMap::new();
    if let Some(pic) = &pic {
        let 在线图片 = Struct在线图片::上传文件获取(签到会话列表[0], pic).await;
        索引到在线图片.insert(0, 在线图片);
        for session in 签到会话列表 {
            签到到索引.insert(session, 0);
        }
    } else {
        let mut 索引 = 0;
        for 签到会话 in 签到会话列表 {
            let 在线图片 = Struct在线图片::默认(签到会话).await;
            if let Some(在线图片) = 在线图片 {
                索引到在线图片.insert(索引, 在线图片);
                索引 += 1;
            } else {
                签到到索引.insert(签到会话, 索引);
            }
        }
    }
    let mut tasks = FuturesUnordered::new();
    for 签到会话 in 签到会话列表 {
        tasks.push(single_sign::拍照签到_单个账号(
            签到,
            &索引到在线图片[&签到到索引[签到会话]],
            签到会话,
        ));
    }
    while let Some(tmp) = tasks.next().await {
        let (用户名称, 签到结果) = tmp?;
        用户真名_签到结果_哈希表.insert(用户名称, 签到结果);
    }
    Ok(用户真名_签到结果_哈希表)
}

pub async fn 二维码签到<'a>(
    签到: &Struct签到,
    c: &str,
    enc: &str,
    位置列表: &Vec<Struct位置>,
    签到会话列表: &'a Vec<&Struct签到会话>,
) -> Result<HashMap<&'a str, Enum签到结果>, reqwest::Error> {
    let mut 用户真名_签到结果_哈希表 = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for 签到会话 in 签到会话列表 {
        tasks.push(single_sign::二维码签到_单个账号(
            签到,
            c,
            enc,
            位置列表,
            签到会话,
        ));
    }
    while let Some(tmp) = tasks.next().await {
        let (用户真名, 签到结果) = tmp?;
        用户真名_签到结果_哈希表.insert(用户真名, 签到结果);
    }
    Ok(用户真名_签到结果_哈希表)
}

pub async fn 位置签到<'a>(
    签到: &Struct签到,
    位置列表: &Vec<Struct位置>,
    是否自动获取签到位置: bool,
    签到会话列表: &'a Vec<&Struct签到会话>,
    是否禁用随机偏移: bool,
) -> Result<HashMap<&'a str, Enum签到结果>, reqwest::Error> {
    let mut 用户真名_签到结果_哈希表 = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for 签到会话 in 签到会话列表 {
        tasks.push(single_sign::位置签到_单个账号(
            签到,
            位置列表,
            是否自动获取签到位置,
            签到会话,
            是否禁用随机偏移,
        ));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        用户真名_签到结果_哈希表.insert(name, state);
    }
    Ok(用户真名_签到结果_哈希表)
}

pub async fn 签到码签到<'a>(
    签到: &Struct签到,
    签到码: &str,
    签到会话列表: &'a Vec<&Struct签到会话>,
) -> Result<HashMap<&'a str, Enum签到结果>, reqwest::Error> {
    let mut 用户真名_签到结果_哈希表 = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for 签到会话 in 签到会话列表 {
        tasks.push(single_sign::签到码签到_单个账号(
            签到,
            签到码,
            签到会话,
        ));
    }
    while let Some(tmp) = tasks.next().await {
        let (用户真名, 签到结果) = tmp?;
        用户真名_签到结果_哈希表.insert(用户真名, 签到结果);
    }
    Ok(用户真名_签到结果_哈希表)
}
