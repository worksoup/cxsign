use crate::{
    sign_session::{
        activity::sign::{SignActivity, SignState},
        session::SignSession,
    },
    utils::{address::Address, photo::Photo},
};

pub async fn photo_sign_single<'a>(
    sign: &SignActivity,
    photo: &Photo,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_stu_name(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => sign.sign_by_photo(&photo, session).await?,
        },
    ))
}

pub async fn general_sign_single<'a>(
    sign: &SignActivity,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_stu_name(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => sign.sign_common(session).await?,
        },
    ))
}

pub async fn qrcode_sign_single<'a>(
    sign: &SignActivity,
    c: &str,
    enc: &str,
    poss: &Vec<Address>,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_stu_name(),
        match if sign.is_refresh_qrcode() {
            sign.pre_sign(session).await?
        } else {
            sign.pre_sign_for_refresh_qrcode_sign(c, enc, session)
                .await?
        } {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => {
                let mut state = SignState::Fail("所有位置均不可用".into());
                for pos in poss {
                    match sign.sign_by_qrcode(enc, &pos, session).await? {
                        r @ SignState::Success => {
                            state = r;
                            break;
                        }
                        SignState::Fail(msg) => {
                            eprintln!(
                                "用户[{}]在二维码签到[{}]中尝试位置[{:?}]时失败！失败信息：[{:?}]",
                                session.get_stu_name(),
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

pub async fn location_sign_single<'a>(
    sign: &SignActivity,
    poss: &Vec<Address>,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_stu_name(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => {
                let mut state = SignState::Fail("所有位置均不可用".into());
                for pos in poss {
                    match sign.sign_by_location(&pos, session).await? {
                        r @ SignState::Success => {
                            state = r;
                            break;
                        }
                        SignState::Fail(msg) => {
                            eprintln!(
                                "用户[{}]在位置签到[{}]中尝试位置[{:?}]时失败！失败信息：[{:?}]",
                                session.get_stu_name(),
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

pub async fn signcode_sign_single<'a>(
    sign: &SignActivity,
    signcode: &str,
    session: &'a SignSession,
) -> Result<(&'a str, SignState), reqwest::Error> {
    Ok((
        session.get_stu_name(),
        match sign.pre_sign(session).await? {
            SignState::Success => SignState::Success,
            SignState::Fail(_) => sign.sign_by_signcode(session, signcode).await?,
        },
    ))
}
