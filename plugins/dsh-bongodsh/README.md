# BongoDSH DSH 插件

直接在 DeepSeek Harness 页面内显示大肥鱼角色。安装插件后即可使用，不需要启动 Rust 桌面挂件，不需要输入监控权限，也没有额外生产依赖。默认使用基础女仆鱼，设置中可切换六个彩色角色。

## 安装

当前版本按 DSH `0.2.1-rc.5` 的客户端加载格式实现，最低版本声明为 `>=0.2.1-rc.5`。已在本机 macOS DeepSeek Harness 桌面应用中更新并启用 0.1.4，逐一验证六个角色及其配套双手在真实会话中的显示与切换。

桌面版打开「插件 → 添加插件」，填写本仓库 `plugins/dsh-bongodsh` 的绝对目录路径，安装后点击「立即启用」。本地目录安装显示为 `link:` 来源，需要保留该目录。

从仓库根目录安装本地插件：

```sh
dsh plugin --profile web add ./plugins/dsh-bongodsh
```

也可以安装打包文件：

```sh
dsh plugin --profile web add /absolute/path/dsh-bongodsh-0.1.4.tgz
```

`web` 是 Web profile。若桌面版使用其他 profile，在 DSH 插件管理页确认对应 profile 后安装；不直接修改桌面应用的内部文件。没有启用插件热加载时，安装后重启对应 DSH 实例并刷新页面。

在插件管理中禁用或移除 `dsh-bongodsh`，即可卸载角色、页面监听和布局观察器。用户设置保留在该站点的本地存储中。

## 使用

- 默认停在可见输入框上沿，找不到输入框时停靠页面右下方。布局改变后重新定位。
- 当前页面的字母、数字、标点等物理按键驱动角色动作。六个角色均配有与各自服装匹配的独立双手，叠放在身体前侧，交替轻压。长按不会重复敲击，多键保持时分别等待松开。
- 拖动角色自由放置，靠近输入框右上方后松手可重新吸附。点击或右键打开设置，键盘可用 Tab 聚焦角色、Enter 打开设置、Escape 关闭。
- 设置包含六个角色切换、暂停、跟随输入框、100–220 px 尺寸、动作演示和恢复位置。刷新页面后保留设置，同一站点的其他标签页会同步更新。升级时保留原有尺寸和位置。
- 窗口失焦、页面隐藏、暂停、卸载和重新加载时清空按住状态。启用系统「减少动态效果」时使用直接的按住姿态。

插件只读取 `KeyboardEvent.code`、连发标记和快捷键修饰状态。不读取 `key`、输入框内容或输入法组合文字，不保存按键记录。PNG 内嵌在客户端包内，不发起图片请求，也不上传数据。密码框不触发动作，Ctrl / Command / Alt 快捷键被过滤，AltGr 输入可正常反馈。中文输入法只按实际物理按键反馈，不根据候选词或上屏字数补造动画。

## 范围

这是页面插件，不在 DSH 失焦时监听其他应用。Rust 桌面版仍是独立程序；两者之间没有桥接，也没有跨进程输入框定位。当前不包含任务完成提醒、声音、自定义素材或系统托盘。

页面定位仅使用 DSH 显式的 `[data-composer-card]` 标记与 CSS 像素，不推断屏幕物理坐标。该标记属于上游实现细节；若未来 DSH 改名，角色会退回页面边缘，需更新适配。页面内 iframe 的按键不纳入当前范围。

## 开发与验证

需要 Node.js 22.19 或更高版本。无需 `npm install`，构建工具只使用 Node 标准库。

```sh
cd plugins/dsh-bongodsh
npm run build
npm run check
npm run preview
```

预览命令输出仅绑定 `127.0.0.1` 的随机端口。手动打开输出的网址；脚本不启动浏览器。预览页加载与安装包相同的 `client.js`，可以检查按键、拖动、设置、输入框重建和插件加载/卸载。它模拟客户端生命周期，不等同于 DSH 安装验收。

`src/model.js` 独立实现与 Rust 核心一致的按键规则和弹簧参数；浏览器不加载 Rust 二进制。`src/characters.js` 登记仓库根目录 `assets/` 中的彩稿，`src/art.js` 使用 SVG 图层显示 PNG 并驱动动作。构建脚本将自包含函数写入 DSH 的惰性模块格式，按内容哈希去重，将所需 PNG 原样编码为 data URI，内嵌到客户端中。`client.js` 随源码保留，安装不运行构建脚本。修改素材后也应运行 `npm run build` 与 `npm run check`。

打包时先创建 `dist` 目录，然后运行：

```sh
npm pack --pack-destination dist
```

包内包含宿主入口、客户端产物、十二个内嵌 PNG 资源、bundle patch、语言信息、本文和素材来源说明。预览、源文件、线稿、测试及 Rust 构建产物不进入安装包。代码许可证待项目所有者决定，当前以 `UNLICENSED`、`private` 声明阻止意外公开发布。

本地回归用例位于忽略的 `tests/`，有这些文件时运行 `node --test tests/model.test.mjs`。CI 检查语法、产物与源码一致性及打包清单，不代表执行了未提交的本地回归用例。

2026-11-02 在 macOS Apple Silicon / Chrome 完成旧版 14 项 Node 回归检查，包含模拟 Windows CRLF 检出后的产物校验，并验证设置保存、拖动保留输入焦点、输入框隐藏回退、重建、卸载和重复加载。另将旧版打包文件离线安装到临时目录，验证包导出，并用本机 DSH `0.2.1-rc.5` 的实际模块加载器完成导入。

2026-11-03 完成 0.1.1 构建、语法与产物一致性检查、16 项 Node 回归检查和打包清单检查。新增覆盖旧设置迁移、角色切换时清除旧手部图层，以及七个内嵌 PNG 解码后与原文件字节一致。在真实 macOS DSH 的插件页完成本地目录安装与启用，确认组件运行中，逐个切换全部六个角色，并检查聊天页和新会话页的输入框上沿定位。输入框中的测试字符未发送，随后清空。Windows、窄窗口、中文输入法和混合 DPI 实机验收仍待完成；本次未改 Rust 源码，未重复运行 Rust 检查。

0.1.2 将三套装扮的双手移到身体前侧，不再缩小身体区域给双手留出底部空间；按下位移由 8 缩小到 3 个 SVG 坐标单位。构建检查与 16 项回归检查通过，已在本机 DSH 重新启用 0.1.2 并检查水手、睡衣、魔法师装的实际位置。PNG 原文件未改动。

0.1.4 合入远端 `main` 的六款配套手部，补齐基础女仆鱼、长发蓝女仆和克劳德的左右手，同时保留 0.1.2 的手部位置和 0.1.3 的角色名称。构建检查和 16 项回归检查通过，内嵌的十二个 PNG 均与项目素材一致。已在本机 DSH 重新启用 0.1.4 并检查六款角色的显示。

## 接入依据与素材

- [DSH 客户端模块文档](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/client/modules/README.md)：`dsh.client`、`./client` 导出与惰性模块加载。
- [DSH 插件管理文档](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/boot/plugin-manager/README.md)：bundle 安装与生命周期。
- [dsh-whale-bongo 包声明](https://github.com/MeteorNOX/dsh-whale-bongo/blob/main/package.json)：仅核对现有社区插件的 bundle / client 结构。

未复制参考项目代码或音效。六个角色使用用户指定会话中生成或编辑的 PNG，保留原图与线稿，无键盘图形。角色生成参考了上游图片和用户附件，不能仅根据代码 MIT 许可认定素材可自由分发。详细记录见随包的 [ASSETS.md](ASSETS.md)，仓库接入说明见 [DSH 插件角色接入](../../docs/dsh-character-assets.md)。
