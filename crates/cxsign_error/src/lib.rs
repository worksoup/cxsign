use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("登录失败！")]
    LoginError(String),
    #[error(transparent)]
    AgentError(#[from] ureq::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("`enc` 为空！")]
    EncError(String),
}
