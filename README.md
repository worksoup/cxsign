# cxsign

X 星签到的命令行工具，可以为每个课程添加单独的地点。支持多个账号签到。支持普通签到、拍照签到、二维码签到、位置签到、二维码位置签到、手势签到、签到码签到。

## 注意事项

**本项目使用的网络 API 均来源于网络公开资料。**

**本项目不提供任何现实中的网络服务，仅提供相关功能实现的参考，请勿用于任何实际用途。**

本项目以 `AGPL-3.0` 协议开源。同时包含如下附加协议：

- **本项目仅供学习和研究使用，严禁商用。**
- 不得广泛传播（严禁包括但不限于网盘分享、群组分享等一次性或持续性多人分享行为）。
- 严禁任何不以学习研究为目的个人分享行为。

## 特别注意

**请注意账号安全**：

虽然输入密码时不会在命令行界面显示任何字符，但是密码依然被缓存本地数据库中且存在被解密的风险。

请注意保护登录者的账号密码。

> - 如果让他人登录你的账号，请仔细甄别其是否值得信任;
> - 如果你登录了其他人的账号，请牢记：能力越大，责任越大，同时不要辜负他人对你的信任;
> - 你的账号不论供自己登录还是供他人登录、不论是否使用本软件或其他第三方软件登录，都推荐为其设置一个单独的、别于其他账号甚至完全无关的、不常用的密码，防止该密码被盗取后用于登录其他平台账号或猜出你设置密码的风格。
>
> 另：为了账号安全，请不要使用 Debug 版本。

(TODO: 考虑是否有必要为数据库添加主密码。)

**本项目目前主要在 Windows 下测试**：

- 本项目使用跨平台库，但由于各位同学都只用 Windows, 所以主要在 Windows 上测试。
- 我也在自己的电脑上测试，故在 Linux 上也应当有良好支持。

## 其他注意

- 本项目在没有账号登录的情况下会进行有限的提示，请注意登录你要操作的账号。

- 相较于旧版，本项目的新版本添加了一个排除列表，在第一次运行时会将**超过数月未发签到**或**还没发过签到**
  的课程添加进排除列表中，这些课程将不会获取他们签到。

  如果他们这时新发了签到，本项目将不会获取之。这时需要使用`cxsign list -a` 命令强制列出所有签到或使用
  `cxsign list -c <COURSE_ID>` 列出特定课程的签到，此时将会刷新排除列表。

  注意，由于 `cxsign list -a` 命令在旧版本中会出现线程数量过多的现象，新版限制了线程数量，所以该命令耗时较长，为十几秒到数分钟不等。

  所以如果只有单个课程新发了签到，建议：

    - 先使用 `cxsign course` 列出所有课程。如果你的课程有变动，请添加 `-f,  --fresh` 选项。课程太多可以搭配 ripgrep 等使用。
    - 然后使用 `cxsign list -c <COURSE_ID>`（注意是**课程号**）列出特定课程的签到，这时本项目会强制获取该课程的签到，如果有新的签到则会将其列出，并将该课程移出排除列表。

## 使用方法

见 [Wiki](https://github.com/worksoup/cxsign/wiki).

## 建议、问题、反馈

欢迎发 issues 和 pr.
