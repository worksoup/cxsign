use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Course {
    course_id: i64,
    class_id: i64,
    teacherfactor: String,
    imageurl: String,
    name: String,
}

impl Course {
    pub fn display(&self) {
        println!(
            "id: {}, 课程名: {}, 任课教师: {}",
            self.course_id, self.name, self.teacherfactor
        )
    }
    pub fn new(
        course_id: i64,
        class_id: i64,
        teacherfactor: &str,
        imageurl: &str,
        name: &str,
    ) -> Course {
        Course {
            course_id,
            class_id,
            teacherfactor: teacherfactor.into(),
            imageurl: imageurl.into(),
            name: name.into(),
        }
    }
    pub fn get_course_id(&self) -> i64 {
        self.course_id
    }
    pub fn get_class_id(&self) -> i64 {
        self.class_id
    }
    pub fn get_任课教师(&self) -> &str {
        &self.teacherfactor
    }
    pub fn get_封面图url(&self) -> &str {
        &self.imageurl
    }
    pub fn get_课程名(&self) -> &str {
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
