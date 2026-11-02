# BongoDSH

使用 Rust 开发的桌面打字挂件，目标为 macOS 和 Windows。角色随敲键做出双手动作，可独立运行；后续通过轻量插件与 DSH 连接，跟随输入框并响应任务事件。

## 当前进度

已实现 egui/eframe 桌面原型，包含透明置顶窗口、角色拖动、双手按键动画和 macOS / Windows 全局键盘监听。默认只响应挂件窗口内的按键，右键可开启全局模式。音效和 DSH 插件尚未接入。

| 部分 | 当前状态 |
| --- | --- |
| 双手交替、长按去重、多键保持、断连重置 | 已实现核心规则 |
| 独立桌面窗口与角色动画 | 已实现 egui/eframe 0.34.3 原型 |
| macOS / Windows 全局输入与权限 | 已实现原生适配与 macOS 主动授权入口，实机验证边界见下文 |
| 点击试敲、拖动与位置锁定、滚轮缩放、右键菜单 | 已实现 |
| 动作预览、尺寸与敲击幅度、置顶与暂停 | 已实现，仅本次运行有效 |
| 点击穿透、托盘与本地设置 | 规划 |
| DSH 输入框吸附与任务提醒 | 后续桥接阶段 |

## 本地开发

需要 Rust stable 1.93 或更高版本。macOS 使用 Xcode Command Line Tools；Windows 使用 MSVC Rust 工具链和 Visual Studio C++ Build Tools。桌面依赖为 eframe 0.34.3，使用 Glow 渲染和内置基础字体，不需要 Node.js 或 Electron。

```sh
cargo run --locked --bin bongodsh
cargo run --locked --bin bongodsh -- --demo
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

不带参数时启动桌面窗口。`--demo` 保留为终端模拟事件演示，`--help` 显示用法。

点击角色后输入字母、数字或标点即可看到双手动作。手掌快速下压，接触时略微压扁，松开后回弹；长按保持，同一帧内按下又松开也能看到敲击。角色不配键盘图形。

| 操作 | 行为 |
| --- | --- |
| 点击身体 / 点击手掌 | 身体点击交替试敲；手掌点击只试敲对应的一只手 |
| 拖动角色 | 移动挂件；超过拖动阈值才移动，不触发点击试敲 |
| 悬停后滚动滚轮 | 在 70%–150% 范围缩放，保持双手落点附近的位置 |
| 右键 / 悬停后的菜单按钮 | 右键菜单包含设置与预览、键盘输入设置、锁定位置和退出；菜单按钮直接打开设置与预览 |
| 设置与预览 | 单手试敲、快慢节奏演示、尺寸、敲击幅度、锁定位置和置顶 |
| 暂停 / 恢复默认 | 暂停会停止演示并清空按住状态；恢复默认重置本次运行的设置 |

设置窗口独立于挂件，缩小角色时仍能完整操作。悬停仅显示右上角设置按钮，底部不显示提示横条。暂停和节奏演示在「设置与预览」面板中操作。设置暂不写入磁盘。透明背景当前仍会接收鼠标，不等于点击穿透。

窗口输入模式在失去焦点时清空按键状态。全局模式在其他应用打字时也驱动动画，并忽略挂件窗口重复上报的事件。点击试敲和节奏演示会暂时停止全局监听，动画结束后恢复此前选择的输入源。拖动、打开菜单和暂停会清空按键状态。中文输入法组合过程尚未专项验证，部分 IME 会吞掉窗口按键事件；不会根据输入文本补造按键。中文界面尝试读取系统字体，找不到字体时使用英文提示。

本地回归用例位于被 Git 忽略的 `crates/bongodsh-core/tests/` 和 `crates/bongodsh-app/tests/`，遵循测试不提交的约定。新克隆仓库不包含这些用例，`cargo test` 通过不能视为已执行相同回归覆盖。

## 全局键盘

右键角色，选择「键盘输入设置…」，再选择「全局键盘」。Windows 不要求管理员权限。macOS 首次使用时，在面板点击「申请输入监控权限」，按系统提示开启「隐私与安全性 → 输入监控」，然后点击「检查并重新连接」。通过终端或编辑器启动时，系统可能将权限归属到启动应用；授权后仍无法连接时，退出并重新启动挂件。

macOS 本地验证可在 Release 构建后运行 `bash scripts/package-macos.sh`，从访达打开生成的 `target/BongoDSH Preview.app`。脚本对完整应用包签名并校验。权限列表未列出应用时，使用面板中的「在访达中显示应用」，再在系统输入监控页面点击「+」手动添加同一个应用。重新构建后可能需要重新授权，详细流程见 [全局键盘监听](docs/global-keyboard.md)。

未授权、撤销权限或监听失败时显示原因并回退到窗口输入，不会反复弹出申请。暂停、演示和退出会停止监听；切换输入源、睡眠和监听中断会重置双手。macOS 安全输入期间暂停全局反馈，系统解除限制后恢复。全局模式选择暂不持久化，每次启动由用户开启。

本轮在 macOS Apple Silicon 上完成本地回归与未授权降级检查，Windows x86_64 MSVC 完成目标编译检查。已授权的跨应用输入、权限撤销、睡眠唤醒和 Windows 实机运行仍需人工验收。实现说明、限制与验收步骤见 [全局键盘监听](docs/global-keyboard.md)。

## 目录

```text
crates/
  bongodsh-core/     输入状态机、反馈结果与双手弹簧动画
  bongodsh-app/      桌面窗口、原生绘制、窗口和全局键盘适配、模拟演示
docs/
  project-baseline.md  产品基调、技术方向、分期和验收
  global-keyboard.md   全局输入、权限流程与平台验证边界
  rust-skills.md       Rust skills 候选与安装命令
.github/workflows/ci.yml  macOS 与 Windows 编译检查
AGENTS.md               项目开发约定
```

## 项目方向与参考

完整约定见 [项目基调](docs/project-baseline.md)，技能候选见 [Rust skills 清单](docs/rust-skills.md)。

- [dsh-bongo-fish](https://github.com/YUsaltyfish/dsh-bongo-fish)：参考打字反馈、形象配置、输入框磁吸和任务提醒。
- [dsh-whale-bongo](https://github.com/MeteorNOX/dsh-whale-bongo)：参考双手回弹、桌面模式、点击穿透和跨屏交互。

本项目独立实现 Rust 桌面程序。当前没有复制这两个项目的代码、图片或音效；本项目的开源许可证尚未确定。
