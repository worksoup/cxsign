use cxlib::default_impl::store::{AccountTable, DataBase};
use indicatif::{MultiProgress, ProgressBar};
use log::{error, warn};
use std::path::PathBuf;
use xddcc::{
    lesson::Lesson, Live, PairVec, ProgressState, ProgressTracker, ProgressTrackerHolder, Room,
};

pub fn xddcc(
    (uid_list_str, device_code, id): (Option<String>, Option<String>, Option<i64>),
    output: Option<PathBuf>,
    (db, multi_progress): (&DataBase, &MultiProgress),
    (previous, just_id, list): (bool, bool, bool),
) {
    if list {
        if device_code.is_some() {
            warn!("多余的参数: `-d, --device-code`.")
        }
        if previous {
            warn!("多余的参数: `-p, --previous`.")
        }
        if id.is_some() {
            warn!("多余的参数: `-r, --record`.")
        }
        if just_id {
            warn!("多余的参数: `-j, --just_id`.")
        }
        // if web {
        //     warn!("多余的参数: `-w, --web`.")
        // }
        let sessions = if let Some(uid_list_str) = uid_list_str {
            AccountTable::get_sessions_by_uid_list_str(db, &uid_list_str)
        } else {
            AccountTable::get_sessions(db)
        };
        if sessions.is_empty() {
            warn!("请至少登录一个账号！");
        }
        let rooms = xddcc::map_sort_by_key(Room::get_all_rooms(sessions.values(), multi_progress));
        xddcc::out(&PairVec::new(rooms), output)
    } else if let Some(device_code) = device_code {
        if uid_list_str.is_some() {
            warn!("多余的参数: `-a, --accounts`.")
        }
        if previous {
            warn!("多余的参数: `-p, --previous`.")
        }
        if id.is_some() {
            warn!("多余的参数: `-r, --record`.")
        }
        if just_id {
            warn!("多余的参数: `-j, --just_id`.")
        }
        let sessions = AccountTable::get_sessions(db);
        if sessions.is_empty() {
            warn!("未有登录的账号！");
        }
        if let Some(session) = sessions.values().next() {
            xddcc::get_live_video_path(session, &device_code)
                .ok()
                .map(|path| {
                    xddcc::out(&path, output.clone());
                    true
                });
        }
    } else if let Some(live_id) = id {
        let sessions = if let Some(uid_list_str) = uid_list_str {
            AccountTable::get_sessions_by_uid_list_str(db, &uid_list_str)
        } else {
            AccountTable::get_sessions(db)
        };
        if sessions.is_empty() {
            warn!("未有登录的账号！");
        }
        if let Some(session) = sessions.into_values().next() {
            if previous {
                if just_id {
                    warn!("多余的参数: `-j, --just_id`.")
                }
                xddcc::out(
                    &Lesson::get_recording_url(&session, live_id).unwrap_or_default(),
                    output.clone(),
                );
            } else if just_id {
                xddcc::out(
                    &Lesson::get_all_lessons(&session, live_id).unwrap_or_default(),
                    output.clone(),
                );
            } else {
                xddcc::out(
                    &PairVec::new(xddcc::map_sort_by_key(
                        Lesson::get_recording_lives(&session, live_id, multi_progress)
                            .unwrap_or_default(),
                    )),
                    output.clone(),
                );
            }
        }
    } else {
        if just_id {
            warn!("多余的参数: `-j, --just_id`.")
        }
        let sessions = if let Some(uid_list_str) = uid_list_str {
            AccountTable::get_sessions_by_uid_list_str(db, &uid_list_str)
        } else {
            AccountTable::get_sessions(db)
        };
        if sessions.is_empty() {
            warn!("未有登录的账号！");
        }
        xddcc::out(
            &PairVec::new(xddcc::map_sort_by_key(Live::get_lives_now(
                sessions.values(),
                previous,
                multi_progress,
            ))),
            output.clone(),
        );
    }
}

use std::error::Error as ErrorTrait;
use std::ops::Deref;
use wnewtype::NewType;

pub(crate) fn prog_init_error_handler<T>(e: impl ErrorTrait) -> T {
    error!("json 解析出错！错误信息：{e}.");
    panic!()
}

#[derive(Debug, NewType)]
pub struct MyProgressBar(ProgressBar);

impl ProgressTracker for MyProgressBar {
    fn inc(&self, delta: u64) {
        self.deref().inc(delta);
    }

    fn finish(&self, data: ProgressState) {
        let data = match data {
            ProgressState::GetRecordingLives => "获取回放地址完成。",
            ProgressState::GetLiveIds => "获取直播号完成。",
            ProgressState::GetLiveUrls => "已获取直播地址。",
            ProgressState::GetDeviceCodes => "获取设备码完成。",
            _ => "获取 Bug 完成。",
        };
        self.finish_with_message(data);
    }
}

#[derive(Debug, NewType)]
pub struct MyMultiProgress(MultiProgress);
impl ProgressTrackerHolder<MyProgressBar> for MultiProgress {
    fn init(&self, total: u64, data: ProgressState) -> MyProgressBar {
        let data = match data {
            ProgressState::GetRecordingLives => {
                "获取回放地址：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            }
            ProgressState::GetLiveIds => {
                "获取直播号：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            }
            ProgressState::GetLiveUrls => {
                "获取地址中：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            }
            ProgressState::GetDeviceCodes => {
                "获取设备码：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            }
            _ => "获取 Bug 中：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        };
        let sty =
            indicatif::ProgressStyle::with_template(data).unwrap_or_else(prog_init_error_handler);
        let pb = self.add(ProgressBar::new(total));
        pb.set_style(sty);
        pb.into()
    }

    fn remove_progress(&self, progress: &MyProgressBar) {
        self.remove(progress);
    }
}
