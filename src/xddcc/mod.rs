mod live;
mod protocol;
mod room;
mod tools;

use crate::xddcc::{live::Live, room::Room, tools::PairVec};
use clap::Parser;
use cxsign::store::tables::AccountTable;
use indicatif::MultiProgress;
use log::warn;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "获取直播信息。")]
pub struct XddccSubCommand {
    /// 获取特定账号下节课的直播信息，格式为以半角逗号隔开的字符串。
    #[arg(short, long)]
    pub accounts: Option<String>,
    /// 覆盖默认行为至获取当前课的直播信息。
    #[arg(short, long)]
    pub this: bool,
    /// 通过 `device_code` 获取直播信息。
    #[arg(short, long)]
    pub device_code: Option<String>,
    /// 导出文件路径。可选提供。
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// 列出所有设备码。
    #[arg(short, long)]
    pub list: bool,
    // /// 网页播放器地址。
    // #[arg(short, long)]
    // pub web: bool,
}
pub fn xddcc(
    accounts: Option<String>,
    this: bool,
    device_code: Option<String>,
    output: Option<PathBuf>,
    list: bool,
    table: &AccountTable,
    multi: &MultiProgress,
) {
    if list {
        if device_code.is_some() {
            warn!("多余的参数: `-d, --device-code`.")
        }
        if this {
            warn!("多余的参数: `-t, --this`.")
        }
        // if web {
        //     warn!("多余的参数: `-w, --web`.")
        // }
        let sessions = if let Some(accounts) = accounts {
            table.get_sessions_by_accounts_str(&accounts)
        } else {
            table.get_sessions()
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
        let sessions = table.get_sessions();
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
    } else {
        let sessions = if let Some(accounts) = accounts {
            table.get_sessions_by_accounts_str(&accounts)
        } else {
            table.get_sessions()
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
