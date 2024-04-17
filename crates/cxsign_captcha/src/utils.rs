use crate::hash::{encode, hash, uuid};

pub fn generate_secrets(
    captcha_id: &str,
    server_time_stamp_mills: u128,
    r#type: &str,
) -> (String, String) {
    let captcha_key = encode(hash(&(server_time_stamp_mills.to_string() + &uuid())));
    let tmp_token = encode(hash(
        &(server_time_stamp_mills.to_string() + captcha_id + r#type + &captcha_key),
    ));
    let tmp_token =
        tmp_token + "%3A" + (server_time_stamp_mills + 300000_u128).to_string().as_str();
    (captcha_key, tmp_token)
}
#[cfg(test)]
mod tests {
    use crate::protocol::{check_captcha, get_captcha, CAPTCHA_ID};
    use crate::utils::generate_secrets;
    #[test]
    fn generate_secrets_test() {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        println!("{time}");
        let k = generate_secrets(CAPTCHA_ID, time, "slide");
        let agent = ureq::Agent::new();
        let r = get_captcha(&agent, CAPTCHA_ID, &k.0, &k.1, time).unwrap();
        println!("{}", r.into_string().unwrap());
    }
    #[test]
    fn check_captcha_test() {
        let agent = ureq::Agent::new();
        let r = check_captcha(&agent, CAPTCHA_ID, 21231258, "TOKEN", 1121213321).unwrap();
        println!("{}", r.into_string().unwrap());
    }
}
