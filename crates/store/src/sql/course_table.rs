use crate::sql::{DataBase, DataBaseTableTrait};
use base::course::Course;
use std::collections::HashMap;

pub struct CourseTable<'a> {
    db: &'a DataBase,
}

impl<'a> CourseTable<'a> {
    pub fn add_course_or<O: Fn(&Self, &Course)>(&self, course: &Course, or: O) {
        let id: i64 = course.get_id();
        let clazzid: i64 = course.get_class_id();
        let name: &str = course.get_name();
        let teacher: &str = course.get_teacher();
        let image: &str = course.get_image_url();
        let mut query =self.db.prepare(format!("INSERT INTO {}(id,clazzid,name,teacher,image) values(:id,:clazzid,:name,:teacher,:image);",Self::TABLE_NAME)).unwrap();
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

    pub fn get_courses(&self) -> HashMap<i64, Course> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut courses = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let id = row.read("id");
                let clazzid = row.read("clazzid");
                let teacher = row.read::<&str, _>("teacher");
                let image = row.read::<&str, _>("image");
                let name = row.read::<&str, _>("name");
                courses.insert(id, Course::new(id, clazzid, teacher, image, name));
            } else {
                eprintln!("课程解析行出错：{c:?}.");
            }
        }
        courses
    }
}

impl<'a> DataBaseTableTrait<'a> for CourseTable<'a> {
    const TABLE_ARGS: &'static str = "id INTEGER UNIQUE NOT NULL,clazzid INTEGER NOT NULL,name TEXT NOT NULL,teacher TEXT NOT NULL,image TEXT NOT NULL";
    const TABLE_NAME: &'static str = "course";

    fn from_ref(db: &'a DataBase) -> Self {
        Self { db }
    }
}
