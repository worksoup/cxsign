use reqwest::{Client, Response};

// 网盘上传接口
static PAN_UPLOAD: &str = "https://pan-yz.chaoxing.com/upload";

pub async fn pan_upload(
    client: &Client,
    buffer: Vec<u8>,
    uid: &str,
    token: &str,
    file_name: &str,
) -> Result<Response, reqwest::Error> {
    let part = reqwest::multipart::Part::bytes(buffer).file_name(file_name.to_string());
    let form_data = reqwest::multipart::Form::new()
        .part("file", part)
        .text("puid", uid.to_string());
    let url = format!("{PAN_UPLOAD}?_from=mobilelearn&_token={token}");
    client.post(url).multipart(form_data).send().await
}