# BongoDSH · 打字挂件

一个陪你打字的小挂件。敲键时双手拍下、身体轻轻回弹；可以贴在 DSH 输入框上沿，也可以拖到桌面，吸附在其他程序的窗口边缘。

当前源码的桌面版和插件版共用以下五款角色。水手鱼、睡衣鱼和魔法师鱼已按基础大肥鱼重绘，统一为长发、闭眼微笑和低趴造型。点击预览可查看当前身体原图；双手在运行时独立叠加，不带键盘图形。

这些角色调整尚未发布，下面的 0.2.5 安装包仍使用旧角色列表和素材。

<table>
  <tr>
    <td align="center"><a href="assets/concepts/blue-maid-prone-colored.png"><img src="assets/concepts/blue-maid-prone-colored.png" width="160" alt="基础大肥鱼"></a><br>基础大肥鱼</td>
    <td align="center"><a href="assets/characters/sailor/body-colored.png"><img src="assets/characters/sailor/body-colored.png" width="160" alt="水手鱼"></a><br>水手鱼</td>
    <td align="center"><a href="assets/characters/pajamas/body-colored.png"><img src="assets/characters/pajamas/body-colored.png" width="160" alt="睡衣鱼"></a><br>睡衣鱼</td>
  </tr>
  <tr>
    <td align="center"><a href="assets/characters/wizard/body-colored.png"><img src="assets/characters/wizard/body-colored.png" width="160" alt="魔法师鱼"></a><br>魔法师鱼</td>
    <td align="center"><a href="assets/concepts/orange-bride-prone-colored.png"><img src="assets/concepts/orange-bride-prone-colored.png" width="160" alt="克劳德"></a><br>克劳德</td>
  </tr>
</table>

图片直接引用仓库 `assets/` 中的当前彩稿。桌面版通过 [Rust 素材映射](crates/bongodsh-app/src/sprites.rs) 内嵌，插件版通过 [角色配置](plugins/dsh-bongodsh/src/characters.js) 打包为 `client.js`；两端使用相同的身体和独立双手。具体文件见 [素材清单](plugins/dsh-bongodsh/ASSETS.md)。

## 安装

当前版本 **0.2.5**，需要 DSH `0.2.1-rc.5` 或更高版本，实际安装验证使用 `0.2.1-rc.5`。

1. 按系统选择下表中的安装包，复制链接。
2. 在 DSH「插件 → 添加插件」中粘贴链接，安装后启用。
3. 重启对应的 DSH 实例并刷新页面。升级后也需要这一步。

| 平台                | 安装包                                                                                                                                   |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| macOS Apple Silicon | [dsh-bongodsh-0.2.5-darwin-arm64.tgz](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-darwin-arm64.tgz) |
| macOS Intel         | [dsh-bongodsh-0.2.5-darwin-x64.tgz](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-darwin-x64.tgz)     |
| Windows x64         | [dsh-bongodsh-0.2.5-win32-x64.tgz](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-win32-x64.tgz)       |

各平台安装包都包含页面插件、全部角色素材和桌面程序，无需安装 Rust，也无需先下载再本地导入。

也可以从命令行安装。以 macOS Apple Silicon 为例：

```sh
dsh plugin --profile web add https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-darwin-arm64.tgz
```

其他平台替换为表中对应链接。`web` 是 profile 名称，按自己的 DSH 实例调整。

## 功能

- 打字反馈：双手随按键动作，身体轻轻回弹；手部和身体的动作幅度可以分别调整。
- 五款角色：在设置中随时切换，角色大小、暂停状态和动作幅度保存在本机。
- 跟随输入框：角色贴近 DSH 输入框上沿，也可以拖动到页面中的其他位置。
- 桌面模式：拖出 DSH 窗口即可移到桌面，靠近其他窗口边缘时可吸附；支持返回 DSH。
- 跨应用反馈：主动开启「全局键盘」后，在其他程序中打字也能触发动作，输入模式会在重启后恢复。

页面与桌面会同步角色、大小、暂停状态和两项动作幅度。页面位置及「跟随输入框」设置单独保留。

## 操作与设置

页面中点击或右键角色打开设置；桌面上右键打开。按 Escape 收起面板。页面也支持用 Tab 聚焦角色、Enter 打开设置。

| 想做什么           | 操作                                                                      |
| ------------------ | ------------------------------------------------------------------------- |
| 换个角色           | 在设置中选择角色                                                          |
| 调整大小           | 拖动「角色大小」滑块，范围为 100–220 px                                  |
| 调整动作           | 分别设置「手部幅度」和「身体幅度」，范围为 0–200%；0% 关闭对应部位的动作 |
| 暂时停下           | 开启「暂停动作」                                                          |
| 贴住输入框         | 开启「跟随输入框」；找不到输入框时会退回页面边缘                          |
| 移到桌面           | 将角色拖出 DSH 窗口，或点击「移到桌面」                                   |
| 贴住其他窗口       | 在桌面模式开启「吸附窗口」，再将角色拖近窗口边缘                          |
| 回到页面           | 点击「返回 DSH」                                                          |
| 恢复位置           | 点击「恢复位置」；页面回到默认位置，桌面回到本轮拖出的起点                |
| 响应其他程序的按键 | 展开「桌面选项」，开启「全局键盘」                                        |

### 全局键盘与隐私

首次使用默认只响应窗口内输入。「全局键盘」需要手动开启，macOS 还需要授予系统输入监控权限。选择会保存在本机，重启或再次拖出后恢复，但不会自动申请系统权限。

缺少权限或监听失败时暂用窗口输入，系统限制期间暂停全局反馈；设置面板会显示原因，并保留原有选择。

只处理物理按键，不读取、记录或上传输入文字。页面模式只响应当前页面，密码框和 Ctrl / Command / Alt 快捷键不触发动作。桌面吸附依据窗口位置和尺寸，不识别其他程序的输入框。

当前不支持音效、任务提醒、自定义素材、托盘或点击穿透。DSH 退出或禁用插件时，会关闭由它启动的桌面角色。

## 本地开发

桌面程序使用 Rust，DSH 插件使用无生产依赖的 JavaScript。

- Rust **1.99.1**，由 `rust-toolchain.toml` 固定。
- Node.js **22.19+**，插件无需 `npm install`。
- macOS 需要 Xcode Command Line Tools；Windows 需要 MSVC 工具链和 Visual Studio C++ Build Tools。

在仓库根目录启动独立桌面版：

```sh
cargo run --locked --bin bongodsh
```

独立启动默认显示基础大肥鱼，右键可切换五款角色、调整大小和动作幅度，点击「退出挂件」关闭。它与 DSH 页面和拖出的桌面角色共用同一套 PNG；独立启动的角色设置仅在本次运行有效，键盘输入模式会保存在本机。插件的本地安装、构建与预览见 [插件开发说明](plugins/dsh-bongodsh/README.md)。

提交前检查 Rust 工程：

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

构建当前平台完整包、通用页面包和校验文件：

```sh
cd plugins/dsh-bongodsh
npm run release
```

产物位于 `plugins/dsh-bongodsh/dist/`。命令会校验版本、生成并核对客户端与 PNG、重建当前平台的桌面程序，再生成安装包；不会自动上传或发布。

### 项目结构

| 目录                      | 内容                             |
| ------------------------- | -------------------------------- |
| `crates/bongodsh-core/` | 平台无关的输入状态机和动作规则   |
| `crates/bongodsh-app/`  | 桌面窗口、角色绘制和系统输入适配 |
| `plugins/dsh-bongodsh/` | DSH 插件、客户端源码和打包脚本   |
| `assets/`               | 角色身体、双手彩稿及历史线稿     |
| `docs/releases/`        | 版本更新与安装说明               |


## 更新记录

[0.2.5](docs/releases/0.2.5.md) 使用 Rust 1.99.1 重建三平台桌面包。近期更新还包括设置面板精简、滑块样式统一、角色精修、跨程序窗口吸附、独立动作幅度和全局键盘偏好保存。可在 [Releases](https://github.com/PetalEcho/BongoDSH/releases) 查看各版本安装包。

## 参考与许可

项目原创代码、脚本、配置和文档文字采用 [MIT 许可证](LICENSE)，允许修改、分发和商业使用，须保留版权与许可声明。第三方依赖沿用各自许可证。

交互参考 [dsh-bongo-fish](https://github.com/YUsaltyfish/dsh-bongo-fish) 和 [dsh-whale-bongo](https://github.com/MeteorNOX/dsh-whale-bongo)，未复制其代码或音效。
