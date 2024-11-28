// Copyright (C) 2024 worksoup <https://github.com/worksoup/>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use clap::{Parser, Subcommand};
#[cfg(feature = "clap_complete_command")]
use clap_complete_command::Shell;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "进行签到。",
    long_about = r#"
进行签到。

关于签到行为：

普通签到不需要指定任何选项。
拍照签到可指定 `-p, --pic` 选项，提供照片位置。如不提供则从云盘上获取图片。
二维码签到可指定 `-p, --pic` 选项，提供照片位置。如不提供则从屏幕上截取。
位置签到可指定 `-l, --location` 选项。如不提供则根据教师设置的签到范围或数据库中获取。
手势或签到码签到须指定 `-s, --signcode` 选项，提供签到码。
"#
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<MainCommand>,
    /// 签到 ID.
    /// 默认以最近起对所有有效签到顺序进行签到，且缺少参数时会跳过并继续。
    pub id: Option<i64>,
    /// 签到账号，格式为以半角逗号隔开的 uid (可通过 accounts 子命令查看).
    /// 默认以一定顺序对所有用户进行签到。
    #[arg(short, long)]
    pub uid: Option<String>,
    /// 指定位置。
    /// 教师未指定位置的位置签到或需要位置的二维码签到需要提供。
    /// 格式为：`地址,经度,纬度,海拔`, 不满足格式的字符串将被视为别名。
    /// 如果该别名不存在，则视为位置 ID.
    /// 其余情况将视为自动获取位置时指定的地址名。
    /// 如未指定或错误指定则按照先课程位置后全局位置的顺序依次尝试。
    #[arg(short, long)]
    pub location: Option<String>,
    /// 本地图片路径。
    /// 拍照签到需要提供，二维码签到可选提供。
    /// 如果是文件，则直接使用该文件作为拍照签到图片或二维码图片文件。
    /// 如果是目录，则会选择在该目录下修改日期最新的图片作为拍照签到图片或二维码图片。
    #[arg(short, long)]
    pub image: Option<PathBuf>,
    /// 精确地截取二维码。
    /// 如果二维码识别过慢可以尝试添加添加此选项。
    #[arg(long)]
    pub precisely: bool,
    /// 签到码。
    /// 签到码签到时需要提供。
    #[arg(short, long)]
    pub code: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum MainCommand {
    /// 账号相关操作（列出、添加、删除）。
    Account {
        #[command(subcommand)]
        command: AccountSubCommand,
    },
    /// 列出所有账号。
    Accounts {
        /// 重新获取账号信息并缓存。
        #[arg(short, long)]
        fresh: bool,
    },
    /// 获取课程信息。
    Courses {
        /// 待操作账号，格式为以半角逗号隔开的 uid (可通过 accounts 子命令查看).
        #[arg(short, long)]
        uid: Option<String>,
    },
    /// 列出有效签到。
    List {
        /// 列出指定课程的签到。
        #[arg(short, long)]
        course: Option<i64>,
        /// 列出所有签到（包括无效签到）。
        #[arg(short, long)]
        all: bool,
    },
    /// 列出所有位置。
    Locations {
        /// 以更好的格式显示结果。
        #[arg(short, long)]
        pretty: bool,
        /// 精简显示结果。
        #[arg(short, long)]
        short: bool,
    },
    /// 显示配置文件夹位置。
    WhereIsConfig,
    /// 获取直播信息。
    Xddcc {
        /// 获取特定账号下节课的直播信息，格式为以半角逗号隔开的字符串。
        #[arg(short, long)]
        accounts: Option<String>,
        /// 通过 `device_code` 获取直播信息。
        #[arg(short, long)]
        device_code: Option<String>,
        /// 获取回放信息，参数为 `live_id`, 默认获取整门课的。
        #[arg(short, long)]
        id: Option<i64>,
        /// 获取某门课中每节课的 `live_id`.
        #[arg(short, long)]
        just_id: bool,
        /// 导出文件路径。可选提供。
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 列出所有设备码。
        #[arg(short, long)]
        list: bool,
        /// 获取直播信息时覆盖默认行为至获取上节课的直播信息，获取回放信息时指定只获取一节课的回放信息。
        #[arg(short, long)]
        previous: bool,
        // /// 网页播放器地址。
        // #[arg(short, long)]
        // web: bool,
    },
    #[cfg(feature = "clap_complete_command")]
    /// 生成命令补全文件。
    Completions {
        /// 补全的 Shell 类型。
        #[arg(value_enum)]
        shell: Shell,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
/// 账号操作子命令。
#[derive(Subcommand, Debug)]
pub enum AccountSubCommand {
    /// 添加账号。
    Add {
        /// 账号（手机号）。
        uname: String,
        /// 密码（明文）。
        /// 指定后将跳过询问密码阶段。
        passwd: Option<String>,
    },
    /// 删除账号。
    Remove {
        /// uid (可通过 accounts 子命令查看).
        uid: String,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Clone)]
pub struct CliArgs {
    pub location_str: Option<String>,
    pub image: Option<PathBuf>,
    // pub capture: bool,
    pub precisely: bool,
    pub signcode: Option<String>,
}
