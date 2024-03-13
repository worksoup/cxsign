use reqwest::{Client, Response};

// 网盘列表
static PAN_LIST: &str = "https://pan-yz.chaoxing.com/opt/listres";

pub async fn pan_list(
    client: &Client,
    parent_id: &str,
    enc: &str,
) -> Result<Response, reqwest::Error> {
    let url = PAN_LIST;
    let url = format!("{url}?puid=0&shareid=0&parentId={parent_id}&page=1&size=50&enc={enc}");
    client.post(url).send().await
}