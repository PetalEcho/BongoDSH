# 角色上色记录

日期：2026-11-03，Asia/Singapore。首次上色包括六张角色身体和三套换装共用的椭圆手形。随后按用户要求，为六个角色各自制作配套手部，替换三套换装的通用彩手，并补齐另外三款的左右手。彩色版本使用 `-colored.png` 后缀，原线稿保留。

首次上色只使用内置 imagegen。每次以上一轮线稿为造型依据，配色参考只决定颜色，不恢复参考图中的坐姿、食物、双手或场景。身体与手分别保存为独立 PNG，没有键盘。

## 文件与配色

| 角色 | 彩色身体 | 配色 |
| --- | --- | --- |
| 基础女仆鱼 | [fish-maid-colored.png](../assets/concepts/fish-maid-colored.png) | 蓝发蓝尾、白色女仆头饰、青蓝蝴蝶结、奶油肤色 |
| 水手装 | [body-colored.png](../assets/characters/sailor/body-colored.png) | 蓝发蓝尾、白帽白衣、海军蓝条纹、青蓝领巾 |
| 睡衣装 | [body-colored.png](../assets/characters/pajamas/body-colored.png) | 蓝发蓝尾、淡薰衣草蓝睡衣与帽子、象牙白绒球、暖黄月星 |
| 魔法师装 | [body-colored.png](../assets/characters/wizard/body-colored.png) | 蓝发蓝尾、靛蓝帽子与披风、暖金星星和系带细节 |
| 长发蓝女仆 | [blue-maid-prone-colored.png](../assets/concepts/blue-maid-prone-colored.png) | 按用户附件采用深蓝到亮蓝的长发、深蓝女仆服、白围裙与荷叶边、青蓝蝴蝶结 |
| 橙发礼服少女 | [orange-bride-prone-colored.png](../assets/concepts/orange-bride-prone-colored.png) | 按用户附件采用铜橙长发、象牙白礼服和头纱、深色束腰、暖金花饰与琥珀色眼睛 |

当前手部按各角色的服装区分袖口、肤色或手套颜色，以及描线细节。每款使用一张对称手形，原样复制为左右两个独立文件；六款造型各不相同。配色说明、验证结果和实际提示词见[角色配套手部](character-hands.md)。

| 装扮 | 左手 | 右手 |
| --- | --- | --- |
| 水手 | [left-hand-colored.png](../assets/characters/sailor/left-hand-colored.png) | [right-hand-colored.png](../assets/characters/sailor/right-hand-colored.png) |
| 睡衣 | [left-hand-colored.png](../assets/characters/pajamas/left-hand-colored.png) | [right-hand-colored.png](../assets/characters/pajamas/right-hand-colored.png) |
| 魔法师 | [left-hand-colored.png](../assets/characters/wizard/left-hand-colored.png) | [right-hand-colored.png](../assets/characters/wizard/right-hand-colored.png) |
| 基础女仆鱼 | [fish-maid-left-hand-colored.png](../assets/concepts/fish-maid-left-hand-colored.png) | [fish-maid-right-hand-colored.png](../assets/concepts/fish-maid-right-hand-colored.png) |
| 长发蓝女仆 | [blue-maid-prone-left-hand-colored.png](../assets/concepts/blue-maid-prone-left-hand-colored.png) | [blue-maid-prone-right-hand-colored.png](../assets/concepts/blue-maid-prone-right-hand-colored.png) |
| 橙发礼服少女 | [orange-bride-prone-left-hand-colored.png](../assets/concepts/orange-bride-prone-left-hand-colored.png) | [orange-bride-prone-right-hand-colored.png](../assets/concepts/orange-bride-prone-right-hand-colored.png) |

## 来源与使用范围

基础鱼娘及换装的蓝白配色参考用户指定的上游 `body.png`，来源记录见[大肥鱼线稿与提示词](image-generation.md)。手部在此前独立椭圆手形的基础上，按各自身体的配色与服装细节重新生成。

两张附件角色分别参考 `ba63bd6cb0f40d4a8d1491ffd1e90a5e.png` 和 `6c064d0b389f8c352f501a89fe4bd825.png` 的颜色，原附件没有复制进仓库。线稿提示词见[趴窝角色记录](prone-lineart-drafts.md)。

这些文件是角色素材，本轮没有改动 Rust 源码或内置角色登记。生成记录不改变原始角色和参考图片的许可。

## 首轮上色检查

首轮六张身体和一个通用手形共进行九次内置生成或定向编辑。蓝发女仆上色后恢复了原线稿的小闭口笑；水手版进行了一次透明通道清理。所有输出只做原样复制，没有使用程序修改像素。后续六款手部的生成与检查另见[配套手部记录](character-hands.md)。

| 素材 | 像素尺寸 |
| --- | --- |
| 基础女仆鱼 | 1591 × 988 |
| 水手装 | 1536 × 1024 |
| 睡衣装 | 1642 × 958 |
| 魔法师装 | 1613 × 975 |
| 长发蓝女仆 | 1721 × 914 |
| 橙发礼服少女 | 1774 × 887 |
| 初版通用手形，彩色版已替换 | 1374 × 1145 |

首轮所有彩稿均为 8 位 RGBA PNG。逐一检查了完整造型、填色区域、身体无手以及透明背景，并只读核验尺寸、透明通道和文件复制一致性。首轮通用手的六份副本当时具有相同 SHA-256，现已替换为各款独立设计。原线稿均未修改。基础鱼娘彩稿比线稿窄 1 像素，其余输出保持原画布尺寸。

部分内置输出的实色区 alpha 为 253 或 254，属于接近不透明的生成结果。工具缩略图曾显示水手版透明像素中隐藏的棕色 RGB；在 macOS 预览中检查最终 PNG，轮廓外没有可见光晕。

本轮只增加图片和更新文档，未修改 Rust 源码，因此未重跑 Rust 测试。检查范围为素材和文档，不代表验证了程序运行时的角色装配。

## 首轮实际上色提示词

六张身体首次上色均以对应线稿作为图 1。基础鱼娘及换装以原始 `body.png` 作为图 2；两张附件角色以各自原图作为图 2。后续修正以各自的彩稿为编辑目标。手部只使用一张已确认的椭圆线稿。

### 基础女仆鱼

```text
Use case: precise-object-edit.
Asset type: one transparent PNG body sprite for a desktop typing companion.
Primary request: color the approved black line drawing in input image 1. This is a COLORING edit, not a redesign.
Input image 1 is the sole edit target and sole source of geometry, silhouette, pose, framing, facial expression, and linework. Its ink lines are black, while its empty interiors and exterior are transparent. Fill the character interiors with the requested colors, and preserve the exterior and natural open gaps as actual alpha transparency.
Input image 2 is ONLY a color reference. Take its soft royal/periwinkle blue hair and fish body, white maid frills, cyan blue bow, warm cream face, little rosy pink cheeks, and dark warm outline colors. Do not substitute image 2's geometry or framing for image 1.
Keep image 1's wide low prone resting pose exactly, the same head and body proportions, short rounded blue bob hair with thick bangs, crescent-shaped ahoge, frilled maid headband, side bow, fin-shaped side hair with white scalloped edging, tiny round dot eyes and little omega-shaped mouth, and the same two-lobed raised fish tail on the right. Preserve every existing visible contour and detail. The body is resting flat near the lower edge.
Apply simple clean anime/cartoon opaque fills, restrained soft cel shading and small highlights. Hair, fish body and tail are medium royal/periwinkle blue, with slightly darker blue shadows and gentle pale-blue highlights. Maid headband and scalloped fin edging are white with very pale cool-gray shadows. Bow is bright cyan-blue. Face is pale warm cream, cheeks soft pink. The little dot eyes and omega mouth stay dark. Preserve the bold smooth dark linework.
Critical constraints: this is the BODY layer only. No hands, arms, fists, paws, exposed feet, detached extra pieces, keyboard, desk, props, floor, cast ground shadow, background scenery, frame, labels, text, watermark, or extra character. Do not add sleeves or rounded blobs in front of the face. Do not turn this into a sitting character. Do not change dot eyes or omega mouth into anime eyes or an open mouth.
Output a single colored body with a genuinely transparent background, retaining image 1's aspect ratio, canvas composition, clear margin, and overall silhouette. Do not fill the whole canvas white. Do not make the character interiors transparent.
```

### 水手装及透明通道清理

```text
Use case: style-transfer. Colorize an approved character line-art BODY sprite without redesigning it.
Image 1 is the sole EDIT TARGET and sole source of geometry, silhouette, pose, costume and expression. Image 2 is a COLOR PALETTE reference only for blue hair and fish tail, warm cream skin and simple cute cartoon color rendering; do not copy its maid outfit, headband or any pose details.
Keep Image 1's exact low, wide belly-down resting posture, head/body proportions, separated bangs and short blue hair, side fin-like hair, raised double fish tail, existing sailor cap including its round top ornament, double-stripe sailor collar and knotted neckerchief. Preserve its round dot eyes, tiny omega-shaped mouth and cheek marks. Do not change the linework, add detailing, replace the hat, or change the expression. This is the BODY ONLY, with no visible hands, paws, exposed forearms or bulbous front sleeve shapes.
Apply clear, simple anime/cartoon colors inside the existing contours: periwinkle and royal-blue hair and tail with modest blue highlights, warm cream face with soft small pink cheeks; white or ivory sailor cap and clothing, deep navy cap band and double collar stripes, and a bright cyan-blue knotted neckerchief. Preserve dark bold contours. Use flat color areas with only restrained cel shading and a few soft highlights, matching the gentle palette reference. Do not paint photorealistic textures or add new accessories.
All character interior regions must become filled, opaque color, including light/white fabric. Everything outside the character silhouette and every natural gap between the outlines must remain genuinely transparent alpha. Preserve the existing full wide canvas and safe margins, with the full cap, hair and tail visible. No background, floor or cast shadow, checkerboard pattern, food, keyboard, desk, text, labels, watermark, borders, additional sprites or hands. Output one transparent PNG body illustration.

--- Attempt 2: alpha-channel cleanup ---
Use case: background-extraction. Precise alpha-channel cleanup only.
The supplied colored sailor fish-girl PNG is the sole EDIT TARGET. Keep its exact pixels visually unchanged inside the character: preserve the blue hair and fish tail, warm cream face and pink cheeks, white sailor cap and uniform, navy stripes, cyan neckerchief, dark outlines, dot eyes, omega-shaped mouth, existing wide prone pose, full silhouette and canvas. Do not redraw the character, add detail or change colors, shadows, highlights, expression or proportions.
Make the interior of the character completely opaque, including all white fabric, hair, face and fish tail. In RGBA terms, solid interior regions should have alpha 255. Make all space outside the dark outer contour and all natural holes between parts fully transparent alpha 0. Remove any background glow, halo, cast shadow, colored haze or faint pixels outside the outline. Only the immediate antialiased outline edge may use partial alpha. Do not add any background or flatten onto black or white.
Keep one complete isolated BODY PNG with its existing margins. No hands, paws, arms, keyboard, added objects, text, border or watermark.
```

### 睡衣装

```text
Use case: precise-object-edit. Image 1 is the sole EDIT TARGET: the approved transparent black line art of the pajama-clad chibi fish girl. Image 2 is a PALETTE-ONLY reference for the blue-haired fish girl's colors and simple rounded cartoon coloring. Color Image 1, keeping its exact low, wide belly-down prone pose, proportions, face, dot eyes, omega-shaped mouth, short bob and side fin-like hair, tail, sleeping cap with its original pompom, moon and star outlines, pajama collar, buttons and stripes. Do not copy Image 2's maid headband, maid costume, bow or geometry. Do not redraw, redesign or add any lines or objects.
Palette: royal-blue and periwinkle-blue hair and blue fish tail, warm cream skin, small soft pink cheek blush. The existing sleeping cap and pajamas are pale lavender-blue, with gentle alternating light and darker lavender-blue stripes where the original outlines specify stripes. The trim and pompom are ivory white. Fill the existing moon and stars with warm pale yellow. Keep the original thick dark outlines and black facial ink marks.
Use clean simple cartoon color fills with modest cel shading and a few subtle highlights, matching the companion blue fish character's softness. Do not introduce complex textures, photorealism, extra embroidery or accessories. Fill the character's enclosed material regions with solid color, while keeping the entire exterior and genuine gaps between hair, fins and costume transparent alpha. Preserve the full approved silhouette and transparent safety margins.
The body remains a separate BODY-only sprite with no visible hands, arms, forepaws, front sleeve blobs, detached hand sprites, feet or added limbs. Preserve the original expression and hat silhouette. No standing or sitting, no keyboard, desk, floor, external cast shadow, background, scenery, text, label, watermark, border or extra object. Output one colored PNG with a genuinely transparent background.
```

### 魔法师装

```text
Use case: precise-object-edit. Image 1 is the sole EDIT TARGET: the approved transparent black line art of a small chibi fish girl in a wizard costume, resting flat on her belly. Image 2 is ONLY a palette and simple cartoon-coloring reference, not a costume or geometry reference. Add color inside Image 1's existing contours while preserving its pose, low wide silhouette, proportions, face, round dot eyes, tiny omega-shaped mouth, bob haircut and side fin-like hair, right-side fish tail, floppy pointed wizard hat, existing stars, cape and tied collar. Do not copy the maid outfit or maid headband from Image 2.
Color the hair and fish tail royal blue and periwinkle blue with a few restrained cool-blue highlights. Give her a warm pale cream face and gentle pink cheeks. Color the existing pointed hat and cape deep indigo blue, with limited lighter blue folds. Fill only the existing outlined stars and tie details with warm golden yellow. Keep the existing dark rounded outlines crisp and visible. Use clean simple cartoon color fills, modest cel shading and a few soft highlights, readable at a small sprite size. Do not add new decorations or elaborate textures.
The character must stay in the same low horizontal prone position, with the original full silhouette and framing. This is exactly one standalone BODY layer: no visible hands, fingers, forepaws, feet, new limbs or paired sleeve blobs in front of the face; do not add a wand, book or other objects. Do not alter the expression or remove existing costume details.
Fill the character's enclosed material regions with solid color. Preserve a genuinely transparent alpha background and true gaps between the character's contours. Keep the entire character inside the canvas with its original clear margin, without cropping. No opaque background, white backdrop, checkerboard pattern, scenery, star field, floor line, external cast shadow, keyboard, desk, text, label, border, watermark or additional characters. Preserve the original line art as the drawing; change only its coloring.
```

### 长发蓝女仆

```text
Use case: precise-object-edit. Color an approved line drawing without changing its design.
Image 1 is the ONLY EDIT TARGET and the sole authority for all outlines, pose, proportions, expression and composition. It is the approved standalone body of a long-haired chibi fish maid resting low and belly-down, with no visible hands or paired round foreground sleeve masses.
Image 2 is a COLOR-PALETTE REFERENCE ONLY. Use its blue-and-white maid palette, but never copy its seated pose, eating action, rice, bowl, spoon, hands, legs, background or any additional shape into Image 1.
Preserve Image 1's exact prone silhouette, large head, face and smiling closed eyes, tiny mouth, long wavy hair, thick bangs, frilled maid headband, side bow, fin-like side hair, maid dress, apron and lace ruffles, rear fish tail, relative placements and wide framing. Her chin remains close to the ground. Keep the sleeves and arms concealed behind the head/body. Do not add hands, paws, mittens, round foreground puff sleeves, legs or detached sprites. Keep every recognizable contour and interior line from Image 1; this is coloring, not a character redesign or pose redraw.
Fill the enclosed character regions with complete opaque colors from Image 2: deep indigo and rich blue hair with brighter blue toward the wavy ends, a blue fish tail, a dark navy-blue maid dress, clean white or softly warm-white apron and frilled lace, a cyan-blue side bow, restrained small gold accents only on existing trim, warm ivory skin and soft pink cheek blush. Preserve crisp dark outlines. Use clean anime flat colors with light cel-shaded shadow shapes and a few soft restrained highlights. No heavy 3D rendering, excessive gloss, painted texture or realistic lighting.
All face, hair, headband, bows, costume and tail interiors must be properly filled and opaque, with no accidental large transparent holes inside the character. Only the background outside the silhouette and genuine open gaps between strands/parts remain truly transparent alpha. No white background, starry scene, checkerboard backdrop, floor line or ground shadow. Show the whole single body within the existing safety margins. No food, bowl, spoon, utensils, keyboard, desk, props, text, labels, frame or watermark.
```

### 长发蓝女仆嘴部修正

```text
Use case: precise-object-edit. Make ONE tiny facial correction to this colored resting blue-haired chibi fish maid.
Replace the open pink oval mouth with the small CLOSED curved smile of her approved line drawing: one short thin dark upward-curving smile line, with no open mouth cavity, no tongue, no inner pink shape and no teeth. Restore warm ivory face color to the area previously occupied by the large open mouth. Keep the smiling closed eyes, cheek blush and face proportions unchanged.
Preserve everything else exactly: the low belly-down resting pose, no visible hands or foreground sleeve masses, long indigo-to-bright-blue wavy hair, maid headband and side cyan bow, dark navy dress and gold trim, white apron and lace ruffles, blue fish tail, all contours, placement, size, framing, colors and gentle cel shading. Do not redesign or add any line, accessory or object.
The character's enclosed regions remain fully colored and opaque. The outside background and genuine gaps stay genuinely transparent alpha. No white background, floor shadow, food, rice, bowl, spoon, hands, legs, keyboard, desk, text, watermark or border. Only the mouth changes.
```

### 橙发礼服少女

```text
Use case: precise-object-edit. Image 1 is the sole EDIT TARGET: the approved standalone line art of a long-haired bridal woman as a low, horizontally prone chibi. Image 2 is a PALETTE-ONLY reference. Color the existing line art in Image 1 without redesigning, redrawing or changing its pose, proportions, face, half-open eye expression, hair shapes, flower and hanging ornaments, veil, high-neck long-sleeved dress, corset, outline, framing or composition. Do not copy Image 2's standing pose.
Use the original character's colors from Image 2, shown clearly without the reference screenshot's heavy dim lighting: long copper-orange and chestnut-orange hair; warm brown/copper flower ornament; charcoal-dark ribbon; ivory-white dress and a light ivory veil; charcoal-black corset; restrained warm gold details and amber jewels; soft warm fair skin; amber-brown eyes. Apply these colors only inside the existing corresponding contours. Preserve the existing dark linework. Use clean refined anime coloring, gentle limited cel shading and small soft highlights, maintaining readable colors at small sprite size. The veil may be subtly translucent.
The approved Image 1 must remain a fully clothed human chibi resting flat on her belly with her chin low, long hair and dress spread behind her, and hands completely concealed. No standing, sitting, kneeling, exposed hands, added limbs, fish tail, fins, animal ears or new accessories. Do not simplify or substitute the approved design.
Fill the character's material regions with color, while preserving genuine transparent alpha outside her silhouette and in true gaps between hair strands, lace and clothing. Keep the entire character visible with the existing transparent safety margin. No white or colored background, checkerboard pattern, ground, cast shadow outside the character, buildings, people, cursor, keyboard, desk, text, labels, border, watermark or extra objects. Exactly one separate colored BODY PNG.
```

### 通用独立手形，历史版本

```text
Use case: precise-object-edit.
The supplied PNG is the sole EDIT TARGET. It contains one clean bold black outline of a simple horizontal, smoothly rounded oval used as a separate cartoon hand sprite. Color ONLY the previously transparent enclosed interior of that existing oval.
Fill the oval with solid white or extremely pale ivory. Add at most a very faint pale cool-gray shade along the inner lower edge, entirely inside the outline. Retain the existing smooth thick black outer line. Keep the same exact oval shape, proportions, placement, framing and canvas, approximately 1.225 width-to-height for the shape on the existing 1374 by 1145 canvas. The result should be a simple flat cartoon sprite, not a rendered 3D object.
Do not redesign this abstract hand into a realistic hand or fist. No fingers, thumbs, wrists, cuffs, sleeves, finger creases, internal outlines, markings, accessories, extra highlights or extra objects. There must be exactly one oval and nothing else. No body, face, limbs, keyboard, desk, text, labels or watermark.
The filled oval interior must be opaque and colored, while everything outside the original black outer contour remains genuinely transparent alpha. No white backdrop, checkerboard pattern, shadow, glow, halo, floor line or ground. Preserve a small clear safety margin around the complete outline; do not crop or add padding. Export one separate transparent PNG hand sprite.
```
