import { invoke } from "@tauri-apps/api/core";
import type { AccountPair } from "./account";
export type Course = {
  id: number;
  class_id: number;
  teacher: string;
  image_url: string;
  name: string;
};
export type CoursePair = {
  course: Course;
  unames: AccountPair[];
};
export type CoursePairs = {
  ok: CoursePair[];
  excluded: CoursePair[];
};
export async function listCourses(): Promise<CoursePair[]> {
  let courses = [];
  await invoke<CoursePair[]>("list_courses")
    .then((data) => {
      courses = data;
    })
    .catch((error) => {
      console.error(error);
    });
  return courses;
}
export async function loadCourses(): Promise<void> {
  await invoke<void>("load_courses").catch((error) => {
    console.error(error);
  });
}
