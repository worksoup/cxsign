use cookie_store::Cookie;
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct UserCookies {
    // JSESSIONID: String,
    // lv: String,
    // uf: String,
    // UID: String,
    // vc: String,
    // vc2: String,
    // vc3: String,
    // cx_p_token: String,
    // p_auth_token: String,
    // xxtenc: String,
    // DSSTASH_LOG: String,
    // route: String,
    // _d: String,
    fid: String,
    _uid: String,
}

impl UserCookies {
    #[allow(non_snake_case)]
    fn new_(
        // JSESSIONID: &str,
        // lv: &str,
        // uf: &str,
        // UID: &str,
        // vc: &str,
        // vc2: &str,
        // vc3: &str,
        // cx_p_token: &str,
        // p_auth_token: &str,
        // xxtenc: &str,
        // DSSTASH_LOG: &str,
        // route: &str,
        // _d: &str,
        fid: &str,
        _uid: &str,
    ) -> Self {
        UserCookies {
            // JSESSIONID: JSESSIONID.into(),
            // lv: lv.into(),
            // uf: uf.into(),
            // UID: UID.into(),
            // vc: vc.into(),
            // vc2: vc2.into(),
            // vc3: vc3.into(),
            // cx_p_token: cx_p_token.into(),
            // p_auth_token: p_auth_token.into(),
            // xxtenc: xxtenc.into(),
            // DSSTASH_LOG: DSSTASH_LOG.into(),
            // route: route.into(),
            // _d: _d.into(),
            fid: fid.into(),
            _uid: _uid.into(),
        }
    }
    #[allow(non_snake_case)]
    pub fn new(cookies: Vec<Cookie>) -> Self {
        // let mut JSESSIONID = String::new();
        // let mut lv = String::new();
        // let mut uf = String::new();
        // let mut UID = String::new();
        // let mut vc = String::new();
        // let mut vc2 = String::new();
        // let mut vc3 = String::new();
        // let mut cx_p_token = String::new();
        // let mut p_auth_token = String::new();
        // let mut xxtenc = String::new();
        // let mut DSSTASH_LOG = String::new();
        // let mut route = String::new();
        // let mut _d = String::new();
        let mut fid = String::new();
        let mut _uid = String::new();
        for c in cookies {
            match c.name() {
                // "JSESSIONID" => {
                //     JSESSIONID = c.value().into();
                // }
                // "lv" => {
                //     lv = c.value().into();
                // }
                // "uf" => {
                //     uf = c.value().into();
                // }
                // "UID" => {
                //     UID = c.value().into();
                // }
                // "vc" => {
                //     vc = c.value().into();
                // }
                // "vc2" => {
                //     vc2 = c.value().into();
                // }
                // "vc3" => {
                //     vc3 = c.value().into();
                // }
                // "cx_p_token" => {
                //     cx_p_token = c.value().into();
                // }
                // "p_auth_token" => {
                //     p_auth_token = c.value().into();
                // }
                // "xxtenc" => {
                //     xxtenc = c.value().into();
                // }
                // "DSSTASH_LOG" => {
                //     DSSTASH_LOG = c.value().into();
                // }
                // "route" => {
                //     route = c.value().into();
                // }
                // "_d" => {
                //     _d = c.value().into();
                // }
                "fid" => {
                    fid = c.value().into();
                }
                "_uid" => {
                    _uid = c.value().into();
                }
                _ => {
                    fid = c.value().into();
                }
            }
        }
        UserCookies {
            // JSESSIONID,
            // lv,
            // uf,
            // UID,
            // vc,
            // vc2,
            // vc3,
            // cx_p_token,
            // p_auth_token,
            // xxtenc,
            // DSSTASH_LOG,
            // route,
            // _d,
            fid,
            _uid,
        }
    }
    pub fn get_uid(&self) -> &str {
        &self._uid
    }
    pub fn get_fid(&self) -> &str {
        &self.fid
    }
}

impl Default for UserCookies {
    fn default() -> Self {
        Self::new_("-1", "")
    }
}
