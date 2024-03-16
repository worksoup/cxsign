use std::path::Path;
use ureq::{Agent, Response};

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
