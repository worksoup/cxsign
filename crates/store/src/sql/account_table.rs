use user::session::Session;

use crate::sql::{DataBase, DataBaseTableTrait};
use std::collections::HashMap;
use log::warn;

pub struct AccountTable<'a> {
    db: &'a DataBase,
}

impl<'a> AccountTable<'a> {
    pub fn get_sessions_by_accounts_str(&self, accounts: &str) -> HashMap<String, Session> {
        let str_list = accounts.split(',').map(|a| a.trim()).collect::<Vec<&str>>();
        let mut s = HashMap::new();
        for account in str_list {
            if let Some(session) = self.get_session(account) {
                s.insert(account.to_string(), session);
            }
        }
        s
    }
    pub fn get_session(&self, account: &str) -> Option<Session> {
        if self.has_account(account) {
            Some(Session::load_json(&self.db.dir, account).unwrap())
        } else {
            None
        }
    }
    pub fn get_sessions(&self) -> HashMap<String, Session> {
        let binding = self.get_accounts();
        let str_list = binding.keys().collect::<Vec<_>>();
        let mut s = HashMap::new();
        for account in str_list {
            if self.has_account(account) {
                let session = Session::load_json(&self.db.dir, account).unwrap();
                s.insert(account.to_string(), session);
            }
        }
        s
    }
    pub fn has_account(&self, uname: &str) -> bool {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE uname=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, uname)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_account(&self, uname: &str) {
        if self.has_account(uname) {
            let mut query = self
                .db
                .prepare(format!("DELETE FROM {} WHERE uname=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, uname)).unwrap();
            query.next().unwrap();
        }
        std::fs::remove_file(self.db.dir.get_json_file_path(uname)).unwrap();
    }

    pub fn add_account_or<O: Fn(&Self, &str, &str, &str)>(
        &self,
        uname: &str,
        pwd: &str,
        name: &str,
        or: O,
    ) {
        let mut query = self
            .db
            .prepare(format!(
                "INSERT INTO {}(uname,pwd,name) values(:uname,:pwd,:name);",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":pwd", pwd.into()),
                    (":uname", uname.into()),
                    (":name", name.into()),
                ][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, uname, pwd, name),
        };
    }

    pub fn update_account(&self, uname: &str, pwd: &str, name: &str) {
        let mut query = self
            .db
            .prepare(format!(
                "UPDATE {} SET pwd=:pwd,name=:name WHERE uname=:uname;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":uname", uname.into()),
                    (":pwd", pwd.into()),
                    (":name", name.into()),
                ][..],
            )
            .unwrap();
        query.next().unwrap();
    }

    pub fn get_accounts(&self) -> HashMap<String, (String, String)> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut accounts = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let uname: &str = row.read("uname");
                let pwd: &str = row.read("pwd");
                let name: &str = row.read("name");
                accounts.insert(uname.into(), (pwd.into(), name.into()));
            } else {
                warn!("账号解析行出错：{c:?}.");
            }
        }
        accounts
    }
    pub fn get_account(&self, account: &str) -> Option<(String, (String, String))> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {} WHERE uname=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, account)).unwrap();
        for c in query.iter() {
            if let Ok(row) = c {
                let uname: &str = row.read("uname");
                let pwd: &str = row.read("pwd");
                let name: &str = row.read("name");
                return Some((uname.into(), (pwd.into(), name.into())));
            } else {
                warn!("账号解析行出错：{c:?}.");
            }
        }
        None
    }
}

impl<'a> DataBaseTableTrait<'a> for AccountTable<'a> {
    const TABLE_ARGS: &'static str =
        "uname CHAR (50) UNIQUE NOT NULL,pwd TEXT NOT NULL,name TEXT NOT NULL";
    const TABLE_NAME: &'static str = "account";

    fn from_ref(db: &'a DataBase) -> Self {
        Self { db }
    }
}
