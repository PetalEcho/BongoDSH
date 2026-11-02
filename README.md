# BongoDSH

使用 Rust 开发的桌面打字挂件，目标为 macOS 和 Windows。角色随敲键做出双手动作，可独立运行；后续通过轻量插件与 DSH 连接，跟随输入框并响应任务事件。

## 当前进度

已实现 egui/eframe 桌面原型，包含透明置顶窗口、拖动吸附、全屏摆放和双手按键动画。当前只响应挂件窗口内的按键，尚未接入全局键盘监听、音效或 DSH 插件。

| 部分 | 当前状态 |
| --- | --- |
| 双手交替、长按去重、多键保持、断连重置 | 已实现核心规则 |
| 独立桌面窗口与角色动画 | 已实现 egui/eframe 0.34.3 原型 |
| macOS / Windows 全局输入与权限 | 规划 |
| 角色拖动、右键菜单、暂停与退出 | 已实现 |
| 窗口边缘吸附与位置跟随 | 已接入 macOS / Windows 原生窗口 API |
| 通用输入框吸附 | 已接入辅助功能 / UI Automation；依赖权限和目标程序暴露控件 |
| 全屏右下角摆放、退出全屏恢复位置 | 已实现 |
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

点击角色后输入字母、数字或标点即可看到双手动作。按住保持，松开回弹；拖动角色移动窗口，右键菜单可演示动作、暂停、切换置顶、调整吸附或退出。角色不配键盘图形。透明背景当前仍会接收鼠标，不等于点击穿透。

将角色拖近其他窗口边缘后松开，会吸附并跟随该窗口移动、缩放。拖近可识别的输入框时优先吸附输入框，优先放在上方，空间不足时尝试下方或侧边。再次拖动即可解除旧吸附；目标关闭、最小化、隐藏或输入框定位失效时也会解除。

前方应用全屏时，挂件默认移到该屏幕可用区域右下角，留出 24 个逻辑点的边距，避开 Dock 或任务栏；退出全屏后恢复原位置或吸附关系。全屏期间手动拖动可以覆盖本次自动位置。右键的「吸附与位置」可关闭自动吸附、关闭全屏摆放、解除吸附或召回到当前屏幕右下角。

macOS 的输入框定位需要用户在「系统设置 → 隐私与安全性 → 辅助功能」授权实际运行挂件的程序，未授权时仍可吸附窗口。程序不会弹出授权请求或更改权限，只查询控件类型、可用状态和边界，不读取文本值。Windows 使用 UI Automation；自绘控件、未暴露辅助功能树或权限较高的程序可能无法识别输入框。DSH 的专用输入框协议仍未接入，不承诺兼容现有 DSH 客户端。

窗口查询在后台线程执行，吸附跟随约每 200ms 更新，空闲时约每 750ms 检查全屏变化。macOS 原生调用使用屏幕逻辑点，Windows 使用桌面物理像素并按挂件 DPI 换算间距。macOS 配置为 accessory 应用以随全屏 Space 显示，因此不显示 Dock 图标，可用角色右键菜单退出。具体运行验证与待验收项目见 [定位说明](docs/window-positioning.md)。

切换到其他应用后会清空按键状态，不监听其他应用的输入。中文输入法组合过程尚未专项验证，部分 IME 会吞掉窗口按键事件；不会根据输入文本补造按键。中文界面尝试读取系统字体，找不到字体时使用英文提示。

本地回归用例位于被 Git 忽略的 `crates/bongodsh-core/tests/` 和 `crates/bongodsh-app/tests/`，遵循测试不提交的约定。新克隆仓库不包含这些用例，`cargo test` 通过不能视为已执行相同回归覆盖。

## 目录

```text
crates/
  bongodsh-core/     输入状态机、反馈结果与双手弹簧动画
  bongodsh-app/      桌面窗口、原生绘制、窗口按键适配、吸附与平台定位
docs/
  project-baseline.md  产品基调、技术方向、分期和验收
  rust-skills.md       Rust skills 候选与安装命令
  window-positioning.md  吸附规则、系统 API 边界与验证记录
.github/workflows/ci.yml  macOS 与 Windows 编译检查
AGENTS.md               项目开发约定
```

## 项目方向与参考

完整约定见 [项目基调](docs/project-baseline.md)，技能候选见 [Rust skills 清单](docs/rust-skills.md)。

- [dsh-bongo-fish](https://github.com/YUsaltyfish/dsh-bongo-fish)：参考打字反馈、形象配置、输入框磁吸和任务提醒。
- [dsh-whale-bongo](https://github.com/MeteorNOX/dsh-whale-bongo)：参考双手回弹、桌面模式、点击穿透和跨屏交互。

本项目独立实现 Rust 桌面程序。当前没有复制这两个项目的代码、图片或音效；本项目的开源许可证尚未确定。
