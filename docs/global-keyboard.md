# 全局键盘监听

## 使用方式

首次启动或没有保存记录时使用窗口输入。之后启动会恢复用户选择的输入模式；恢复全局模式只检查现有权限，不自动申请权限。从 DSH 拖出的角色在右键面板的「桌面选项」中选择全局键盘；独立鲸鱼原型通过「键盘输入设置…」进入。在「仅窗口内输入」模式下，离开挂件窗口就停止反馈；全局模式可在其他应用打字时驱动双手动画。暂停期间停止真实监听。DSH 桌面角色的设置面板打开时不处理真实按键反馈。输入源选择在用户切换时保存；暂停、重连、权限失效和退出不改写它。

macOS 使用 Input Monitoring，相关权限 API 需要 macOS 10.15 或更高版本。点击「申请输入监控权限」才调用系统申请；启动、重连和后台检查仅查询状态。点击「打开系统设置」可进入输入监控面板，操作系统授权开关由用户完成。应用不会请求辅助功能控制或键盘事件发送权限。Apple 提供了 [查询权限](https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess()) 和 [申请权限](https://developer.apple.com/documentation/coregraphics/cgrequestlisteneventaccess()) 两个独立接口。

通过 `cargo run`、终端或编辑器运行未打包程序时，系统可能按启动应用显示权限。开启对应项目后点击「检查并重新连接」；若仍提示失败，退出挂件再从原入口启动。授权归属、系统提示和重启要求需在最终签名安装包上重新验收。拒绝、撤销或初始化失败时保留窗口输入，并在设置中显示状态。

Windows 使用当前登录会话中的键盘事件，不要求管理员权限。锁屏、安全桌面、UAC 提示及其他登录会话不在监听能力范围内。普通桌面上的高权限应用、远程桌面和不同输入法仍需实机验收。

## 输入源选择的保存

macOS 保存到 `~/Library/Application Support/BongoDSH/keyboard-input-v1`，Windows 保存到 `%APPDATA%/BongoDSH/keyboard-input-v1`。文件只包含 `global` 或 `window`，不保存按键、输入文字或系统授权状态。每次明确切换输入模式时，在同目录写入临时文件并原子替换；重启、返回 DSH 后再次拖出及独立启动共用这项偏好。

没有配置文件时使用窗口输入。读取失败或内容损坏时回退窗口输入并提示原因；保存失败时本次选择仍生效，面板提示未保存，重启可能恢复旧值。macOS 的输入监控权限仍由系统管理，恢复全局模式不会触发授权申请。

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

本地回归覆盖全局事件在无焦点时驱动双手、窗口事件去重、焦点切换、权限失效降级、暂停状态切换、源切换丢弃旧事件、重复按下、快捷键松开、重置后迟到事件、扩展扫描码和队列溢出。原生生命周期冒烟测试检测到本机未授权，确认启动直接报告缺少权限，不触发授权弹窗。

本次全局监听提交版本的 `cargo fmt`、macOS 全目标严格 Clippy、38 项本地回归、Release 构建和 `--demo` 均通过；上面的未授权原生冒烟用例另外执行通过。Windows x86_64 MSVC 全目标严格 Clippy 通过。受限沙箱中的桌面检查遇到系统服务连接限制，随后在沙箱外使用同一 Release 产物确认创建了可见的 300×270 窗口，检查后关闭测试进程。此次只核对窗口创建，不代表完成设置界面交互或已授权键盘监听验收。

2026-11-03，0.2.3 新增 9 项输入偏好回归，格式、工作区测试、macOS/Windows Clippy 和 release 构建通过。真实 DSH 中开启全局模式后，返回页面再移到桌面、退出 DSH 后重启再移到桌面，均自动恢复选择；原生进程退出与重新启动没有改写偏好文件。系统暂时限制监听期间，全局模式选择仍保留。三种父管道关闭路径均正常退出。本次没有修改系统权限，权限撤销和 Windows 运行仍待专项验收。

Windows x86_64 MSVC 可在当前 macOS 环境做目标编译与 Clippy 检查，不能在这里运行 Windows 可执行文件。跨平台编译通过不能代替以下实机验收：

| 场景 | 预期结果 | 本轮状态 |
| --- | --- | --- |
| macOS 首次桌面启动 | 无保存记录时使用窗口输入，不自动请求权限 | 确认可见的 300×270 窗口 |
| macOS 未授权开启全局模式 | 显示授权说明，保留窗口输入，不自动申请 | 原生降级检查通过 |
| macOS 授权、拒绝后重试、撤销、重新授权 | 状态正确，撤销清空双手，重连可恢复 | 待用户授权后实测 |
| 普通编辑器、浏览器与中文 IME 输入 | 双手交替，短按可见，长按不重复敲击 | 自动事件回归通过，真实跨应用待实测 |
| 按住键再暂停或切源 | 双手复位，旧松开和连发不污染新模式 | 本地回归通过 |
| 睡眠、唤醒、拔键盘、安全输入、锁屏解锁 | 状态复位，没有卡手，恢复后新敲击正常 | 通知处理已实现，完整实机流程待验收 |
| Windows 普通桌面、UAC、设备变化、远程桌面 | 不影响前台应用，系统限制时安全停止反馈 | 目标编译通过，实机待验收 |
| 退出与重复启停 | 关闭原生监听、通知与线程 | 回收逻辑已检查；已授权启停待实测 |

运行原生冒烟测试的命令如下。它只查询现有授权，有权限时检查监听启停，无权限时检查降级，不修改系统权限。

```sh
cargo test -p bongodsh-app --test global_input --locked native_listener_starts_or_reports_missing_permission_and_stops -- --ignored --nocapture
```
