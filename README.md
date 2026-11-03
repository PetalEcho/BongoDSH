# BongoDSH

使用 Rust 开发的桌面打字挂件，目标为 macOS 和 Windows。DSH 页面插件提供六个彩色角色，可拖出为独立桌面窗口并吸附其他程序的窗口边缘。独立启动的桌面原型保留矢量小鲸鱼。

## 当前进度

已实现 egui/eframe 桌面原型，包含透明置顶窗口、角色拖动、双手按键动画和 macOS / Windows 全局键盘监听。默认只响应挂件窗口内的按键，右键可开启全局模式。DSH 页面插件支持按键动画、输入框吸附、设置保存和拖出桌面；音效和任务提醒尚未接入。

| 部分 | 当前状态 |
| --- | --- |
| 双手交替、长按去重、多键保持、断连重置 | 已实现核心规则 |
| 独立桌面窗口与角色动画 | 已实现 egui/eframe 0.34.3 原型 |
| macOS / Windows 全局输入与权限 | 已实现原生适配与 macOS 主动授权入口，实机验证边界见下文 |
| 角色拖动、右键菜单、暂停与退出 | 已实现 |
| 点击穿透、托盘与本地设置 | 规划 |
| DSH 页面插件 | 已实现独立 Web bundle，已在 macOS DSH 中安装并验证六个角色显示与切换 |
| DSH 拖出桌面、其他程序窗口边缘吸附 | 已实现；验证边界见 [桌面桥接](docs/desktop-bridge.md) |
| 原生输入框定位与任务提醒 | 后续阶段 |

## DSH 插件版

直接在 DSH 内显示角色，无需运行 Rust 程序。从仓库根目录执行：

```sh
dsh plugin --profile web add ./plugins/dsh-bongodsh
```

插件按本机 DSH `0.2.1-rc.5` 的接口实现。默认角色为基础女仆鱼，支持六个彩色角色切换、页面按键、输入框吸附、拖动、暂停、尺寸及手部和身体幅度调整与本地设置保存；不监听其他应用，不申请系统键盘权限。素材原图和线稿保存在根目录 `assets/`，来源与接入方式见 [角色接入记录](docs/dsh-character-assets.md)。安装、打包、预览及兼容边界见 [插件说明](plugins/dsh-bongodsh/README.md)。

使用拖出功能前运行 `cargo build --workspace --release --locked`。将角色拖出 DSH 窗口，或在角色设置中选择「移到桌面」，靠近其他程序窗口边缘后松手即可吸附。桌面右键面板支持与页面一致的角色切换、暂停、大小、独立手部和身体幅度及恢复位置，角色、大小、暂停及两项幅度会同步回 DSH。关闭设置不会退出挂件，选择「返回 DSH」才恢复页面角色。拖出默认仍只响应角色窗口内按键；跨应用打字反馈需要在「桌面选项」中主动开启全局键盘。页面模式无需 Rust 程序。

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

点击角色后输入字母、数字或标点即可看到双手动作。按住保持，松开回弹；拖动角色移动窗口，右键菜单可暂停、切换置顶或退出。角色不配键盘图形。透明背景当前仍会接收鼠标，不等于点击穿透。

窗口输入模式在失去焦点时清空按键状态。全局模式在其他应用打字时也驱动动画，并忽略挂件窗口重复上报的事件。两种模式都只处理物理按键，不读取输入文字，也不根据输入法组合文本补造动画。中文输入法尚未专项实机验证。中文界面尝试读取系统字体，找不到字体时使用英文提示。

本地回归用例位于被 Git 忽略的 `crates/bongodsh-core/tests/` 和 `crates/bongodsh-app/tests/`，遵循测试不提交的约定。新克隆仓库不包含这些用例，`cargo test` 通过不能视为已执行相同回归覆盖。

## 全局键盘

右键从 DSH 拖出的角色，展开「桌面选项」并选择全局键盘。独立启动的鲸鱼原型可选择「启用全局键盘」或「键盘输入设置…」。Windows 不要求管理员权限。macOS 首次使用时，在面板点击「申请输入监控权限」，按系统提示开启「隐私与安全性 → 输入监控」，然后点击「检查并重新连接」。通过终端或编辑器启动时，系统可能将权限归属到启动应用；授权后仍无法连接时，退出并重新启动挂件。

未授权、撤销权限或监听失败时显示原因并回退到窗口输入，不会反复弹出申请。暂停和退出会停止监听；切换输入源、睡眠和监听中断会重置双手。macOS 安全输入期间暂停全局反馈，系统解除限制后恢复。全局模式选择暂不持久化，每次启动由用户开启。

本轮在 macOS Apple Silicon 上完成本地回归与未授权降级检查，Windows x86_64 MSVC 完成目标编译检查。已授权的跨应用输入、权限撤销、睡眠唤醒和 Windows 实机运行仍需人工验收。实现说明、限制与验收步骤见 [全局键盘监听](docs/global-keyboard.md)。

## 目录

```text
assets/
  concepts/         三个角色的独立身体与双手，保留线稿和彩稿
  characters/       水手、睡衣、魔法师的独立身体与双手，保留线稿和彩稿
plugins/
  dsh-bongodsh/     DSH 页面插件、预编译客户端与本地预览
crates/
  bongodsh-core/     输入状态机、反馈结果与双手弹簧动画
  bongodsh-app/      桌面窗口、原生绘制、窗口和全局键盘适配、命令行模拟
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

本项目独立实现 Rust 桌面程序和 DSH 页面插件，未复制上述项目的代码或音效。页面角色素材根据用户指定的参考图和附件生成，原始参考图未复制到仓库。素材来源与许可单独记录在 [ASSETS.md](plugins/dsh-bongodsh/ASSETS.md)；本项目的开源许可证尚未确定。
