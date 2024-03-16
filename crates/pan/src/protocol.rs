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
    file: &std::fs::File,
    uid: &str,
    token: &str,
    file_name: &str,
) -> Result<Response, ureq::Error> {
    let file_ext: &Path = file_name.as_ref();
    let file_ext = file_ext.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mime = mime_guess::from_ext(&file_ext).first_or_octet_stream();
    let mut multipart = multipart::client::lazy::Multipart::new();
    multipart.add_stream("file", file, Some(file_name), Some(mime));
    multipart.add_text("puid", uid);
    let multipart = multipart.prepare().unwrap();
    client
        .post(&format!("{PAN_UPLOAD}?_from=mobilelearn&_token={token}"))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", multipart.boundary()),
        )
        .send(multipart)
}
