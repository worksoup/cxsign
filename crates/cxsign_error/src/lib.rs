use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("登录失败！")]
    LoginError(String),
    #[error(transparent)]
    AgentError(#[from] ureq::Error),
}
