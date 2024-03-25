use crate::sql::{DataBase, DataBaseTableTrait};
use log::{error, warn};

pub struct AliasTable<'a> {
    db: &'a DataBase,
}

impl<'a> AliasTable<'a> {
    pub fn has_alias(&self, alias: &str) -> bool {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE name=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, alias)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_alias(&self, alias: &str) {
        let mut query = self
            .db
            .prepare(format!("DELETE FROM {} WHERE name=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, alias)).unwrap();
        query.next().unwrap();
    }

    pub fn add_alias_or<O: Fn(&Self, &str, i64)>(&self, alias: &str, location_id: i64, or: O) {
        let mut query = self
            .db
            .prepare(format!(
                "INSERT INTO {}(name,lid) values(:name,:lid);",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[(":name", alias.into()), (":lid", location_id.into())][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, alias, location_id),
        };
    }
    pub fn update_alias(&self, alias: &str, location_id: i64) {
        let mut query = self
            .db
            .prepare(format!(
                "UPDATE {} SET name=:name,lid=:lid WHERE name=:name;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[(":name", alias.into()), (":lid", location_id.into())][..],
            )
            .unwrap();
        query.next().unwrap();
    }
    pub fn get_aliases(&self, location_id: i64) -> Vec<String> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {} WHERE lid=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, location_id)).unwrap();
        let mut aliases = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let name: &str = row.read("name");
                aliases.push(name.to_owned());
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        aliases
    }

    pub fn get_location_id(&self, alias: &str) -> Option<i64> {
        if self.has_alias(alias) {
            let mut query = self
                .db
                .prepare(format!("SELECT * FROM {} WHERE name=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, alias)).unwrap();
            let c: Vec<sqlite::Row> = query
                .iter()
                .filter_map(|e| if let Ok(e) = e { Some(e) } else { None })
                .collect();
            let row = &c[0];
            let location_id: i64 = row.read("lid");
            Some(location_id)
        } else {
            None
        }
    }
}

impl<'a> DataBaseTableTrait<'a> for AliasTable<'a> {
    const TABLE_ARGS: &'static str = "name CHAR (50) UNIQUE NOT NULL,lid INTEGER NOT NULL";
    const TABLE_NAME: &'static str = "alias";

    fn from_ref(db: &'a DataBase) -> Self {
        Self { db }
    }
}
