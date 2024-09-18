pub mod lesson;
mod live;
mod protocol;
mod room;
mod tools;

use crate::xddcc::{live::Live, room::Room, tools::PairVec};
use cxsign::default_impl::store::{AccountTable, DataBase};
use indicatif::MultiProgress;
use lesson::Lesson;
use log::warn;
use std::path::PathBuf;

pub fn xddcc(
    accounts: Option<String>,
    this: bool,
    device_code: Option<String>,
    id: Option<i64>,
    just_id: bool,
    output: Option<PathBuf>,
    list: bool,
    db: &DataBase,
    multi: &MultiProgress,
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
        let rooms = tools::map_sort_by_key(Room::get_all_rooms(sessions.values(), multi));
        tools::out(&PairVec::new(rooms), output)
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
            tools::get_live_video_path(session, &device_code)
                .ok()
                .map(|path| {
                    tools::out(&path, output.clone());
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
                tools::out(
                    &Lesson::get_recording_url(&session, live_id).unwrap_or_default(),
                    output.clone(),
                );
            } else {
                if just_id {
                    tools::out(
                        &Lesson::get_all_lessons(&session, live_id).unwrap_or_default(),
                        output.clone(),
                    );
                } else {
                    tools::out(
                        &PairVec::new(tools::map_sort_by_key(
                            Lesson::get_recording_lives(&session, live_id, multi)
                                .unwrap_or_default(),
                        )),
                        output.clone(),
                    );
                }
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
        tools::out(
            &PairVec::new(tools::map_sort_by_key(Live::get_lives_now(
                sessions.values(),
                this,
                multi,
            ))),
            output.clone(),
        );
    }
}
