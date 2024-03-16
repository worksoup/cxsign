use ureq::{Agent, Response};

// 网盘列表
static PAN_LIST: &str = "https://pan-yz.chaoxing.com/opt/listres";

pub fn pan_list(client: &Agent, parent_id: &str, enc: &str) -> Result<Response, ureq::Error> {
    client
        .post(&format!(
            "{PAN_LIST}?puid=0&shareid=0&parentId={parent_id}&page=1&size=50&enc={enc}"
        ))
        .call()
}
