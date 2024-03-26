use crate::multipart::{Field, PreparedFields};
use std::fs::File;
use std::path::Path;
use ureq::{Agent, Response};

// 超星网盘页
static PAN_CHAOXING: &str = "https://pan-yz.chaoxing.com";

pub fn pan_chaoxing(client: &Agent) -> Result<Response, ureq::Error> {
    client.get(PAN_CHAOXING).call()
}

// 网盘列表
static PAN_LIST: &str = "https://pan-yz.chaoxing.com/opt/listres";

pub fn pan_list(client: &Agent, parent_id: &str, enc: &str) -> Result<Response, ureq::Error> {
    client
        .post(&format!(
            "{PAN_LIST}?puid=0&shareid=0&parentId={parent_id}&page=1&size=50&enc={enc}"
        ))
        .call()
}

// 获取超星云盘的 token
static PAN_TOKEN: &str = "https://pan-yz.chaoxing.com/api/token/uservalid";

pub fn pan_token(client: &Agent) -> Result<Response, ureq::Error> {
    client.get(PAN_TOKEN).call()
}

// 网盘上传接口
static PAN_UPLOAD: &str = "https://pan-yz.chaoxing.com/upload";

pub fn pan_upload(
    client: &Agent,
    file: &File,
    uid: &str,
    token: &str,
    file_name: &str,
) -> Result<Response, ureq::Error> {
    let file_ext: &Path = file_name.as_ref();
    let file_ext = file_ext.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mime = mime_guess::from_ext(&file_ext).first_or_octet_stream();
    let mut fields = Vec::<Field>::default();
    Field::add_stream(&mut fields, "file", file, Some(file_name), Some(mime));
    Field::add_text(&mut fields, "puid", uid);
    let multipart = PreparedFields::from_fields(&mut fields).unwrap();
    client
        .post(&format!("{PAN_UPLOAD}?_from=mobilelearn&_token={token}"))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", multipart.get_boundary()),
        )
        .send(multipart)
}
