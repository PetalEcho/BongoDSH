# 窗口与输入框定位

## 行为

- 左键拖动仍使用系统窗口拖动。开始拖动时解除旧目标，松开后查询可见窗口，以前后遮挡顺序选择目标。
- 窗口四边的吸附距离为 32 个逻辑点，角色轮廓与边缘间隔为 8 个逻辑点。保存边缘及相对位置，窗口移动、缩放后重新计算位置。
- 输入框优先于窗口边缘，垂直接近距离为 48 个逻辑点，也接受与角色轮廓相交的输入框。尝试上方、下方、右侧、左侧，只有整个挂件窗口能放入可用区域时才吸附，避免把角色夹到输入内容上。
- 前方窗口全屏时，挂件放在所在屏幕可用区域右下角，边距 24 个逻辑点。恢复窗口后回到先前位置或原吸附目标。全屏期间拖动或解除吸附会覆盖本次自动位置，下次进入全屏才重新应用默认位置。
- 目标消失、最小化、切到其他 Space、输入框不可访问或权限撤销后解除吸附。显示器断开后把挂件限制在最近的可用屏幕内。没有可用显示器信息时不移动。

设置位于角色右键菜单的「吸附与位置」。本轮设置只保留在进程内，没有新增配置文件或网络服务。键盘输入仍仅限挂件自身窗口。

## 实现边界

`positioning.rs` 负责几何计算与吸附状态，`docking.rs` 负责拖动生命周期、后台查询及窗口移动。查询携带代次，开始新拖动或更改设置后丢弃旧结果。后台线程每次只处理一个请求，系统辅助功能查询不会阻塞 UI；退出时通道断开，线程完成当前查询后释放系统对象。

macOS 通过 [CGWindowListCopyWindowInfo](https://developer.apple.com/documentation/coregraphics/cgwindowlistcopywindowinfo(_:_:)) 获取可见窗口的边界和标识。输入框使用 Accessibility 的 `AXWindows`、`AXChildren`、`AXRole`、`AXPosition`、`AXSize` 等属性，保留目标元素以跟随布局变化。只检查 `AXIsProcessTrusted`，不请求或修改授权。每个应用设置 50ms 消息超时，输入框扫描限制为 256 个节点并检查 180ms 遍历时限。单次正在进行的系统调用仍可能超过遍历时限。

macOS 坐标以主屏顶部为原点，转换 `NSScreen` 的左下原点坐标，并保留其他显示器的负坐标。挂件使用 accessory 激活策略及 `CanJoinAllSpaces`、`FullScreenAuxiliary`，不显示 Dock 图标。在 macOS 13 及以后还设置 [CanJoinAllApplications](https://developer.apple.com/documentation/appkit/nswindow/collectionbehavior-swift.struct/canjoinallapplications)，允许加入其他应用的全屏空间；更早系统的跨应用全屏显示未验收。全屏边界计算包含 [NSScreen.safeAreaInsets](https://developer.apple.com/documentation/appkit/nsscreen/safeareainsets)，识别原生全屏为摄像头区域留下的空间；旧系统不提供该属性时使用完整屏幕。已授权时额外读取匹配窗口的 `AXFullScreen`。未授权时仅靠边界判断，手动铺满整个安全区的普通窗口也会采用全屏摆放。

Windows 使用 `EnumWindows`、DWM 边界、`EnumDisplayMonitors` 和挂件 DPI。排除自身进程、隐藏、最小化、cloaked 和工具窗口；移动挂件时不激活窗口、不改变尺寸或层级。输入框通过 UI Automation 查询 Edit / ComboBox 控件，排除已知只读控件。COM 对象只存在于独立 MTA 线程，符合 [Microsoft 的 UI Automation 线程要求](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)。设置连接及事务超时，遍历限制为 256 个节点和 180ms；失效控件按无法定位处理。

输入框功能依赖目标程序暴露原生辅助功能信息。自绘界面、部分 Web 编辑器和受保护程序可能无法定位。扫描只读取控件类型、状态与边界，不查询输入值、选区文本、窗口标题或控件名称。DSH 仍需要经过验证的显式布局桥接，本轮没有实现或声明兼容该协议。

## 依赖与安全边界

增加 `raw-window-handle` 获取 eframe 已创建的原生窗口。macOS 直接使用依赖树中已有的 `core-foundation`、`core-graphics`、`objc2`、`objc2-app-kit`、`objc2-foundation` 版本，仅启用所需 AppKit / Foundation 功能。Windows 增加 Microsoft `windows` 绑定，仅启用窗口、显示器、DWM、DPI、鼠标按钮状态、COM 和辅助功能 API。所有平台绑定按目标平台编译；没有引入异步运行时。

应用默认 `deny(unsafe_code)`，仅 `platform` 模块允许原生 FFI，同时保留 `deny(unsafe_op_in_unsafe_fn)`，每处调用说明指针、线程和对象生命周期。核心 crate 的 `forbid(unsafe_code)` 与标准库依赖边界保持不变。

## 验证

2026-11-02，本地 macOS Apple Silicon、Rust 1.93.1：

- 20 个本地位置回归用例通过，涵盖吸附跟随、遮挡顺序、输入框避让、目标消失、全屏恢复、手动覆盖、负坐标、断屏召回、混合 DPI 换算、小屏幕边界及刘海安全区。
- macOS 原生枚举冒烟检查通过，读取到 16 个可见外部窗口；当前辅助功能权限未开启，没有运行真实输入框吸附验收。
- macOS 桌面运行验证通过：独立测试应用覆盖屏幕、进入原生全屏 Space 后，挂件均移到预期右下角；退出全屏后恢复原坐标。原生全屏实测包含摄像头安全区，未更改系统权限。
- `cargo fmt --all -- --check`、两端 Clippy、macOS release 构建、工作区测试与命令行模拟演示通过。
- 测试文件仅存在于忽略的 `crates/bongodsh-app/tests/`，不提交。远端 CI 无法执行这些本地用例。

Windows 仅进行交叉编译和 Clippy 检查，尚未实机验证。混合 DPI 多屏、拔屏、macOS Intel、权限动态撤销、自绘输入框及 Windows 独占全屏仍需实机验收；独占全屏是否允许桌面叠层由系统和目标程序决定。
