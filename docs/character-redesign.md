# 三款大肥鱼换装重绘

日期：2026-11-03，Asia/Singapore。

以基础大肥鱼的彩稿 `assets/concepts/blue-maid-prone-colored.png` 为造型基准，将水手鱼、睡衣鱼、魔法师鱼改为铺开的长蓝发、闭眼微笑和低矮横向趴姿。服装分别保留水手帽与领巾、月星睡帽与条纹睡衣、星星尖帽与披风。基础大肥鱼、克劳德和历史线稿不变。

本轮使用内置 imagegen，每款分别调用一次，三次均成功；没有调用备用 CLI/API。身体不含双手、键盘或场景，现有配套手部继续独立叠加。为适配新脸型，页面与 Rust 的三款手部布局同步为中心 x=45/161、边界尺寸 34×27，沿用基础大肥鱼的位置。

## 保存文件

| 角色 | 彩色身体 | 画布尺寸 | alpha≥16 的可见范围 x/y/宽/高 |
| --- | --- | --- | --- |
| 水手鱼 | [body-colored.png](../assets/characters/sailor/body-colored.png) | 1721×914 | 45 / 11 / 1634 / 888 |
| 睡衣鱼 | [body-colored.png](../assets/characters/pajamas/body-colored.png) | 1721×914 | 51 / 24 / 1630 / 856 |
| 魔法师鱼 | [body-colored.png](../assets/characters/wizard/body-colored.png) | 1721×914 | 52 / 15 / 1626 / 884 |

生成结果原样复制到项目，没有程序裁剪、重采样或修改像素。使用项目 PNG 解码器检查三张均为非交错 8 位 RGBA，可见轮廓未触及画布边缘，并逐字节核对副本与生成结果一致。旧彩稿可从 Git 历史恢复，原线稿仍代表旧设计。

素材许可沿用 [素材来源](../plugins/dsh-bongodsh/ASSETS.md)，生成过程不赋予额外授权。

## 接入验证

- `npm run build` 和 `npm run check` 通过，客户端内嵌十个去重 PNG，字节与源文件一致。
- 在本机内置浏览器打开插件预览，逐一切换三款角色检查 160 px 下的显示，透明背景正常、轮廓完整、双手位于脸部两侧且未遮挡眼睛与嘴部。此项为本地页面预览，不代表 DSH 客户端实机验收。
- 修改的 Rust 文件格式检查、`cargo clippy --workspace --lib --bins --all-features --locked -- -D warnings` 通过。
- `npm run prepare:native` 通过，完成 macOS arm64 release 构建并刷新插件目录中的本地桌面组件。
- 全量 Rust 格式、Clippy 与测试在前一轮已被本地旧测试对已删除 `appearance.rs` 的引用阻塞，本轮没有修改这些测试，也没有重复运行全量检查。未进行原生窗口或 Windows 实机验收，未发布。

提交前从暂存区导出干净快照，在 macOS arm64 上通过 `cargo fmt --all -- --check`、全目标 Clippy、`cargo test --workspace --all-features --locked` 和 release 构建，以及插件产物一致性、旧角色设置迁移与许可证随包检查。快照不包含被忽略的本地回归测试，Cargo 实际执行 0 项测试，不能视为本地回归覆盖；原工作树中的旧测试问题仍保留。

## 实际提示词

每次图 1 为基础大肥鱼彩稿，图 2 为对应角色的旧彩稿，设置 `transparent_background: true`。图 1 决定造型、脸型、长发与画风，图 2 仅提供服装特征。三次提示词均为以下共同部分与对应服装部分拼接。

### 共同部分

```text
Use case: style-transfer. Asset: one transparent PNG BODY sprite for a small desktop typing companion. Image 1 is the PRIMARY character identity, face, hair, pose, proportions and rendering reference: the long-haired blue maid. Image 2 is the existing EDIT TARGET, used only for its specific costume identity. Redesign image 2 as a costume variant of image 1. Match image 1 closely: wide low belly-down prone pose, large head at left, small clothed body extending right, raised two-lobed blue fish tail at right, long flowing deep-blue hair with bright blue ends pooling along the bottom and behind the body, graceful thin anime outlines, restrained cel shading, warm peach cream face, pink cheeks, curved CLOSED smiling eyes with eyelashes and tiny closed curved smile. Replace image 2's round dot eyes, omega mouth and short bob. Keep the character friendly and modestly dressed. Body-only layer: absolutely NO hands, arms, fists, paws or exposed feet; hands will be independently overlaid. No keyboard, desk, floor, cast shadow, glow, scenery, text, watermark, borders, atlas or extra pieces. Output one complete character on TRUE transparent alpha with fully colored opaque interiors and clean antialiased edges, not a painted checkerboard. Wide landscape canvas about 1.88:1 matching image 1, full silhouette including hat and tail with small even safety margins.
```

### 水手鱼

```text
Costume variant: SAILOR. Replace all maid garments with image 2's white sailor cap with navy band and small round blue top ornament, ivory-white sailor uniform, navy collar with two white stripes, and bright cyan neckerchief. Translate them into the refined long-haired character's low prone pose. Keep hair fin-like side tufts from image 1, but no maid headband, lace apron or gold dress. No anchor, ship, life buoy or extra props.
```

### 睡衣鱼

```text
Costume variant: PAJAMAS. Replace all maid garments with image 2's pale lavender-blue striped pajamas, soft floppy sleeping cap with ivory trim and ivory pompom, small warm-yellow moon and star appliques and simple pale buttons. Keep long blue hair pooling around the low prone body and relaxed closed-eye smile. Keep the cap low and floppy enough to fit a wide composition. No maid headband, maid apron, pillow, blanket, bed or extra props.
```

### 魔法师鱼

```text
Costume variant: WIZARD. Replace all maid garments with image 2's soft bent-tip indigo wizard hat decorated with a few warm-gold stars, an indigo-violet cape draped over the prone body and a small gold clasp and ribbon at the neck. Keep the hat moderately low and bent so the head and body proportions stay close to image 1. Keep long blue hair pooling along the lower edge and refined closed-eye smile. No maid headband or apron, no wand, book, magic effects, floating stars or extra props.
```
