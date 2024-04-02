use crate::sql::{DataBase, DataBaseTableTrait};
use log::warn;

pub struct ExcludeTable<'a> {
    db: &'a DataBase,
}

impl<'a> ExcludeTable<'a> {
    pub fn has_exclude(&self, id: i64) -> bool {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE id=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, id)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn get_excludes(&self) -> Vec<i64> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut excludes = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let id = row.read("id");
                excludes.push(id);
            } else {
                warn!("账号解析行出错：{c:?}.");
            }
        }
        excludes
    }

    pub fn add_exclude(&self, id: i64) {
        let mut query = self
            .db
            .prepare(format!("INSERT INTO {}(id) values(:id);", Self::TABLE_NAME))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(&[(":id", id.into())][..])
            .unwrap();
        let _ = query.next();
    }

    pub fn delete_exclude(&self, id: i64) {
        if self.has_exclude(id) {
            let mut query = self
                .db
                .prepare(format!("DELETE FROM {} WHERE id=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, id)).unwrap();
            query.next().unwrap();
        }
    }

    pub fn update_excludes(&self, excludes: &Vec<i64>) {
        Self::delete(self.db);
        for exclude in excludes {
            self.add_exclude(*exclude);
        }
    }
}

impl<'a> DataBaseTableTrait<'a> for ExcludeTable<'a> {
    const TABLE_ARGS: &'static str = "id UNIQUE NOT NULL";
    const TABLE_NAME: &'static str = "exclude";

    fn from_ref(db: &'a DataBase) -> Self {
        Self { db }
    }
}
