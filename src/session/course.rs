use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Struct课程 {
    课程号: i64,
    班级号: i64,
    任课教师: String,
    封面图url: String,
    课程名: String,
}

impl Display for Struct课程 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "课程号: {}, 课程名: {}, 任课教师: {}",
            self.课程号, self.课程名, self.任课教师
        )
    }
}

impl Struct课程 {
    pub fn new(
        course_id: i64,
        class_id: i64,
        teacherfactor: &str,
        imageurl: &str,
        name: &str,
    ) -> Struct课程 {
        Struct课程 {
            课程号: course_id,
            班级号: class_id,
            任课教师: teacherfactor.into(),
            封面图url: imageurl.into(),
            课程名: name.into(),
        }
    }
    pub fn get_课程号(&self) -> i64 {
        self.课程号
    }
    pub fn get_班级号(&self) -> i64 {
        self.班级号
    }
    pub fn get_任课教师(&self) -> &str {
        &self.任课教师
    }
    pub fn get_封面图url(&self) -> &str {
        &self.封面图url
    }
    pub fn get_课程名(&self) -> &str {
        &self.课程名
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CourseRaw {
    #[serde(rename = "id")]
    pub 课程号: i64,
    #[serde(rename = "teacherfactor")]
    pub 任课教师: String,
    #[serde(rename = "imageurl")]
    pub 封面图url: String,
    #[serde(rename = "name")]
    pub 课程名: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Courses {
    pub data: Vec<CourseRaw>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CourseContent {
    #[serde(rename = "course")]
    pub 课程: Option<Courses>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClassRaw {
    #[serde(rename = "key")]
    pub 班级号: Value,
    #[serde(rename = "content")]
    pub content: CourseContent,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetCoursesR {
    #[serde(rename = "channelList")]
    pub channel_list: Vec<ClassRaw>,
}
