use crate::activity::sign::{Enum签到结果, Struct签到};
use crate::utils::address::根据位置及范围获取随机偏移后的位置;
use crate::{
    session::Struct签到会话,
    utils::{address::Struct位置, photo::Struct在线图片},
};

pub async fn 拍照签到_单个账号<'a>(
    sign: &Struct签到,
    photo: &Struct在线图片,
    session: &'a Struct签到会话,
) -> Result<(&'a str, Enum签到结果), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match sign.预签到(session).await? {
            Enum签到结果::成功 => Enum签到结果::成功,
            Enum签到结果::失败 { 失败信息: _ } => {
                sign.作为拍照签到处理(&photo, session).await?
            }
        },
    ))
}

pub async fn 通用签到_单个账号<'a>(
    sign: &Struct签到,
    session: &'a Struct签到会话,
) -> Result<(&'a str, Enum签到结果), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match sign.预签到(session).await? {
            Enum签到结果::成功 => Enum签到结果::成功,
            Enum签到结果::失败 { 失败信息: _ } => sign.作为普通签到处理(session).await?,
        },
    ))
}

pub async fn 二维码签到_单个账号<'a>(
    sign: &Struct签到,
    c: &str,
    enc: &str,
    pos_vec: &Vec<Struct位置>,
    session: &'a Struct签到会话,
) -> Result<(&'a str, Enum签到结果), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match if sign.二维码是否刷新() {
            sign.预签到(session).await?
        } else {
            sign.预签到_对于有刷新二维码签到(c, enc, session).await?
        } {
            Enum签到结果::成功 => Enum签到结果::成功,
            Enum签到结果::失败 { 失败信息: _ } => {
                let mut state = Enum签到结果::失败 {
                    失败信息: "所有位置均不可用".into(),
                };
                for pos in pos_vec {
                    match sign.作为二维码签到处理(enc, pos, session).await? {
                        r @ Enum签到结果::成功 => {
                            state = r;
                            break;
                        }
                        Enum签到结果::失败 { 失败信息 } => {
                            eprintln!(
                                "用户[{}]在二维码签到[{}]中尝试位置[{}]时失败！失败信息：[{:?}]",
                                session.get_用户真名(),
                                sign.签到名,
                                pos,
                                失败信息
                            );
                        }
                    };
                }
                state
            }
        },
    ))
}

pub async fn 位置签到_单个账号<'a>(
    签到: &Struct签到,
    位置列表: &Vec<Struct位置>,
    是否自动获取签到位置: bool,
    签到会话: &'a Struct签到会话,
    是否禁用随机偏移: bool,
) -> Result<(&'a str, Enum签到结果), reqwest::Error> {
    Ok((
        签到会话.get_用户真名(),
        match 签到.预签到(签到会话).await? {
            Enum签到结果::成功 => Enum签到结果::成功,
            Enum签到结果::失败 { 失败信息 } => {
                if 是否自动获取签到位置
                    && let Some(pos) =
                        crate::utils::address::在html文本中寻找位置及范围(&失败信息)
                {
                    println!(
                        "用户[{}]已获取到教师指定的签到位置：{}, 要求范围：{} 米，将使用随机偏移后的位置签到。",
                        签到会话.get_用户真名(),
                        pos.位置,
                        pos.范围
                    );
                    let pos = if 是否禁用随机偏移 {
                        pos.位置
                    } else {
                        根据位置及范围获取随机偏移后的位置(pos)
                    };
                    println!("用户[{}]签到使用位置：{}.", 签到会话.get_用户真名(), pos);
                    签到.作为位置签到处理(&pos, 签到会话).await?
                } else {
                    let mut state = Enum签到结果::失败 {
                        失败信息: "所有位置均不可用".into(),
                    };
                    for pos in 位置列表 {
                        match 签到.作为位置签到处理(pos, 签到会话).await? {
                            r @ Enum签到结果::成功 => {
                                state = r;
                                break;
                            }
                            Enum签到结果::失败 { 失败信息 } => {
                                eprintln!(
                                    "用户[{}]在位置签到[{}]中尝试位置[{}]时失败！失败信息：[{:?}]",
                                    签到会话.get_用户真名(),
                                    签到.签到名,
                                    pos,
                                    失败信息
                                );
                            }
                        };
                    }
                    state
                }
            }
        },
    ))
}

pub async fn 签到码签到_单个账号<'a>(
    sign: &Struct签到,
    signcode: &str,
    session: &'a Struct签到会话,
) -> Result<(&'a str, Enum签到结果), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match sign.预签到(session).await? {
            Enum签到结果::成功 => Enum签到结果::成功,
            Enum签到结果::失败 { 失败信息: _ } => {
                sign.作为签到码签到处理(session, signcode).await?
            }
        },
    ))
}
