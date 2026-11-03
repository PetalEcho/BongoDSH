# BongoDSH DSH 插件

五款彩色打字角色，可贴在 DSH 输入框上沿，也可拖出为桌面挂件。版本 0.2.6 包含最新角色素材、对话 Tab 显隐和可拖动设置窗口。

最新角色预览见 [项目 README](https://github.com/PetalEcho/BongoDSH/blob/main/README.md)。页面插件、拖出的桌面窗口和独立桌面版均引用根目录 `assets/` 中的同一套身体与双手；源码文件对应关系见 [素材清单](ASSETS.md)。

原创代码采用 [MIT 许可证](LICENSE)。角色美术及内嵌图片不在 MIT 授权范围内，来源与使用边界见 [ASSETS.md](ASSETS.md)。

## 安装

需要 DSH `>=0.2.1-rc.5`，实际验证版本为 `0.2.1-rc.5`。在「插件 → 添加插件」粘贴对应平台的 GitHub Release 链接，安装后启用。macOS Apple Silicon 使用：

```text
https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.6/dsh-bongodsh-0.2.6-darwin-arm64.tgz
```

| 平台 | Release 安装链接 |
| --- | --- |
| macOS Apple Silicon | [dsh-bongodsh-0.2.6-darwin-arm64.tgz](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.6/dsh-bongodsh-0.2.6-darwin-arm64.tgz) |
| macOS Intel | [dsh-bongodsh-0.2.6-darwin-x64.tgz](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.6/dsh-bongodsh-0.2.6-darwin-x64.tgz) |
| Windows x64 | [dsh-bongodsh-0.2.6-win32-x64.tgz](https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.6/dsh-bongodsh-0.2.6-win32-x64.tgz) |

命令行也可直接添加链接：

```sh
dsh plugin --profile web add https://github.com/PetalEcho/BongoDSH/releases/download/v0.2.6/dsh-bongodsh-0.2.6-darwin-arm64.tgz
```

其他平台替换为表中对应链接；profile 按 DSH 实例调整。安装或升级后重启 DSH 并刷新页面。禁用或移除插件会清理页面角色和监听，用户设置保留。

完整包包含页面、桌面程序和全部角色素材，无需安装 Rust。macOS 完整包未经过 Developer ID 签名和公证。Intel Mac 和 Windows x64 界面尚未实机验证。页面模式不需要系统键盘权限。

## 设置与操作

| 设置 | 作用 |
| --- | --- |
| 角色 | 切换基础大肥鱼、水手鱼、睡衣鱼、魔法师鱼和克劳德 |
| 暂停动作 | 停止动作和按键反馈 |
| 跟随输入框 | 页面角色贴近输入框，关闭后可自由放置 |
| 吸附窗口 | 桌面角色靠近其他窗口边缘后吸附 |
| 角色大小 | 100–220 px，步长 10 px |
| 手部幅度 / 身体幅度 | 独立调整 0–200%，0% 停止对应部位动作 |
| 恢复位置 | 页面回到默认位置，桌面回到本轮拖出的起点 |
| 移到桌面 / 返回 DSH | 切换页面和桌面模式 |

默认角色为基础大肥鱼。旧设置中的 `fish-maid` 自动回退为 `blue-maid`，其余设置保留。

页面角色可点击或右键打开设置，桌面角色使用右键；Escape 收起。页面可用 Tab 聚焦角色、Enter 打开设置。五款角色使用独立身体与双手图层，无键盘图形；双手置于身体前侧并避开脸部。

页面和桌面的设置窗口都可按住顶部 BongoDSH 标题栏拖动，避开挂件或其他内容。页面面板会限制在可见区域内，并在本次页面会话中保留移动位置；桌面面板关闭后再次打开时回到挂件附近。

页面角色仅在对话 tab 且有可见输入框时显示；切到其他 tab 时隐藏并收起设置，返回对话后恢复。关闭「跟随输入框」也遵循此规则。移到桌面的角色不受页面 tab 切换影响。

角色、大小、暂停和动作幅度保存在本地，并在页面与桌面之间同步。全局键盘在「桌面选项」中开启，选择保存在本机，重启或再次拖出后恢复。macOS 首次使用需要输入监控权限；缺少权限时显示原因，保留原有选择，不自动申请授权。

## 输入与桌面边界

页面只响应当前页面的物理按键，密码框和 Ctrl / Command / Alt 快捷键不触发动作。桌面按选择响应吸附窗口或全局物理按键。两种模式均不读取、保存或上传输入文字，不根据输入法上屏字数补造动画。

桌面的「吸附窗口」输入仅在当前吸附窗口获得焦点时生效，同一应用的其他窗口也不会触发。未吸附、更换目标、目标失焦或关闭时清空按键状态；「全局键盘」响应所有应用，打开设置面板不会暂停全局反馈。macOS 两种模式都需要输入监控权限，窗口模式另需主动授权辅助功能，以读取焦点窗口的进程和几何信息；不读取窗口标题、控件值或输入内容。无法唯一识别焦点窗口时不响应，不自动退回全局输入。

「重新连接」旁显示连接中、已连接、系统限制、未授权或连接失败。已连接表示监听已启动；「已收到按键」表示本次连接确实收到了有效物理按键。重新连接后会清空这一反馈。若一直等待按键，请在普通文本框里试打，检查系统输入监控权限或安全输入限制；若已收到按键却没有动作，请检查手部和身体幅度是否为 0%。暂停动作时停止监听，恢复后按所选输入模式重新连接。

吸附窗口失焦、暂停或监听中断时清空按住状态。DSH 退出或禁用插件会回收它启动的桌面进程；刷新页面可重新连接已启动的角色。独立运行的桌面原型不依赖 DSH。

页面定位依赖 DSH 的输入框标记；桌面吸附只读取窗口几何，不定位其他程序的输入框。当前没有声音、任务提醒、自定义素材、托盘或点击穿透。

## 从源码运行

插件使用 Node.js 22.19+，无生产依赖，也无需 `npm install`。从仓库根目录安装本地目录：

```sh
dsh plugin --profile web add ./plugins/dsh-bongodsh
```

本地目录安装需要保留原目录。开发和发布统一使用 Rust 1.99.1；macOS 使用 Xcode Command Line Tools，Windows 使用 MSVC 工具链。从仓库根目录运行 `cargo build --workspace --release --locked`，宿主会查找 `target/release/bongodsh` 或 `bongodsh.exe`。

```sh
cd plugins/dsh-bongodsh
npm run build
npm run check
npm run preview
```

预览只监听 `127.0.0.1`，手动打开输出网址。预览使用同一份 `client.js`，不等同于真实 DSH 安装验收。

## 打包

```sh
cd plugins/dsh-bongodsh
npm run release
```

命令校验 Rust 与插件版本，生成并核对 `client.js` 和内嵌 PNG，重建当前平台桌面程序。随后在 `dist/` 生成通用页面包、当前平台完整包及 `SHA256SUMS.txt`。macOS 完整包内的 App 版本与插件一致。命令不会上传或发布。

两个包都包含宿主桥接、客户端、五款角色的十个去重 PNG、语言信息、README 和素材说明。完整包额外包含当前平台的桌面程序；其他平台的旧构建不会混入。测试、源码、预览、线稿和用户设置均不打包。构建会核对 PNG 字节，不重新压缩原图。

如只需刷新本机桌面组件，使用 `npm run prepare:native`。它只更新当前平台的 `native/`。修改素材后必须重新生成客户端并重建原生程序。

macOS 本地调试全局键盘时，运行 `npm run prepare:native` 后，从 Finder 打开 `native/darwin-arm64/BongoDSH Companion.app`，Intel Mac 对应 `darwin-x64`。脚本会对完整应用包进行本地临时签名并校验；这不等同于 Developer ID 签名或公证。仅运行 `cargo build` 不会更新已有应用包。

若系统「输入监控」列表没有挂件，点击列表下的「＋」，选择上述 `.app` 并开启权限，然后退出并重新打开该应用。授权应用包后，应从同一个应用包启动；直接运行 `target/debug/bongodsh` 或 `cargo run` 的进程不能据此假定已有权限。重新构建替换本地临时签名的应用后，系统可能要求重新授权。

本地回归放在被忽略的 `tests/`，存在时可运行 `node --test tests/*.test.mjs`。CI 校验语法、生成产物与包清单，不包含未提交的回归用例。

## 素材

未复制参考项目的代码或音效。角色由用户指定参考图和附件生成，不能仅依据参考项目的代码许可证判断素材许可。详见随包的 [ASSETS.md](ASSETS.md)。原创代码采用 MIT，包声明为 `SEE LICENSE IN LICENSE` 和 `private`，角色图片及内嵌形式的许可单独说明。
