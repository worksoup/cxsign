use crate::session::course::Struct课程;
use crate::utils::address::Struct位置;
use sqlite::Connection;
use std::{collections::HashMap, fs::File, ops::Deref};

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
        let db_dir = crate::utils::配置文件夹.join("cx.db");
        if db_dir.metadata().is_err() {
            File::create(db_dir.clone()).unwrap();
        }
        let connection = Connection::open(db_dir.to_str().unwrap()).unwrap();
        let db = Self { connection };
        db.create_table_account();
        db.create_table_course();
        db.创建表pos();
        db.create_table_alias();
        db
    }
}
// account
impl DataBase {
    const CREATE_ACCOUNT_SQL: &'static str =
        "CREATE TABLE account (uname CHAR (50) UNIQUE NOT NULL,pwd TEXT NOT NULL,name TEXT NOT NULL);";

    fn has_table_account(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='account';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }

    pub fn has_account(&self, uname: &str) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM account WHERE uname=?;")
            .unwrap();
        query.bind((1, uname)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_account(&self, uname: &str) {
        if self.has_account(uname) {
            let mut query = self
                .connection
                .prepare("DELETE FROM account WHERE uname=?;")
                .unwrap();
            query.bind((1, uname)).unwrap();
            query.next().unwrap();
        }
        std::fs::remove_file(super::配置文件夹.join(uname.to_string() + ".json")).unwrap();
    }

    pub fn add_account_or<O: Fn(&DataBase, &str, &str, &str)>(
        &self,
        uname: &str,
        pwd: &str,
        name: &str,
        or: O,
    ) {
        let mut query = self
            .connection
            .prepare("INSERT INTO account(uname,pwd,name) values(:uname,:pwd,:name);")
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
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, uname, pwd, name),
        };
    }

    pub fn update_account(&self, uname: &str, pwd: &str, name: &str) {
        let mut query = self
            .connection
            .prepare("UPDATE account SET pwd=:pwd,name=:name WHERE uname=:uname;")
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

    fn create_table_account(&self) {
        if !self.has_table_account() {
            self.connection.execute(Self::CREATE_ACCOUNT_SQL).unwrap();
        }
    }

    pub fn get_accounts(&self) -> HashMap<String, (String, String)> {
        let mut query = self.connection.prepare("SELECT * FROM account;").unwrap();
        let mut accounts = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let uname: &str = row.read("uname");
                let pwd: &str = row.read("pwd");
                let name: &str = row.read("name");
                accounts.insert(uname.into(), (pwd.into(), name.into()));
            } else {
                eprintln!("账号解析行出错：{c:?}.");
            }
        }
        accounts
    }
}
// course
impl DataBase {
    const CREATE_COURSE_SQL: &'static str ="CREATE TABLE course (id INTEGER UNIQUE NOT NULL,clazzid INTEGER NOT NULL,name TEXT NOT NULL,teacher TEXT NOT NULL,image TEXT NOT NULL);";

    fn has_table_course(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='course';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }
    pub fn add_course_or<O: Fn(&DataBase, &Struct课程)>(&self, course: &Struct课程, or: O) {
        let id: i64 = course.get_课程号();
        let clazzid: i64 = course.get_班级号();
        let name: &str = course.get_课程名();
        let teacher: &str = course.get_任课教师();
        let image: &str = course.get_封面图url();
        let mut query =self.connection.prepare("INSERT INTO course(id,clazzid,name,teacher,image) values(:id,:clazzid,:name,:teacher,:image);").unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":id", id.into()),
                    (":clazzid", clazzid.into()),
                    (":name", name.into()),
                    (":teacher", teacher.into()),
                    (":image", image.into()),
                ][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, course),
        }
    }
    pub fn delete_all_course(&self) {
        let mut query = self.connection.prepare("DELETE FROM course;").unwrap();
        query.next().unwrap();
        println!("已删除旧的课程信息。");
    }
    fn create_table_course(&self) {
        if !self.has_table_course() {
            self.connection.execute(Self::CREATE_COURSE_SQL).unwrap();
        }
    }
    pub fn get_courses(&self) -> HashMap<i64, Struct课程> {
        let mut query = self.connection.prepare("SELECT * FROM course;").unwrap();
        let mut courses = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let id = row.read("id");
                let clazzid = row.read("clazzid");
                let teacher = row.read::<&str, _>("teacher");
                let image = row.read::<&str, _>("image");
                let name = row.read::<&str, _>("name");
                courses.insert(id, Struct课程::new(id, clazzid, teacher, image, name));
            } else {
                eprintln!("课程解析行出错：{c:?}.");
            }
        }
        courses
    }
}
// 位置
impl DataBase {
    const CREATE_POS_SQL: &'static str ="CREATE TABLE pos(posid INTEGER UNIQUE NOT NULL,courseid INTEGER NOT NULL,addr TEXT NOT NULL,lon TEXT NOT NULL,lat TEXT NOT NULL,alt TEXT NOT NULL);";

    fn 是否存在表pos(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='pos';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }
    pub fn 是否存在为某id的位置(&self, 位置id: i64) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM pos WHERE posid=?;")
            .unwrap();
        query.bind((1, 位置id)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }
    pub fn 添加位置_失败后则<O: Fn(&DataBase, i64, i64, &Struct位置)>(
        &self,
        位置id: i64,
        course_id: i64,
        位置: &Struct位置,
        or: O,
    ) {
        let addr = 位置.get_地址();
        let lat = 位置.get_纬度();
        let lon = 位置.get_经度();
        let alt = 位置.get_海拔();
        let mut query =self.connection.prepare("INSERT INTO pos(posid,courseid,addr,lat,lon,alt) values(:posid,:courseid,:addr,:lat,:lon,:alt);").unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":posid", 位置id.into()),
                    (":courseid", course_id.into()),
                    (":addr", addr.into()),
                    (":lat", lat.into()),
                    (":lon", lon.into()),
                    (":alt", alt.into()),
                ][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, 位置id, course_id, 位置),
        }
    }
    pub fn 删除为某id的位置(&self, 位置id: i64) {
        self.connection
            .execute("DELETE FROM pos WHERE posid=".to_string() + 位置id.to_string().as_str() + ";")
            .unwrap();
        let aliases = self.get_aliases(位置id);
        for alias in aliases {
            self.delete_alias(&alias)
        }
    }
    // pub fn 删除所有位置(&self) {
    //     self.connection.execute("DELETE FROM pos;").unwrap();
    //     self.删除所有别名();
    // }
    fn 创建表pos(&self) {
        if !self.是否存在表pos() {
            self.connection.execute(Self::CREATE_POS_SQL).unwrap();
        }
    }
    pub fn 获取所有位置(&self) -> HashMap<i64, (i64, Struct位置)> {
        let mut query = self.connection.prepare("SELECT * FROM pos;").unwrap();
        let mut 位置列表 = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let 位置id = row.read("posid");
                let 地址 = row.read("addr");
                let 纬度 = row.read("lat");
                let 经度 = row.read("lon");
                let 海拔 = row.read("alt");
                let 课程号 = row.read("courseid");
                位置列表.insert(位置id, (课程号, Struct位置::new(地址, 经度, 纬度, 海拔)));
            } else {
                eprintln!("位置解析行出错：{c:?}.");
            }
        }
        位置列表
    }
    pub fn 获取为某id的位置(&self, 位置id: i64) -> (i64, Struct位置) {
        let mut query = self
            .connection
            .prepare("SELECT * FROM pos WHERE posid=?;")
            .unwrap();
        query.bind((1, 位置id)).unwrap();
        let c: Vec<sqlite::Row> = query
            .iter()
            .filter_map(|e| if let Ok(e) = e { Some(e) } else { None })
            .collect();
        let row = &c[0];
        let addr = row.read("addr");
        let lat = row.read("lat");
        let lon = row.read("lon");
        let alt = row.read("alt");
        let courseid = row.read("courseid");
        (courseid, Struct位置::new(addr, lon, lat, alt))
    }
    pub fn 获取特定课程的位置和其id(&self, course_id: i64) -> HashMap<i64, Struct位置> {
        let mut query = self
            .connection
            .prepare("SELECT * FROM pos WHERE courseid=?;")
            .unwrap();
        query.bind((1, course_id)).unwrap();
        let mut 位置列表 = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let 位置id = row.read("posid");
                let 地址 = row.read("addr");
                let 纬度 = row.read("lat");
                let 经度 = row.read("lon");
                let 海拔 = row.read("alt");
                位置列表.insert(位置id, Struct位置::new(地址, 经度, 纬度, 海拔));
            } else {
                eprintln!("位置解析行出错：{c:?}.");
            }
        }
        位置列表
    }
    pub fn 获取特定课程的位置(&self, course_id: i64) -> Vec<Struct位置> {
        let mut query = self
            .connection
            .prepare("SELECT * FROM pos WHERE courseid=?;")
            .unwrap();
        query.bind((1, course_id)).unwrap();
        let mut 位置列表 = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let addr = row.read("addr");
                let lat = row.read("lat");
                let lon = row.read("lon");
                let alt = row.read("alt");
                位置列表.push(Struct位置::new(addr, lon, lat, alt));
            } else {
                eprintln!("位置解析行出错：{c:?}.");
            }
        }
        位置列表
    }
}

// alias
impl DataBase {
    const CREATE_ALIAS_SQL: &'static str =
        "CREATE TABLE alias (name CHAR (50) UNIQUE NOT NULL,posid INTEGER NOT NULL);";

    fn has_table_alias(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='alias';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }

    pub fn has_alias(&self, alias: &str) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM alias WHERE name=?;")
            .unwrap();
        query.bind((1, alias)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_alias(&self, alias: &str) {
        let mut query = self
            .connection
            .prepare("DELETE FROM alias WHERE name=?;")
            .unwrap();
        query.bind((1, alias)).unwrap();
        query.next().unwrap();
    }

    // pub fn 删除所有别名(&self) {
    //     self.connection.execute("DELETE FROM alias;").unwrap();
    // }

    pub fn add_alias_or<O: Fn(&DataBase, &str, i64)>(&self, 别名: &str, 位置id: i64, or: O) {
        let mut query = self
            .connection
            .prepare("INSERT INTO alias(name,posid) values(:name,:posid);")
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(&[(":name", 别名.into()), (":posid", 位置id.into())][..])
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, 别名, 位置id),
        };
    }
    pub fn update_alias(&self, alias: &str, 位置id: i64) {
        let mut query = self
            .connection
            .prepare("UPDATE alias SET name=:name,posid=:posid WHERE name=:name;")
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(&[(":name", alias.into()), (":posid", 位置id.into())][..])
            .unwrap();
        query.next().unwrap();
    }

    fn create_table_alias(&self) {
        if !self.has_table_alias() {
            self.connection.execute(Self::CREATE_ALIAS_SQL).unwrap();
        }
    }
    pub fn 获取为某别名的位置(&self, alias: &str) -> Option<Struct位置> {
        if self.has_alias(alias) {
            let mut query = self
                .connection
                .prepare("SELECT * FROM alias WHERE name=?;")
                .unwrap();
            query.bind((1, alias)).unwrap();
            let c: Vec<sqlite::Row> = query
                .iter()
                .filter_map(|e| if let Ok(e) = e { Some(e) } else { None })
                .collect();
            let row = &c[0];
            let 位置id: i64 = row.read("posid");
            Some(self.获取为某id的位置(位置id).1)
        } else {
            None
        }
    }
    pub fn get_aliases(&self, 位置id: i64) -> Vec<String> {
        let mut query = self
            .connection
            .prepare("SELECT * FROM alias WHERE posid=?;")
            .unwrap();
        query.bind((1, 位置id)).unwrap();
        let mut aliases = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let name: &str = row.read("name");
                aliases.push(name.to_owned());
            } else {
                eprintln!("位置解析行出错：{c:?}.");
            }
        }
        aliases
    }
}
