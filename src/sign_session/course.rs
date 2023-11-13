use serde_derive::{Deserialize, Serialize};
use toml::Value;

#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Course {
    course_id: i64,
    class_id: i64,
    teacherfactor: String,
    imageurl: String,
    name: String,
}

impl Course {
    pub fn new(
        course_id: i64,
        class_id: i64,
        teacherfactor: String,
        imageurl: String,
        name: String,
    ) -> Course {
        Course {
            course_id,
            class_id,
            teacherfactor,
            imageurl,
            name,
        }
    }
    pub fn get_id(&self) -> i64 {
        self.course_id
    }
    pub fn get_class_id(&self) -> i64 {
        self.class_id
    }
    pub fn get_teacher_name(&self) -> &str {
        &self.teacherfactor
    }
    pub fn get_image_url(&self) -> &str {
        &self.imageurl
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CourseRaw {
    pub id: i64,
    pub teacherfactor: String,
    pub imageurl: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Courses {
    pub data: Vec<CourseRaw>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CourseContent {
    pub course: Option<Courses>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Class {
    pub key: i64,
    pub content: CourseContent,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClassRaw {
    pub key: Value,
    pub content: CourseContent,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct GetCoursesR {
    pub channelList: Vec<ClassRaw>,
}
