use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case, reason = "这是一个要求序列化为 json 的结构体，post 时要求的 json 数据格式没有采用蛇形命名法。")]
pub struct SignCookies {
    name: String,
    fid: String,
    pid: String,
    refer: String,
    _blank: String,
    t: bool,
    vc3: String,
    _uid: String,
    _d: String,
    uf: String,
    lv: String,
    UID: String,
}