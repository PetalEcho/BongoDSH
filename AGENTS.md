# BongoDSH 项目约定

## 产品与范围

- BongoDSH 是使用 Rust 实现的独立桌面打字挂件，目标平台为 macOS 和 Windows。
- DSH 指参考仓库所使用的 DeepSeek Harness。后续由轻量 DSH 插件传递布局和任务事件，桌面进程可独立运行。
- 项目基调、分期和平台边界见 `docs/project-baseline.md`。当前已有 egui/eframe 桌面原型，只响应窗口内按键，全局监听和 DSH 接入仍未实现。
- 角色小巧、低干扰，主要操作通过右键或托盘进入。设置面板优先展示预览和常用设置，不使用超大标题或口号式 Hero。
- 用户明确要求角色不配键盘图形。保持角色与双手的独立剪影。

## Rust 实现

- 使用 Rust stable、Edition 2024；最低版本声明在根 `Cargo.toml`。
- `bongodsh-core` 维护平台无关状态与规则，不访问窗口、文件系统、网络或系统键盘。
- `bongodsh-app` 负责组装应用。后续 macOS、Windows API 放在独立平台模块或 crate，按目标平台编译。
- 桌面使用 eframe 0.34.3，启用 `glow` 和 `default_fonts`；核心仍只依赖标准库。新增依赖前说明用途、平台支持和引入成本，只启用需要的 features。
- 核心 crate 禁止 `unsafe`。将来原生 FFI 在隔离模块中单独制定 lint，并为每处 `unsafe` 说明安全前提。
- 可恢复错误返回 `Result`，不用 `unwrap()`、`expect()` 或 panic 处理用户输入和权限失败。
- 事件循环按需唤醒；不要在输入回调中执行磁盘、网络或音频解码任务。

## 输入与平台边界

- 核心只接收物理按键标识和按下、松开、重置事件；不接收、记录或上传输入文本。
- 输入源互斥启用，切换、断连、暂停、休眠或权限失效时重置状态。快捷键过滤不能丢弃已经接收的按键对应的松开事件。
- 模拟输入、应用内输入、全局输入的能力必须分开说明。macOS 全局输入权限由用户主动开启。
- 透明、置顶、点击穿透、多显示器和 DPI 要分别验证两个平台，不以单端编译通过代替运行验证。
- DSH 输入框位置来自显式桥接协议，不能仅靠桌面窗口位置推断；桥接协议未经验证前不声称支持现有 DSH 客户端。
- 图片和音效的许可独立记录。参考项目的素材不能仅依据代码 MIT 许可复制进来。

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
- 提交 `Cargo.lock`。仅按明确文件路径暂存，提交前检查暂存清单与差异。
- 推送前逐个检查待推送提交；任一提交包含测试文件变更时停止该推送。
- 不自动提交、推送、发布或更改系统权限。记录实际运行平台，未运行的检查明确标注。
