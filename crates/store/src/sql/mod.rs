mod account_table;
mod alias_table;
mod course_table;
mod location_table;

pub use account_table::*;
pub use alias_table::*;
pub use course_table::*;
pub use location_table::*;

use sqlite::Connection;
use std::fs::File;
use std::ops::Deref;

pub trait DataBaseTableTrait<'a> {
    const TABLE_ARGS: &'static str;
    const TABLE_NAME: &'static str;
    fn from_ref(db: &'a DataBase) -> Self;
    fn create(db: &DataBase) {
        if !Self::is_existed(db) {
            db.execute(format!(
                "CREATE TABLE {} ({});",
                Self::TABLE_NAME,
                Self::TABLE_ARGS
            ))
            .unwrap();
        }
    }
    fn is_existed(db: &DataBase) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{}';",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }
    fn delete(db: &DataBase) {
        let mut query = db
            .prepare(format!("DELETE FROM {};", Self::TABLE_NAME))
            .unwrap();
        query.next().unwrap();
        println!("已删除数据表 {}。", Self::TABLE_NAME);
    }
}

pub struct DataBase {
    connection: Connection,
}
impl Deref for DataBase {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
// self
impl DataBase {
    pub fn new() -> Self {
        let db_dir = base::get_database_dir();
        if db_dir.metadata().is_err() {
            File::create(db_dir.clone()).unwrap();
        }
        let connection = Connection::open(db_dir.to_str().unwrap()).unwrap();
        let db = Self { connection };
        db
    }
    pub fn add_table<'a, T: DataBaseTableTrait<'a>>(&'a self) -> T {
        T::create(self);
        T::from_ref(self)
    }
}
