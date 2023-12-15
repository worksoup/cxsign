use clap::{Parser, Subcommand};
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
位置签到可指定 `    --pos` 或 `-l, --location` 选项。如不提供则根据教师设置的签到范围或数据库中获取。
手势或签到码签到须指定 `-s, --signcode` 选项，提供签到码。
"#
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<MainCmds>,
    /// 签到 ID.
    /// 默认以最近起对所有有效签到顺序进行签到，且缺少参数时会跳过并继续。
    pub active_id: Option<i64>,
    /// 签到账号，格式为以半角逗号隔开的字符串。
    /// 默认以一定顺序对所有用户进行签到。
    #[arg(short, long)]
    pub accounts: Option<String>,
    /// 指定位置。
    /// 教师未指定位置的位置签到或需要位置的二维码签到需要提供。
    /// 格式为：`地址,经度,纬度,海拔`, 不满足格式的字符串将被视为别名。
    /// 如果该别名不存在，则视为位置 ID.
    /// 其余情况将按照先课程位置后全局位置的顺序依次尝试。
    #[arg(short, long)]
    pub pos: Option<String>,
    /// 本地图片路径。
    /// 拍照签到需要提供，二维码签到可选提供。
    /// 如果是文件，则直接使用该文件作为拍照签到图片或二维码图片文件。
    /// 如果是目录，则会选择在该目录下修改日期最新的图片作为拍照签到图片或二维码图片。
    #[arg(short, long)]
    pub img: Option<PathBuf>,
    // /// 从屏幕上获取二维码。
    // /// 二维码签到时需要提供。
    // #[arg(short, long)]
    // pub capture: bool,
    /// 精确地截取二维码。
    /// 如果二维码识别过慢可以尝试添加添加此选项。
    #[arg(long)]
    pub precise: bool,
    /// 签到码。
    /// 签到码签到时需要提供。
    #[arg(short, long)]
    pub signcode: Option<String>,
    /// 禁用位置随机偏移。
    #[arg(short, long)]
    pub no_random_shift: bool,
}

#[derive(Subcommand, Debug)]
pub enum MainCmds {
    /// 账号相关操作（列出、添加、删除）。
    /// 默认列出所有账号。
    Account {
        #[command(subcommand)]
        command: Option<AccCmds>,
        /// 重新获取账号信息并缓存。
        #[arg(short, long)]
        fresh: bool,
    },
    /// 列出所有课程。
    Course {
        /// 重新获取课程信息并缓存。
        #[arg(short, long)]
        fresh: bool,
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
    /// 位置相关操作（列出、添加、删除）。
    /// 默认列出所有位置。
    Pos {
        /// 指定 posid.
        posid: Option<i64>,
        /// 列出位置。
        #[arg(short, long)]
        list: bool,
        /// 添加位置。
        /// 格式为：`地址,经度,纬度,海拔`.
        #[arg(short, long)]
        new: Option<String>,
        /// 导入位置。
        /// 每行一个位置。课程号在前，位置在后，最后是别名。它们由字符 `$` 隔开。
        /// 其中位置的格式为 `地址,经度,纬度,海拔`, 别名的格式为以 `/` 分隔的字符串数组。
        #[arg(short, long)]
        import: Option<PathBuf>,
        /// 导出位置。
        /// 每行一个位置。课程号在前，位置在后，最后是别名。它们由字符 `$` 隔开。
        /// 其中位置的格式为 `地址,经度,纬度,海拔`, 别名的格式为以 `/` 分隔的字符串数组。
        #[arg(short, long)]
        export: Option<PathBuf>,
        /// 为位置添加别名。须同时指定
        #[arg(short, long)]
        alias: Option<String>,
        /// 删除位置。
        #[arg(short, long)]
        remove: bool,
        /// 删除所有位置。
        #[arg(long)]
        remove_all: bool,
        /// 删除所有别名。
        #[arg(long)]
        remove_all_alias: bool,
        /// 指定课程号。
        #[arg(short, long)]
        course: Option<i64>,
        /// 指定全局。
        #[arg(short, long)]
        global: bool,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
    /// 显示配置文件夹位置。
    WhereIsConfig,
}

#[derive(Subcommand, Debug)]
pub enum AccCmds {
    /// 添加账号。
    Add {
        /// 账号（手机号）。
        uname: String,
    },
    /// 删除账号。
    Remove {
        /// 账号（手机号）。
        uname: String,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
}

pub struct CliArgs {
    pub 位置字符串: Option<String>,
    pub 图片或图片路径: Option<std::path::PathBuf>,
    // pub capture: bool,
    pub 是否精确识别二维码: bool,
    pub 签到码: Option<String>,
    pub 是否禁用随机偏移: bool,
}

// pub enum PosCmds {
//     /// 添加位置。
//     Add {
//         /// 绑定该位置到指定课程。
//         /// 默认添加为全局位置。
//         course: Option<i64>,
//         /// 地址名称、经纬度与海拔。
//         /// 格式为：`地址,经度,纬度,海拔`.
//         pos: String,
//     },
//     /// 删除位置。
//     Remove {
//         /// 位置 ID.
//         posid: Option<i64>,
//         /// 无需确认直接删除。
//         yes: bool,
//         all: bool,
//     },
//     /// 导入位置。
//     Import {
//         /// 导入位置。
//         /// 每行一个位置。课程号在前，位置在后，由字符 `$` 隔开。
//         input: std::path::PathBuf,
//     },
//     /// 导入位置。
//     Export {
//         /// 导出位置。
//         /// 无法解析的行将会被跳过。
//         output: std::path::PathBuf,
//     },
//     // 为位置添加别名
//     Alias {
//         // 位置 ID.
//         posid: i64,
//         // 别名。
//         alias: String,
//     },
// }
