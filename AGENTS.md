# BongoDSH 项目约定

## 产品与范围

- BongoDSH 是使用 Rust 实现的独立桌面打字挂件，目标平台为 macOS 和 Windows。
- DSH 指参考仓库所使用的 DeepSeek Harness。`plugins/dsh-bongodsh/` 提供页面角色和拖出桌面的宿主桥接；原生输入框定位与任务事件尚未接入。使用边界见 `plugins/dsh-bongodsh/README.md`。
- 当前已有 egui/eframe 桌面原型与 macOS / Windows 全局键盘适配，首次默认窗口输入，用户主动选择的输入模式保存在本机，重启或重新拖出后恢复；恢复全局模式不自动申请系统权限。权限、安装和兼容边界见 `plugins/dsh-bongodsh/README.md`。
- 角色小巧、低干扰，主要操作通过右键或托盘进入。设置面板优先展示预览和常用设置，不使用超大标题或口号式 Hero。
- 用户明确要求角色不配键盘图形。身体与双手保持独立图层，双手叠放在身体前侧，不在身体下方单独留一排空间。

## Rust 实现

- 使用 Rust stable、Edition 2024；最低版本声明在根 `Cargo.toml`。
- `bongodsh-core` 维护平台无关状态与规则，不访问窗口、文件系统、网络或系统键盘。
- `bongodsh-app` 负责组装应用。后续 macOS、Windows API 放在独立平台模块或 crate，按目标平台编译。
- 桌面使用 eframe 0.34.3，启用 `glow` 和 `default_fonts`；核心仍只依赖标准库。新增依赖前说明用途、平台支持和引入成本，只启用需要的 features。
- 核心 crate 以 `forbid(unsafe_code)` 禁止 `unsafe`。工作区默认 `deny(unsafe_code)`，仅应用层 `global_input/{macos,windows}.rs` 和 `platform/{macos,windows}.rs` 放行原生 FFI，并启用 `unsafe_op_in_unsafe_fn` 和 `clippy::undocumented_unsafe_blocks` 检查，为每处 `unsafe` 说明安全前提。
- 可恢复错误返回 `Result`，不用 `unwrap()`、`expect()` 或 panic 处理用户输入和权限失败。
- 事件循环按需唤醒；不要在输入回调中执行磁盘、网络或音频解码任务。

## 输入与平台边界

- 核心只接收物理按键标识和按下、松开、重置事件；不接收、记录或上传输入文本。
- 输入源互斥启用，切换、断连、暂停、休眠或权限失效时重置状态。快捷键过滤不能丢弃已经接收的按键对应的松开事件。
- 模拟输入、应用内输入、全局输入的能力必须分开说明。macOS 全局输入权限由用户主动开启。
- 透明、置顶、点击穿透、多显示器和 DPI 要分别验证两个平台，不以单端编译通过代替运行验证。
- DSH 输入框位置来自显式桥接协议，不能仅靠桌面窗口位置推断；桥接协议未经验证前不声称支持现有 DSH 客户端。
- 图片和音效的许可独立记录。参考项目的素材不能仅依据代码 MIT 许可复制进来。

## DSH 页面插件

- 插件使用无生产依赖的 JavaScript。拖出经 DSH 认证连接启动固定本地二进制，只传角色、尺寸、暂停、独立手部和身体幅度及归一化抓取位置，不启动额外网络监听或接收任意路径与命令。宿主管道关闭须回收子进程。
- 角色原图和线稿保存在根 `assets/`，来源见 `plugins/dsh-bongodsh/ASSETS.md`；插件通过 `src/characters.js` 登记彩稿。构建时将 PNG 原样编码并去重，内嵌到 `client.js`，兼容没有资源接口的 DSH 0.2.1-rc.5，不修改原图像素。没有独立手部素材的角色仅做身体动作。
- 修改 `plugins/dsh-bongodsh/src/` 后运行该目录的 `npm run build` 与 `npm run check`，保留生成的 `client.js`。安装包使用 `files` 白名单，预览和本地测试不打包。
- 页面坐标仅使用 CSS 像素和显式输入框标记；原生鼠标接续拖动，跨程序吸附只读取窗口几何，不读取其他程序输入框。插件卸载时清理 DOM、事件、定时器、动画帧、观察器和宿主子进程。
- 插件本地测试放在忽略的 `plugins/dsh-bongodsh/tests/`，同样不得暂存、提交或推送。

## 验证与 Git

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
cargo run --locked --bin bongodsh -- --demo
```

- 本地回归用例放在各 crate 的 `tests/`，当前两个测试目录按用户约定忽略。其他测试文件同样不得暂存、提交或推送。
- CI 只验证检出后可用的目标。测试未随源码提交时，CI 的 `cargo test` 不代表执行了本地回归用例。
- `docs/` 仅跟踪和推送 `releases/`；其他文档保留在本地并忽略。
- 提交 `Cargo.lock`。仅按明确文件路径暂存，提交前检查暂存清单与差异。
- 推送前逐个检查待推送提交；任一提交包含测试文件变更时停止该推送。
- 不自动提交、推送、发布或更改系统权限。记录实际运行平台，未运行的检查明确标注。
