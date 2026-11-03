# BongoDSH · 打字挂件

BongoDSH 是一个随按键动作的小角色：双手拍下，身体轻轻回弹。它可以停在 DeepSeek Harness（DSH）的输入框上沿，也可以拖到桌面，贴在其他程序的窗口边缘。桌面程序使用 Rust，可独立运行，支持 macOS 和 Windows。

## 角色预览

当前源码提供基础大肥鱼、水手鱼、睡衣鱼、魔法师鱼和克劳德。角色没有键盘图形，身体与双手分别绘制，手部和身体的动作幅度可以独立调整。

<table>
  <tr>
    <td align="center"><img src="assets/concepts/blue-maid-prone-colored.png" width="160" alt="基础大肥鱼"><br>基础大肥鱼</td>
    <td align="center"><img src="assets/characters/sailor/body-colored.png" width="160" alt="水手鱼"><br>水手鱼</td>
    <td align="center"><img src="assets/characters/pajamas/body-colored.png" width="160" alt="睡衣鱼"><br>睡衣鱼</td>
  </tr>
  <tr>
    <td align="center"><img src="assets/characters/wizard/body-colored.png" width="160" alt="魔法师鱼"><br>魔法师鱼</td>
    <td align="center"><img src="assets/concepts/orange-bride-prone-colored.png" width="160" alt="克劳德"><br>克劳德</td>
  </tr>
</table>

以上展示的是当前源码中的角色。角色移除、更名，以及水手鱼、睡衣鱼和魔法师鱼的重绘尚未发布，下方的 0.2.5 安装包不包含这些更新。

## 安装到 DSH

安装包版本为 **0.2.5**。插件要求 DSH **0.2.1-rc.5 或更高版本**，已完成的安装验证使用 0.2.1-rc.5。

1. 复制对应平台的安装包链接。
2. 在 DSH「插件 → 添加插件」中粘贴链接，安装并启用。
3. 重启对应的 DSH 实例，再刷新页面。升级后也需要这一步。

| 平台                | 安装包                                                                                                          |
| ------------------- | --------------------------------------------------------------------------------------------------------------- |
| macOS Apple Silicon | [下载 0.2.5](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-darwin-arm64.tgz) |
| macOS Intel         | [下载 0.2.5](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-darwin-x64.tgz)   |
| Windows x64         | [下载 0.2.5](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-win32-x64.tgz)    |

完整包包含页面插件、角色素材和对应平台的桌面程序，无需安装 Rust。macOS 包尚未经过 Developer ID 签名和公证；Intel Mac 与 Windows x64 的界面尚未实机验证。

也可以通过命令行安装。以 macOS Apple Silicon 为例，`web` 按实际 DSH profile 名称调整，其他平台替换为对应安装包链接：

```sh
dsh plugin --profile web add https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.5/dsh-bongodsh-0.2.5-darwin-arm64.tgz
```

其他版本的安装包可在 [Releases](https://github.com/PetalEcho/BongoDSH/releases) 查看。

## 使用

页面中点击或右键角色打开设置，桌面上使用右键；按 Escape 收起面板。页面还支持用 Tab 聚焦角色、Enter 打开设置。

| 操作               | 方法                                                                      |
| ------------------ | ------------------------------------------------------------------------- |
| 切换角色           | 在设置中选择角色                                                          |
| 调整大小           | 拖动「角色大小」，范围为 100–220 px                                      |
| 调整动作           | 分别设置「手部幅度」和「身体幅度」，范围为 0–200%；0% 停止对应部位的动作 |
| 暂停反馈           | 开启「暂停动作」                                                          |
| 跟随 DSH 输入框    | 开启「跟随输入框」；找不到输入框时退回页面边缘                            |
| 移到桌面           | 将角色拖出 DSH 窗口，或点击「移到桌面」                                   |
| 吸附其他窗口       | 在桌面模式开启「吸附窗口」，再将角色拖近窗口边缘                          |
| 返回页面           | 在桌面设置中点击「返回 DSH」                                              |
| 恢复位置           | 点击「恢复位置」；页面回到默认位置，桌面回到本轮拖出的起点                |
| 响应其他程序的按键 | 在桌面设置中展开「桌面选项」，开启「全局键盘」                            |

角色、大小、暂停状态和两项动作幅度保存在本机，并在页面与拖出的桌面角色之间同步。页面位置及「跟随输入框」设置单独保留。

DSH 退出或禁用插件时，会关闭由它启动的桌面角色。独立启动的桌面程序不依赖 DSH。

### 输入模式与隐私

页面模式只响应当前页面的按键，不需要系统键盘权限。密码框及常规 Ctrl / Command / Alt 快捷键不触发动作。

桌面首次使用默认只响应窗口内输入。需要在其他程序中打字时触发动作，可主动开启「全局键盘」。macOS 首次开启需要输入监控权限；输入模式会保存在本机，重启或再次拖出后恢复，但恢复时不会自动申请权限。

缺少权限或监听失败时，设置面板会显示原因并保留原有选择，暂用窗口输入。系统限制期间暂停全局反馈。

两种模式都只处理物理按键，不读取、记录或上传输入文字，也不按输入法最终上屏字数补造动画。桌面吸附只使用窗口位置与尺寸，不识别其他程序的输入框。DSH 页面定位依赖输入框标记，原生输入框定位和任务事件尚未接入。

目前没有音效、任务提醒、自定义素材、托盘或点击穿透功能。

## 从源码运行

开发工具要求：

- Rust 1.99.1，项目已固定工具链版本。
- Node.js 22.19+，用于插件构建与预览；插件无生产依赖，无需 `npm install`。
- macOS 需要 Xcode Command Line Tools；Windows 需要 MSVC 工具链和 Visual Studio C++ Build Tools。

### 独立桌面版

在仓库根目录运行：

```sh
cargo run --locked --bin bongodsh
```

默认显示基础大肥鱼。右键可切换角色、调整大小和动作幅度，点击「退出挂件」关闭。独立启动时，角色设置仅在本次运行有效，键盘输入模式会保存在本机。

### DSH 插件与页面预览

在仓库根目录构建桌面程序，并将插件目录添加到对应的 DSH profile：

```sh
cargo build --workspace --release --locked
dsh plugin --profile web add ./plugins/dsh-bongodsh
```

本地目录安装后须保留该目录。修改插件源码或素材后，重新构建并检查客户端；修改素材还需要重建桌面程序。

以下命令从仓库根目录进入插件目录执行：

```sh
cd plugins/dsh-bongodsh
npm run build
npm run check
npm run preview
```

预览仅监听 `127.0.0.1`，手动打开终端输出的网址。它用于检查页面效果，不能代替真实 DSH 安装和桌面交互验证。

### 检查与打包

Rust 改动在仓库根目录运行：

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

本地回归测试不随仓库提交，因此检出代码后的 `cargo test` 和 CI 不代表执行了这些用例。桌面交互仍需在对应平台实机验证。

在插件目录执行 `npm run release`，生成当前平台完整包、通用页面包和 SHA-256 校验文件。命令会校验版本、构建并核对客户端与图片、重建当前平台桌面程序，产物放在插件目录的 `dist/` 下，不会自动上传或发布。

## 项目背景

BongoDSH 的交互参考了 [dsh-bongo-fish](https://github.com/YUsaltyfish/dsh-bongo-fish) 和 [dsh-whale-bongo](https://github.com/MeteorNOX/dsh-whale-bongo)，未复制它们的代码或音效。项目使用 Rust 实现独立桌面挂件，通过无生产依赖的 JavaScript 插件连接 DSH 页面与桌面。

## 许可

项目原创代码、脚本、配置和文档文字采用 MIT 许可证，使用及分发时须保留版权与许可声明。第三方依赖沿用各自许可证。
