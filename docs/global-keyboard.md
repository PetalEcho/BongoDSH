# 全局键盘监听

## 使用方式

程序启动时使用窗口输入，不自动申请权限或开启全局监听。右键角色进入「键盘输入设置…」，选择全局键盘；「设置与预览」面板也提供同一入口。在「仅窗口内输入」模式下，离开挂件窗口就停止反馈；全局模式可在其他应用打字时驱动双手动画。暂停、点击试敲和节奏演示会停止真实监听，继续或最后一次回弹结束后恢复此前选择的模式。全局模式的预览期间不接收窗口按键，避免双源混用。恢复默认会停止全局监听并切回窗口模式。当前不保存输入源选择。

macOS 使用 Input Monitoring，相关权限 API 需要 macOS 10.15 或更高版本。点击「申请输入监控权限」才调用系统申请；启动、重连和后台检查仅查询状态。点击「打开系统设置」可进入输入监控面板，操作系统授权开关由用户完成。应用不会请求辅助功能控制或键盘事件发送权限。查询使用 [CGPreflightListenEventAccess](https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess())，申请使用 `IOHIDRequestAccess(kIOHIDRequestTypeListenEvent)`。Apple 在 [WWDC19](https://developer.apple.com/videos/play/wwdc2019/701/) 说明了该申请方式，后续 [开发者支持回复](https://developer.apple.com/forums/thread/828052) 也将其用于应用未出现在输入监控列表的情况。

通过 `cargo run`、终端或编辑器运行未打包程序时，系统可能按启动应用显示权限。开启对应项目后点击「检查并重新连接」；若仍提示失败，退出挂件再从原入口启动。授权归属、系统提示和重启要求需在最终签名安装包上重新验收。拒绝、撤销或初始化失败时保留窗口输入，并在设置中显示状态。

### macOS 本地预览与未列出应用时的处理

先退出旧预览，再构建并打包：

```sh
cargo build --workspace --release --locked
bash scripts/package-macos.sh
```

从访达打开 `target/BongoDSH Preview.app`。脚本为整个应用包添加本地 ad hoc 签名，并验证二进制、应用标识和 Info.plist 一致；不会启动应用、申请权限或更改系统设置。自定义构建目录时，可将二进制路径作为脚本第一个参数。

在键盘设置中点击「申请输入监控权限」，然后打开系统设置。列表中没有应用时，点击「在访达中显示应用」，再在系统输入监控页面使用「+」选择同一个 `BongoDSH Preview.app` 并开启。授权后点击「检查并重新连接」，仍不可用时退出并重新打开这个应用，再选择全局键盘。

本地签名用于本机开发验证，不代表 Developer ID 签名或公证。[Apple 的代码身份说明](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements) 指出 ad hoc 身份与具体构建绑定，因此重新构建后可能需要用户重新授权。不要修改 TCC 数据库或重置其他应用的权限来解决此问题。

Windows 使用当前登录会话中的键盘事件，不要求管理员权限。锁屏、安全桌面、UAC 提示及其他登录会话不在监听能力范围内。普通桌面上的高权限应用、远程桌面和不同输入法仍需实机验收。

## 实现边界

- `bongodsh-core` 保持标准库依赖和 `forbid(unsafe_code)`，只接收物理键标识、按下、松开与重置。
- `bongodsh-app/src/global_input` 管理独立监听线程、有界队列、状态与线程退出。切换时停止并回收旧线程，销毁旧队列，避免迟到事件进入新输入源。
- `macos.rs` 使用 [只读 CGEventTap](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions/listenonly)、独立 CFRunLoop 和 IOKit 睡眠通知。启用安全输入时暂停反馈；每秒检查权限、事件源及遗漏的松开，超时禁用后重置并重新启用。
- `windows.rs` 使用 [WH_KEYBOARD_LL](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc) 和独立消息循环，所有事件都调用下一钩子，不阻止前台应用收键。winit 已拥有进程级 Raw Input 注册，另行注册会更换接收窗口，因此不再注册第二套 Raw Input。[Microsoft 的注册约束](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerrawinputdevices) 说明一个进程中每类设备只有一个接收窗口。
- Windows 隐藏窗口接收睡眠、设备变化与会话通知。每秒校对遗漏的松开；消息循环发生长停顿时重建可能被系统移除的钩子。[会话通知](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification) 在退出时注销。
- 两个平台的回调只过滤键码并尝试投递事件，UI 唤醒在回调返回后执行。队列最多保存 256 个事件，溢出时丢弃该批并重置，避免丢失松开造成卡手。监听线程用低频维护等待检查状态，静止时不持续重绘。
- 只在 `macos.rs`、`windows.rs` 放行 `unsafe`；两处都拒绝未显式标注的 unsafe 操作和缺少安全说明的 unsafe 代码块。原生句柄在所属线程创建、使用和释放。

不读取字符、输入法文本、剪贴板或前台窗口内容，不写入按键日志。临时按键状态仅用于动画。字母区、数字、标点、空格、退格、Tab、回车和数字键盘会触发动作，Shift 输入保留；Command、Control、Alt / Option 组合和功能键被过滤。AltGr 组合目前也按快捷键过滤。松开不受快捷键过滤影响。Windows 排除系统标为注入的事件，macOS 依据系统连发标记去重。

同一个物理键码不按设备区分，多键盘同时按住同一码不作为独立按键计数。尚未验证所有国际键盘、IME、远程桌面与系统特殊键。

## 验证记录与待验收项

2026-11-02，在 macOS Apple Silicon、Rust 1.93.1 上执行。默认自动测试使用模拟事件，不申请权限，不发送真实系统按键。测试位于被忽略的本地 `tests/` 目录，不随源码提交。

本地回归覆盖全局事件在无焦点时驱动双手、窗口事件去重、焦点切换、权限失效降级、暂停与演示互斥、源切换丢弃旧事件、重复按下、快捷键松开、重置后迟到事件、扩展扫描码和队列溢出。原生生命周期冒烟测试检测到本机未授权，确认启动直接报告缺少权限，不触发授权弹窗。

初次全局监听提交 `b55862b` 的验证记录为 `cargo fmt`、macOS 全目标严格 Clippy、38 项本地回归、Release 构建和 `--demo` 均通过；上面的未授权原生冒烟用例另外执行通过。Windows x86_64 MSVC 全目标严格 Clippy 通过。受限沙箱中的桌面检查遇到系统服务连接限制，随后在沙箱外使用同一 Release 产物确认创建了可见的 300×270 窗口，检查后关闭测试进程。此次只核对窗口创建，不代表完成设置界面交互或已授权键盘监听验收。

动画优化工作区已同步该提交并合并双手、拖动、缩放与独立设置。本次工作区执行了 32 项本地回归，另行通过原生权限检查，结果为 `PermissionRequired`，没有申请或更改系统权限。新增回归确认无焦点时接收全局事件、忽略重复窗口事件、权限失效复位及降级、预览与监听互斥、暂停和切源丢弃旧队列。macOS 格式检查、严格 Clippy、Release 构建及 Windows 目标严格 Clippy 均通过；已打开最终预览的键盘设置，已授权跨应用输入仍待用户操作后验收。

用户反馈系统未列出应用后，发现旧预览只保留 Rust 链接器签名，应用包校验失败。现已添加完整应用包签名脚本，`codesign --verify --strict` 通过，签名标识为 `dev.bongodsh.preview`。同时改用 IOKit 主动申请入口，补充手动添加说明、访达定位按钮和键盘设置窗口聚焦。修正后再次通过上述 32 项回归、macOS / Windows 严格 Clippy 和 Release 构建；原生界面已确认未授权状态及访达定位，尚未代用户申请权限。

随后用户反馈已连接，但桌面工具回读本轮预览仍为未授权，系统输入监控列表也未列出本应用。已打开添加入口，当前系统停在用户触控 ID 验证步骤。尚未确认该预览包授权成功，不能将此轮记录为已通过真实跨应用输入验收。

Windows x86_64 MSVC 可在当前 macOS 环境做目标编译与 Clippy 检查，不能在这里运行 Windows 可执行文件。跨平台编译通过不能代替以下实机验收：

| 场景 | 预期结果 | 本轮状态 |
| --- | --- | --- |
| macOS 桌面启动 | 默认窗口输入，不自动请求权限，创建角色窗口 | 确认可见的 300×270 窗口 |
| macOS 未授权开启全局模式 | 显示授权说明，保留窗口输入，不自动申请 | 原生降级检查通过 |
| macOS 授权、拒绝后重试、撤销、重新授权 | 状态正确，撤销清空双手，重连可恢复 | 待用户授权后实测 |
| 普通编辑器、浏览器与中文 IME 输入 | 双手交替，短按可见，长按不重复敲击 | 自动事件回归通过，真实跨应用待实测 |
| 按住键再暂停、切源或演示 | 双手复位，旧松开和连发不污染新模式 | 本地回归通过 |
| 睡眠、唤醒、拔键盘、安全输入、锁屏解锁 | 状态复位，没有卡手，恢复后新敲击正常 | 通知处理已实现，完整实机流程待验收 |
| Windows 普通桌面、UAC、设备变化、远程桌面 | 不影响前台应用，系统限制时安全停止反馈 | 目标编译通过，实机待验收 |
| 退出与重复启停 | 关闭原生监听、通知与线程 | 回收逻辑已检查；已授权启停待实测 |

运行原生冒烟测试的命令如下。它只查询现有授权，有权限时检查监听启停，无权限时检查降级，不修改系统权限。

```sh
cargo test -p bongodsh-app --test desktop_regression --locked native_listener_starts_or_reports_missing_permission_and_stops -- --ignored --nocapture
```
