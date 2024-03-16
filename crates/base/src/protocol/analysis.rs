use ureq::{Agent, Response};

// analysis
static ANALYSIS: &str = "https://mobilelearn.chaoxing.com/pptSign/analysis";

pub fn analysis(client: &Agent, active_id: &str) -> Result<Response, ureq::Error> {
    let url = ANALYSIS;
    let url = format!("{url}?vs=1&DB_STRATEGY=RANDOM&aid={active_id}");
    client.get(&url).call()
}

// analysis 2
static ANALYSIS2: &str = "https://mobilelearn.chaoxing.com/pptSign/analysis2";

pub fn analysis2(client: &Agent, code: &str) -> Result<Response, ureq::Error> {
    let url = ANALYSIS2;
    let url = format!("{url}?DB_STRATEGY=RANDOM&code={code}");
    client.get(&url).call()
}
