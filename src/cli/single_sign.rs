use crate::activity::sign::{SignActivity, SignState};
use crate::utils::address::根据位置及范围获取随机偏移后的位置;
use crate::{
    session::SignSession,
    utils::{address::Struct位置, photo::Photo},
};

pub async fn 拍照签到_单个账号<'a>(
    sign: &SignActivity,
    photo: &Photo,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => sign.sign_by_photo(photo, session).await?,
        },
    ))
}

pub async fn 通用签到_单个账号<'a>(
    sign: &SignActivity,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => sign.sign_common(session).await?,
        },
    ))
}

pub async fn 二维码签到_单个账号<'a>(
    sign: &SignActivity,
    c: &str,
    enc: &str,
    pos_vec: &Vec<Struct位置>,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match if sign.is_refresh_qrcode() {
            sign.pre_sign(session).await?
        } else {
            sign.pre_sign_for_refresh_qrcode_sign(c, enc, session)
                .await?
        } {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => {
                let mut state = SignState::Fail("所有位置均不可用".into());
                for pos in pos_vec {
                    match sign.sign_by_qrcode(enc, pos, session).await? {
                        r @ SignState::Success => {
                            state = r;
                            break;
                        }
                        SignState::Fail(msg) => {
                            eprintln!(
                                "用户[{}]在二维码签到[{}]中尝试位置[{:?}]时失败！失败信息：[{:?}]",
                                session.get_用户真名(),
                                sign.name,
                                pos,
                                msg
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
    sign: &SignActivity,
    poss: &Vec<Struct位置>,
    auto_fetch_pos: bool,
    session: &'a SignSession,
    no_random_shift: bool,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(msg) => {
                if auto_fetch_pos && let Some(pos) = crate::utils::address::在html文本中寻找位置及范围(&msg) {
                    println!(
                        "用户[{}]已获取到教师指定的签到位置：{}, 要求范围：{} 米，将使用随机偏移后的位置签到。",
                        session.get_用户真名(),
                        pos.位置,
                        pos.范围
                    );
                    let pos = if no_random_shift {
                        pos.位置
                    } else {
                        根据位置及范围获取随机偏移后的位置(pos)
                    };
                    println!("用户[{}]签到使用位置：{}.", session.get_用户真名(), pos);
                    sign.sign_by_location(&pos, session).await?
                } else {
                    let mut state = SignState::Fail("所有位置均不可用".into());
                    for pos in poss {
                        match sign.sign_by_location(pos, session).await? {
                            r @ SignState::Success => {
                                state = r;
                                break;
                            }
                            SignState::Fail(msg) => {
                                eprintln!(
                                    "用户[{}]在位置签到[{}]中尝试位置[{:?}]时失败！失败信息：[{:?}]",
                                    session.get_用户真名(),
                                    sign.name,
                                    pos,
                                    msg
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
    sign: &SignActivity,
    signcode: &str,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_用户真名(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => sign.sign_by_signcode(session, signcode).await?,
        },
    ))
}
