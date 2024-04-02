use crate::protocol;
use cxsign_user::Session;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;
use ureq::serde_json;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Course {
    id: i64,
    class_id: i64,
    teacher: String,
    image_url: String,
    name: String,
}

impl Display for Course {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "班级号：{}, 课程号: {}, 课程名: {}, 任课教师: {}",
            self.class_id, self.id, self.name, self.teacher
        )
    }
}

impl Course {
    pub fn get_courses(session: &Session) -> Result<Vec<Course>, Box<ureq::Error>> {
        let r = protocol::back_clazz_data(session.deref())?;
        let courses = Course::get_list_from_response(r)?;
        info!("用户[{}]已获取课程列表。", session.get_stu_name());
        Ok(courses)
    }

    fn get_list_from_response(r: ureq::Response) -> Result<Vec<Course>, Box<ureq::Error>> {
        let r: GetCoursesR = r.into_json().unwrap();
        let mut arr = Vec::new();
        for c in r.channel_list {
            if let Some(data) = c.content.course {
                for course in data.data {
                    if c.id.is_i64() {
                        arr.push(Course::new(
                            course.id,
                            c.id.as_i64().unwrap(),
                            course.teacher.as_str(),
                            course.image_url.unwrap_or("".into()).as_str(),
                            course.name.as_str(),
                        ))
                    }
                }
            }
        }
        Ok(arr)
    }

    pub fn new(id: i64, class_id: i64, teacher: &str, image_url: &str, name: &str) -> Course {
        Course {
            id,
            class_id,
            teacher: teacher.into(),
            image_url: image_url.into(),
            name: name.into(),
        }
    }
    // fn from_raw(raw: &CourseRaw, class_id: i64) -> Course {
    //     Self {
    //         id: raw.id,
    //         class_id,
    //         teacher: raw.teacher.clone(),
    //         image_url: raw.image_url.clone(),
    //         name: raw.name.clone(),
    //     }
    // }
    pub fn get_id(&self) -> i64 {
        self.id
    }
    pub fn get_class_id(&self) -> i64 {
        self.class_id
    }
    pub fn get_teacher(&self) -> &str {
        &self.teacher
    }
    pub fn get_image_url(&self) -> &str {
        &self.image_url
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct CourseRaw {
    id: i64,
    #[serde(rename = "teacherfactor")]
    teacher: String,
    #[serde(rename = "imageurl")]
    image_url: Option<String>,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Courses {
    data: Vec<CourseRaw>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CourseContent {
    course: Option<Courses>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ClassRaw {
    #[serde(rename = "key")]
    id: serde_json::Value,
    content: CourseContent,
}

#[derive(Deserialize, Serialize, Debug)]
struct GetCoursesR {
    #[serde(rename = "channelList")]
    channel_list: Vec<ClassRaw>,
}
