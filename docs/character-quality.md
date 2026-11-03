# 四款角色画质精修

日期：2026-11-03，Asia/Singapore。

本篇记录早先保留短发和圆点眼的画质精修。当前水手鱼、睡衣鱼和魔法师鱼已按基础大肥鱼的长发与笑脸重新设计，最新素材、提示词及检查见[三款换装重绘](character-redesign.md)。旧基础女仆鱼已从运行列表移除。

按用户要求，将角色列表前四款的画质向长发蓝女仆靠近。保留各自短发、圆点眼睛、闭合的小嘴、服装和横向趴姿，将粗深棕描边细化为深蓝描线，并细化头发高光、布料褶皱与阴影。长发蓝女仆只作为渲染质量参考，没有复制其长发、服装或表情。

身体继续使用独立透明 PNG，不包含双手、键盘、桌面或场景。蓝女仆、克劳德、原线稿和已有手部素材保持不变。

## 文件

| 角色 | 文件 | 输出尺寸 |
| --- | --- | --- |
| 基础女仆鱼 | [fish-maid-colored.png](../assets/concepts/fish-maid-colored.png) | 1591 × 989 |
| 水手装 | [body-colored.png](../assets/characters/sailor/body-colored.png) | 1536 × 1024 |
| 睡衣装 | [body-colored.png](../assets/characters/pajamas/body-colored.png) | 1642 × 958 |
| 魔法师装 | [body-colored.png](../assets/characters/wizard/body-colored.png) | 1613 × 975 |

基础女仆鱼输出高度比此前多 1 像素，其余尺寸不变。插件按图片自身宽高比装配身体。

## 生成与检查

四款分别使用内置 imagegen 一次完成精修，每次以对应原彩稿为编辑目标，以 [长发蓝女仆](../assets/concepts/blue-maid-prone-colored.png) 为第二张渲染参考。生成后原样复制到上述路径，没有使用脚本绘图、裁剪、锐化、重采样或修改透明通道。

逐张检查了完整造型、服装、无手身体、细线条及透明背景，并只读解码 PNG 检查尺寸和 alpha。四张均为 8 位 RGBA，alpha 为 0 的像素约占 42%–51%，alpha 大于 32 的可见像素没有触及画布边缘。四角为透明或接近透明，魔法师左下角 alpha 为 1，其余为 0。部分实色像素 alpha 为 252–254，保留生成结果。

工具缩略图中水手的轮廓外仍可能显示蓝白色 RGB，但所检外圈位置的真实 alpha 为 0，不作为可见背景。实际插件显示由浏览器按透明通道合成，仍应随装配检查深浅背景效果。

本记录只涵盖素材检查。插件构建、手部位置和桌面交互由对应修改的验证记录说明。素材来源与许可沿用 [角色上色记录](colored-characters.md) 和 [DSH 角色接入记录](dsh-character-assets.md)，本次生成不改变原角色或参考图片的许可。

## 实际提示词

四次调用使用以下共同提示词，图 1 为各自原彩稿，图 2 为长发蓝女仆彩稿，均设置透明背景。

```text
Use case: style-transfer.
Asset type: polished transparent PNG BODY sprite for a small desktop typing companion.
Image 1 is the edit target and the sole authority for character identity, short hair silhouette, pose, framing, costume, facial expression, and body proportions. Image 2 is ONLY the rendering-quality reference: fine clean anime contour lines, controlled soft cel shading, blue hair highlights and delicate cloth rendering. Do not borrow Image 2's long hair, closed eyes, clothing design, facial features or anatomy.
Refine Image 1 to the same finished illustration quality as Image 2. Replace the excessively thick dark brown sticker-like contour with thin, crisp, nuanced dark indigo linework. Refine the existing hair locks with subtle clean strand divisions and smooth blue gradients; use restrained luminous highlights, warm cream skin and soft pink cheeks, and subtle fabric folds and shading. Keep Image 1's small round solid dot eyes and tiny omega-shaped closed mouth unchanged in identity and expression. The design must still read at 100–220 px wide, so avoid noisy micro-texture.
Composition invariants: same low wide prone resting body, large head on the left and the raised two-lobed fish tail on the right, same outfit and accessories, all body parts within the canvas, transparent margin around all outer edges. Keep the original horizontal aspect ratio and approximately the original framing.
This is only the BODY layer: no hands, arms, paws, exposed feet, rounded sleeve blobs in front of the face, detached pieces, keyboard, desk, ground, cast shadow, glow, decorative scenery, text, watermark, frame, or second character. Do not turn the pose upright or sitting. Do not add human limbs.
Output one high-quality full-color isolated body PNG with genuine transparent alpha outside the silhouette and in natural open gaps. Character interiors must be opaque, including white fabric. No checkerboard pixels or opaque backdrop.
```

每次另附对应服装约束：

### 基础女仆鱼

```text
Costume constraints: Preserve the crescent-shaped ahoge above the white ruffled maid headband, cyan side bow, white scalloped fin-shaped side hair, short blue bob hair and simple blue fish body. Keep this a simple short-haired maid-fish with no added dress, gold ornaments or long curls.
```

### 水手装

```text
Costume constraints: Preserve the white sailor cap with navy band and blue round top ornament, white outfit with navy double-stripe sailor collar, cyan tied neckerchief, short blue bob hair and blue fish tail. Use subtle cloth folds without changing collar geometry or adding sleeves/hands. Remove all visible outer halo and any background glow; alpha must be clean beyond the contour.
```

### 睡衣装

```text
Costume constraints: Preserve the lavender-blue striped nightcap with yellow crescent moon and two stars, ivory scalloped brim and ivory pom-pom, matching striped pajamas with two white buttons, short blue bob hair and blue fish tail. Refine the existing fabric folds and soft trim without adding accessories or hands.
```

### 魔法师装

```text
Costume constraints: Preserve the bent indigo wizard hat with two gold stars, indigo cape and warm gold throat bow and clasp, short blue bob hair and blue fish tail. Refine subtle folds in hat and cape and blue hair shading. Do not add a staff, wand, book, sparkles, hands or new costume ornaments.
```
