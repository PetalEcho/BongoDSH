# BongoDSH DSH 插件

六款彩色打字角色，可贴在 DSH 输入框上沿，也可拖出为桌面挂件。当前版本 0.2.4。

## 安装

从 [GitHub Release](https://github.com/PetalEcho/BongoDSH/releases/tag/v0.2.4) 下载：

| 文件 | 内容 |
| --- | --- |
| `dsh-bongodsh-0.2.4-darwin-arm64.tgz` | macOS Apple Silicon 完整包，含页面、桌面程序和全部素材 |
| `dsh-bongodsh-0.2.4-web.tgz` | 通用页面包，含全部素材，不含桌面程序 |
| `SHA256SUMS.txt` | 安装包校验值 |

需要 DSH `>=0.2.1-rc.5`，实际验证版本为 `0.2.1-rc.5`。在「插件 → 添加插件」填写安装包的绝对路径，安装后启用。命令行安装：

```sh
dsh plugin --profile web add /absolute/path/dsh-bongodsh-0.2.4-darwin-arm64.tgz
```

使用页面包时替换文件名；profile 按 DSH 实例调整。安装或升级后重启 DSH 并刷新页面。禁用或移除插件会清理页面角色和监听，用户设置保留。

当前 Release 不包含 Windows 或 Intel Mac 桌面程序。macOS 完整包未经过 Developer ID 签名和公证。页面模式不需要 Rust 或系统键盘权限。

## 设置与操作

| 设置 | 作用 |
| --- | --- |
| 角色 | 切换基础女仆鱼、水手装、睡衣装、魔法师装、长发蓝女仆和克劳德 |
| 暂停动作 | 停止动作和按键反馈 |
| 跟随输入框 | 页面角色贴近输入框，找不到时退回页面边缘 |
| 吸附窗口 | 桌面角色靠近其他窗口边缘后吸附 |
| 角色大小 | 100–220 px，步长 10 px |
| 手部幅度 / 身体幅度 | 独立调整 0–200%，0% 停止对应部位动作 |
| 恢复位置 | 页面回到默认位置，桌面回到本轮拖出的起点 |
| 移到桌面 / 返回 DSH | 切换页面和桌面模式 |

页面角色可点击或右键打开设置，桌面角色使用右键；Escape 收起。页面可用 Tab 聚焦角色、Enter 打开设置。六款角色使用独立身体与双手图层，无键盘图形；双手置于身体前侧并避开脸部。

角色、大小、暂停和动作幅度保存在本地，并在页面与桌面之间同步。全局键盘在「桌面选项」中开启，选择保存在本机，重启或再次拖出后恢复。macOS 首次使用需要输入监控权限；缺少权限时显示原因，保留原有选择，不自动申请授权。

## 输入与桌面边界

页面只响应当前页面的物理按键，密码框和 Ctrl / Command / Alt 快捷键不触发动作。桌面开启全局键盘后才响应其他程序。两种模式均不读取、保存或上传输入文字，不根据输入法上屏字数补造动画。

窗口失焦、暂停或监听中断时清空按住状态。DSH 退出或禁用插件会回收它启动的桌面进程；刷新页面可重新连接已启动的角色。独立运行的桌面原型不依赖 DSH。

页面定位依赖 DSH 的输入框标记；桌面吸附只读取窗口几何，不定位其他程序的输入框。当前没有声音、任务提醒、自定义素材、托盘或点击穿透。协议见 [桌面桥接](https://github.com/PetalEcho/BongoDSH/blob/v0.2.4/docs/desktop-bridge.md)，权限说明见 [全局键盘](https://github.com/PetalEcho/BongoDSH/blob/v0.2.4/docs/global-keyboard.md)。

## 从源码运行

插件使用 Node.js 22.19+，无生产依赖，也无需 `npm install`。从仓库根目录安装本地目录：

```sh
dsh plugin --profile web add ./plugins/dsh-bongodsh
```

本地目录安装需要保留原目录。桌面功能需要 Rust stable 1.93+；macOS 使用 Xcode Command Line Tools，Windows 使用 MSVC 工具链。从仓库根目录运行 `cargo build --workspace --release --locked`，宿主会查找 `target/release/bongodsh` 或 `bongodsh.exe`。

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

两个包都包含宿主桥接、客户端、六款角色的十二个去重 PNG、语言信息、README 和素材说明。完整包额外包含当前平台的桌面程序；其他平台的旧构建不会混入。测试、源码、预览、线稿和用户设置均不打包。构建会核对 PNG 字节，不重新压缩原图。

如只需刷新本机桌面组件，使用 `npm run prepare:native`。它只更新当前平台的 `native/`。修改素材后必须重新生成客户端并重建原生程序。

本地回归放在被忽略的 `tests/`，存在时可运行 `node --test tests/*.test.mjs`。CI 校验语法、生成产物与包清单，不包含未提交的回归用例。实际平台和历史检查见 [验证记录](https://github.com/PetalEcho/BongoDSH/blob/v0.2.4/docs/plugin-validation.md)。

## 素材

未复制参考项目的代码或音效。角色由用户指定参考图和附件生成，不能仅依据参考项目的代码许可证判断素材许可。详见随包的 [ASSETS.md](ASSETS.md)。项目许可证尚未确定，包声明为 `UNLICENSED` 和 `private`。
