use cxsign::default_impl::store::{AccountTable, DataBase};
use indicatif::{MultiProgress, ProgressBar};
use log::{error, warn};
use std::path::PathBuf;
use xddcc::{lesson::Lesson, Live, PairVec, ProgressTracker, ProgressTrackerHolder, Room};

pub fn xddcc(
    (accounts, device_code, id): (Option<String>, Option<String>, Option<i64>),
    output: Option<PathBuf>,
    (db, multi_progress): (&DataBase, &MultiProgress),
    (this, just_id, list): (bool, bool, bool),
) {
    if list {
        if device_code.is_some() {
            warn!("多余的参数: `-d, --device-code`.")
        }
        if this {
            warn!("多余的参数: `-t, --this`.")
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
        let sessions = if let Some(accounts) = accounts {
            AccountTable::get_sessions_by_accounts_str(db, &accounts)
        } else {
            AccountTable::get_sessions(db)
        };
        if sessions.is_empty() {
            warn!("请至少登录一个账号！");
        }
        let rooms = xddcc::map_sort_by_key(Room::get_all_rooms(sessions.values(), multi_progress));
        xddcc::out(&PairVec::new(rooms), output)
    } else if let Some(device_code) = device_code {
        if accounts.is_some() {
            warn!("多余的参数: `-a, --accounts`.")
        }
        if this {
            warn!("多余的参数: `-t, --this`.")
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
        let sessions = if let Some(accounts) = accounts {
            AccountTable::get_sessions_by_accounts_str(db, &accounts)
        } else {
            AccountTable::get_sessions(db)
        };
        if sessions.is_empty() {
            warn!("未有登录的账号！");
        }
        if let Some(session) = sessions.into_values().next() {
            if this {
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
        let sessions = if let Some(accounts) = accounts {
            AccountTable::get_sessions_by_accounts_str(db, &accounts)
        } else {
            AccountTable::get_sessions(db)
        };
        if sessions.is_empty() {
            warn!("未有登录的账号！");
        }
        xddcc::out(
            &PairVec::new(xddcc::map_sort_by_key(Live::get_lives_now(
                sessions.values(),
                this,
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

    fn finish(&self, progress_bar_holder: &impl ProgressTrackerHolder<Self>, msg: &'static str) {
        self.finish_with_message(msg);
        progress_bar_holder.remove_progress(self);
    }
}

#[derive(Debug, NewType)]
pub struct MyMultiProgress(MultiProgress);
impl ProgressTrackerHolder<MyProgressBar> for MultiProgress {
    fn init(&self, total: u64, data: &str) -> MyProgressBar {
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
