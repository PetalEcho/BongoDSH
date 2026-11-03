# BongoDSH

DSH 打字挂件，提供六款彩色角色、独立双手动画和桌面模式。角色可贴在输入框上沿，也可拖出 DSH，吸附其他程序的窗口边缘。

## 下载与安装

当前版本 **0.2.4**。从 [GitHub Releases](https://github.com/PetalEcho/BongoDSH/releases/tag/v0.2.4) 下载：

| 安装包 | 适用范围 |
| --- | --- |
| `dsh-bongodsh-0.2.4-darwin-arm64.tgz` | macOS Apple Silicon，包含页面插件、全部角色素材和桌面组件 |
| `dsh-bongodsh-0.2.4-web.tgz` | DSH 页面模式，不含桌面组件 |
| `SHA256SUMS.txt` | 两个安装包的 SHA-256 校验值 |

需要 DSH `0.2.1-rc.5` 或更高版本，实际安装验证使用 `0.2.1-rc.5`。在 DSH「插件 → 添加插件」中填写下载文件的绝对路径，安装后启用。也可以运行：

```sh
dsh plugin --profile web add /absolute/path/dsh-bongodsh-0.2.4-darwin-arm64.tgz
```

安装通用包时替换文件名。`web` 是 profile 名称，按自己的 DSH 实例调整。安装或升级后重启对应 DSH 实例并刷新页面。

完整包已包含桌面程序，无需安装 Rust。当前未提供 Windows 或 Intel Mac 桌面安装包；这些平台可以使用页面包。macOS 包未经过 Developer ID 签名和公证。

## 使用

- 页面角色可点击或右键打开设置，桌面角色使用右键；Escape 收起设置。
- 切换角色、暂停动作，调整角色大小及手部、身体幅度。幅度为 0–200%，0% 关闭对应部位的动作。
- 在页面中拖动角色自由放置，开启「跟随输入框」可贴近输入框。
- 拖出 DSH 窗口，或点击「移到桌面」。桌面模式可吸附其他窗口；点击「返回 DSH」回到页面。
- 在桌面设置中展开「桌面选项」，开启「全局键盘」，即可响应其他程序中的按键。选择会保存在本机，重启或重新拖出后自动恢复。

角色、暂停、大小及两项幅度会在页面和桌面之间同步。页面位置与输入框跟随设置单独保留。全局键盘首次需要用户开启；macOS 还需要系统输入监控授权。缺少权限或监听失败时暂用窗口输入，系统限制期间暂停全局反馈。面板显示原因，保留已保存的选择。详见 [全局键盘说明](docs/global-keyboard.md)。

只处理物理按键，不读取、记录或上传输入文字。页面模式只响应当前页面。当前没有音效、任务提醒、托盘或点击穿透。

## 0.2.4 更新

- 精简页面和桌面设置文字，保留必要的错误与权限提示。
- 统一三条滑块的细轨道、圆形手柄、填充色和操作高度。
- 插件、桌面程序和 App 版本统一，重新构建完整安装包。
- 包含此前的角色精修、手部遮挡调整、紧贴输入框、跨程序吸附、独立动作幅度和全局键盘偏好保存。

## 本地开发

Rust stable 1.93+；macOS 需要 Xcode Command Line Tools，Windows 需要 MSVC 工具链和 Visual Studio C++ Build Tools。插件构建需要 Node.js 22.19+，无需安装 Node 依赖。

```sh
cargo run --locked --bin bongodsh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

独立启动显示矢量鲸鱼原型。DSH 使用六款 PNG 角色。页面插件的开发、安装及预览见 [插件 README](plugins/dsh-bongodsh/README.md)。

构建当前平台完整包、通用页面包和校验文件：

```sh
cd plugins/dsh-bongodsh
npm run release
```

产物在 `plugins/dsh-bongodsh/dist/`。命令校验版本，生成并核对客户端与 PNG，重建桌面程序，再按平台分别打包。它只生成本地文件，不自动上传或发布。

macOS 已验证页面显示、拖出、设置同步及全局键盘偏好恢复。Windows 完成目标编译检查，尚未实机验收。详细记录见 [插件验证记录](docs/plugin-validation.md)、[桌面桥接](docs/desktop-bridge.md)。本地回归位于忽略的测试目录，新克隆仓库不含这些用例，CI 的 `cargo test` 不代表同等回归覆盖。

## 源码与素材

- `crates/bongodsh-core/`：输入状态机和动作规则。
- `crates/bongodsh-app/`：桌面窗口、绘制和系统输入适配。
- `plugins/dsh-bongodsh/`：DSH 插件、客户端源码和打包脚本。
- `assets/`：六款角色的身体、双手彩稿及线稿。
- `docs/`：协议、平台边界和验证记录。

交互参考 [dsh-bongo-fish](https://github.com/YUsaltyfish/dsh-bongo-fish) 和 [dsh-whale-bongo](https://github.com/MeteorNOX/dsh-whale-bongo)，未复制其代码或音效。角色素材由用户指定参考图和附件生成，来源与许可见 [素材说明](plugins/dsh-bongodsh/ASSETS.md)。项目许可证尚未确定。
