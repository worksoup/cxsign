use reqwest::{Client, Response};

// analysis
static ANALYSIS: &str = "https://mobilelearn.chaoxing.com/pptSign/analysis";

pub async fn analysis(client: &Client, active_id: &str) -> Result<Response, reqwest::Error> {
    let url = ANALYSIS;
    let url = format!("{url}?vs=1&DB_STRATEGY=RANDOM&aid={active_id}");
    client.get(url).send().await
}

// analysis 2
static ANALYSIS2: &str = "https://mobilelearn.chaoxing.com/pptSign/analysis2";

pub async fn analysis2(client: &Client, code: &str) -> Result<Response, reqwest::Error> {
    let url = ANALYSIS2;
    let url = format!("{url}?DB_STRATEGY=RANDOM&code={code}");
    client.get(url).send().await
}