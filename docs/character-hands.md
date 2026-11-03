# 角色配套手部

日期：2026-11-03。按用户要求，根据六个角色各自的服装、配色和描线风格制作独立手部。三套换装原先共用的白色椭圆彩手已替换，另外三款新增配套左右手。身体和彩色手部分别保存，原线稿保留。

## 素材

| 角色及身体参考 | 手部设计 | 独立文件 |
| --- | --- | --- |
| [基础女仆鱼](../assets/concepts/fish-maid-colored.png) | 奶油肤色圆手、白色扇贝荷叶边、青蓝束带和短蓝袖布，粗深棕描边 | [左手](../assets/concepts/fish-maid-left-hand-colored.png)、[右手](../assets/concepts/fish-maid-right-hand-colored.png) |
| [水手装](../assets/characters/sailor/body-colored.png) | 奶油肤色圆手、海军蓝双白条袖口和短白袖布 | [左手](../assets/characters/sailor/left-hand-colored.png)、[右手](../assets/characters/sailor/right-hand-colored.png) |
| [睡衣装](../assets/characters/pajamas/body-colored.png) | 奶油肤色圆手、淡紫蓝宽竖条纹袖口和象牙白镶边 | [左手](../assets/characters/pajamas/left-hand-colored.png)、[右手](../assets/characters/pajamas/right-hand-colored.png) |
| [魔法师装](../assets/characters/wizard/body-colored.png) | 奶油肤色圆手、靛蓝袖口、暖金边和金星 | [左手](../assets/characters/wizard/left-hand-colored.png)、[右手](../assets/characters/wizard/right-hand-colored.png) |
| [长发蓝女仆](../assets/concepts/blue-maid-prone-colored.png) | 浅桃肤色圆手、海军蓝袖口、白色蕾丝、细金线和青蓝蝴蝶结，描线更细 | [左手](../assets/concepts/blue-maid-prone-left-hand-colored.png)、[右手](../assets/concepts/blue-maid-prone-right-hand-colored.png) |
| [橙发礼服少女](../assets/concepts/orange-bride-prone-colored.png) | 象牙白手套、蕾丝袖口、暖金边、琥珀饰件和珍珠，匹配礼服的精细描线 | [左手](../assets/concepts/orange-bride-prone-left-hand-colored.png)、[右手](../assets/concepts/orange-bride-prone-right-hand-colored.png) |

每款均为正面、基本对称的圆润手形，短袖口位于后方，没有具体手指、拇指或长手臂。一个款式的最终图片原样复制为左右两份，共十二个 PNG 文件；六个款式各不相同。未将手拼回身体，也没有键盘、道具或场景。

## 生成与检查

六款均使用内置 imagegen，各一次生成成功。以各自角色身体为服饰与配色参考，以原椭圆手形为圆润轮廓参考。没有使用 CLI/API、程序绘图、镜像或像素修改。

十二张图片均为 1374 × 1145、8 位 RGBA PNG。已逐款检查外观，核验 PNG 数据完整性、透明通道、四角透明和复制一致性。各款左右文件具有相同 SHA-256，六款之间的 SHA-256 均不同。生成输出中接近不透明的 alpha 253 或 254 保留原样。

本轮只修改彩色手部素材及文档，身体和原线稿未改，未修改 Rust 源码或内置角色登记，因此未重跑 Rust 测试。原角色与参考素材的来源说明见[角色上色记录](colored-characters.md)，此记录不改变素材许可。

## 实际提示词

### 基础女仆鱼

```text
Use case: precise-object-edit.
Asset: one standalone colored cartoon hand sprite, separate from the character body.
Image 1 is the edit target and a simple shape reference: a single bold horizontal oval hand outline. Image 2 is ONLY the character costume, palette and line-style reference for the short-haired blue fish maid. Do not reproduce any head, face, hair, fish tail or body from Image 2.
Turn the generic oval into the short-haired fish maid's own hand. Keep the palm a smooth, softly rounded horizontal warm-cream oval, with no fingers, thumb, creases or palm markings. Attach only a compact maid cuff behind its upper edge. The cuff has clean white rounded scalloped ruffles, a narrow bright cyan-blue binding band, and at most a small short periwinkle-blue piece of sleeve fabric. Match Image 2's white maid frills, cyan bow colors and warm dark-brown thick rounded outlines. The identifiable design is WHITE SCALLOPED CUFF plus CYAN BAND; do not use a deep-navy cuff with gold trim.
Show the front-facing rounded palm low in the composition, with the small cuff at the upper rear, naturally joined as one hand-and-cuff sprite. The overall design is bilaterally symmetric so the same artwork can be used as either hand. Keep the cuff short and clear, not a long sleeve or forearm. Use simple clean cartoon fills, restrained cel shading and a few very soft highlights. Preserve the warm cream hand as a filled solid shape, not a hole or empty oval.
Exactly one isolated hand including its short cuff, centered and occupying about 75 percent of the frame with clear margins. Prefer a 1374 by 1145 landscape canvas. Keep all parts visible. Genuine transparent alpha everywhere outside the hand and cuff. No opaque backdrop, checkerboard, white background, floor line, external cast shadow or glow. No second hand, atlas, body, face, character, arm, props, keyboard, desk, text, label, watermark or border.
```

### 水手装

```text
Use case: precise-object-edit.
Image 1 is the EDIT TARGET: the existing simple white oval hand sprite. Image 2 is a COSTUME, COLOR AND LINE-STYLE REFERENCE ONLY: the blue-haired chibi sailor fish. Do not include any part of her body, head, face, hair, hat or tail in the output.
Replace the generic oval with one distinctive matching sailor hand sprite. Keep a compact, soft, horizontally rounded chibi hand/palm shape facing the viewer, with no visible fingers or thumb. Change the hand to the character's warm pale cream skin tone. Behind the top of this rounded hand, attach only a very short sailor sleeve: a clear deep-navy cuff with TWO ivory-white horizontal stripes, and a small amount of ivory-white sleeve fabric above it. The navy cuff and its two white stripes must be easily readable at small size. This should clearly match Image 2's navy-and-white sailor costume, instead of remaining a plain white oval.
The whole single hand and short cuff are front-facing and approximately left-right symmetrical, so this one design can serve as either hand. The palm is in the lower foreground; the short cuff is behind its upper edge. Keep the palm broad and rounded. No long arm, exposed wrist, separate fingers, thumb bump, finger creases, nails, palm lines, icons, face marks, jewelry, bow, objects or extra hands.
Match the reference's bold smooth dark-brown rounded contours, clean flat anime/cartoon fills, restrained cel shading and modest highlights. Skin is warm pale cream with a very soft warm shadow near the cuff; the cuff is deep navy with two white stripes; the short sleeve is ivory. Do not make the hand realistic, glossy plastic, or a 3D render.
Output exactly ONE standalone colored hand-and-short-cuff PNG, not an atlas or a pair. Use the existing 1374 by 1145 canvas proportions where possible. Center the complete sprite at about 75 percent of the canvas with comfortable transparent safety margins on every side. Fill the sprite interiors with color. Everything outside the outline must be truly transparent alpha, with no backdrop, checkerboard, haze, glow, ground, cast shadow, keyboard, desk, text, label, watermark or border. Do not crop the cuff or hand.
```

### 睡衣装

```text
Use case: precise-object-edit.
Image 1 is the EDIT TARGET: a separate simple oval cartoon hand sprite. Image 2 is ONLY the costume, color and line-style reference: the blue-haired fish girl in pale lavender-blue striped pajamas. Create one matching custom pajama hand from Image 1. Do not include or redraw the character from Image 2.

The hand is a small horizontal, smooth, rounded warm-cream palm shape facing the viewer, without fingers, a separate thumb, nails or palm creases. It keeps the cute rounded hand silhouette rather than becoming a realistic fist. Immediately behind the top of the palm, attach a VERY SHORT soft pajama sleeve cuff. The cuff has alternating broad pale-lavender-blue and slightly deeper lavender-blue vertical stripes, with a narrow soft ivory trim around the wrist opening, matching the pajamas in Image 2. The little cuff must be clearly visible as clothing and make this asset distinct from the plain white generic oval. The sleeve is behind and above the palm, not a second detached oval. Keep the entire silhouette approximately left-right symmetric so this one asset can be used for either hand.

Use the same bold smooth dark-brown outlines and clean simple cartoon fills as the reference body. Use restrained cel shading and very small soft highlights. The palm has warm cream skin matching the character's face; the sleeve is pale lavender-blue striped fabric with ivory trim. No pattern on the palm, no icons, no pillow, no sleep cap, no stars, no face, no body and no long arm.

Show exactly ONE isolated hand with its short sleeve cuff, centered at about 75 percent of the original 1374 by 1145 canvas with clear transparent safety margins. Keep the whole contour inside the frame. The enclosed palm and fabric regions must be filled with color. Everything outside the contour must be genuinely transparent alpha. No white backdrop, checkerboard pattern, floor, cast shadow, glow, halo, text, watermark, label, border, keyboard, desk, extra object, duplicate hand or sprite atlas.
```

### 魔法师装

```text
Use case: precise-object-edit.
Asset type: one isolated transparent hand sprite for the wizard-costumed blue fish girl.
Image 1 is the EDIT TARGET and the starting rounded hand shape. Image 2 is ONLY the clothing, color and drawing-style reference. Do not draw the character from Image 2.
Replace the generic white oval in Image 1 with one character-specific chibi hand: a smooth plump horizontal rounded palm in warm pale cream skin color, connected to a very short deep-indigo wizard sleeve cuff behind its upper edge. The hand faces the viewer. Make the short cuff slightly flared, with one narrow warm golden-yellow border and one small golden star centered on its visible fabric, matching the existing wizard hat and cape in Image 2. The cuff is a short fabric sleeve tube, never a hat: no pointed tip, hat brim or long arm. The entire silhouette should be compact and approximately bilateral-symmetric so the same single image can be used on either side.
Keep the palm simple and unmarked, with no separate fingers, thumb, knuckles, palm lines, wrist or realistic anatomy. The clothing cuff must clearly distinguish this sprite from the original generic white oval, but it remains smaller than the plump palm. Use the same bold rounded dark-brown contour and clean simple cel shading as the body reference, with restrained blue-violet fabric shadows and gentle warm skin shading. Preserve the cute broad rounded hand language.
Show exactly ONE complete isolated hand with its short sleeve cuff, centered on a preferably 1374 by 1145 transparent canvas, occupying about 75 percent of the canvas with clear margin on every side. All material interiors are colored; everything outside the silhouette is genuinely transparent alpha. Do not flatten onto a white or checkerboard background.
No second hand, paired sprites, atlas, face, eyes, mouth, hair, fish body, tail, long forearm, full sleeve, wand, book, flame, extra ornaments, desk, keyboard, background, floor line, cast shadow, outer glow, halo, text, label, border or watermark.
```

### 长发蓝女仆

```text
Use case: stylized-concept.
Create one standalone colored HAND sprite that belongs to the long-haired blue fish maid in reference Image 1.
Image 1 is the only reference for character style, costume and palette: refined chibi anime linework, warm fair skin, dark navy maid dress, white ruffled lace and restrained gold trim. Image 2 is only a silhouette guide for a simple smoothly rounded horizontal oval hand. Do not reproduce the character's body or face.
Draw a single small, warm peach-cream rounded hand mound facing the viewer, with a smooth horizontal oval silhouette and no individual fingers, thumb, knuckles or palm creases. Behind its upper edge, attach only a very short dark navy-blue maid sleeve cuff. The cuff has a neat white frilled lace edge and narrow warm gold piping, echoing the dress in Image 1. Keep the sleeve short and compact, with a little fabric depth. A tiny blue bow centered on the cuff is allowed, but it must remain subtle and smaller than the hand. Keep the design bilaterally balanced so exactly the same sprite can serve the left or right animated hand.
Use clean refined chibi anime outlines matching Image 1, slightly finer than a generic thick-line cartoon. Fill the hand with warm pale peach-cream skin, the short sleeve with deep navy, and the ruffled cuff with white and a little cool pale lavender shadow. Use small restrained gold lines on the cuff, gentle cel shading and modest highlights. Preserve a readable, rounded and simple hand silhouette, with the small lace cuff providing the character-specific detail.
Show exactly one complete detached hand with its short cuff, centered and occupying about 75 percent of a 1374 by 1145 canvas with a clear safety margin. This is a usable individual PNG sprite, not an atlas, a pair of hands, a glove display, a long arm, or a full character. All material regions are filled with color; everything outside the silhouette and any true open gaps is genuinely transparent alpha.
No body, face, hair, fish tail, extra limbs, rice, bowl, spoon, food, utensils, keyboard, desk, floor, background, cast shadow, outer glow, white backdrop, checkerboard pattern, text, label, border or watermark.
```

### 橙发礼服少女

```text
Use case: stylized-concept.
Asset type: one isolated hand sprite for the supplied orange-haired bridal chibi character, with a genuinely transparent background.
Image 1 is the sole COSTUME, COLOR and DRAWING-STYLE reference. Use its ivory bridal fabric, delicate lace, warm gold trim, amber jewels and refined brown anime outlines. Do not reproduce the girl, her face, hair, body, skirt or veil. Image 2 is ONLY a SHAPE reference for a smooth rounded horizontal chibi hand mass; do not borrow any other costume.
Create exactly ONE small front-facing chibi hand wearing an ivory-white glove. The palm is softly rounded and slightly narrower than a plain broad oval, with a smooth mitten-like silhouette but no separate thumb, fingers, knuckles, nails, finger lines or exposed skin. It should remain an abstract rounded hand suitable for a tiny typing companion, not a realistic anatomical glove.
Behind its upper edge, attach a very short compact ivory lace dress cuff. Use delicate scalloped white lace, fine warm-gold edging, and one small amber diamond-shaped jewel in a gold setting at the cuff center, with at most two tiny pearl details. A narrow charcoal-dark band behind the lace may provide a restrained accent consistent with the reference corset. The dominant palette is ivory-white and warm gold, never blue. Keep the cuff much smaller than the hand, and do not turn it into a large flower, crown, necklace, sleeve or full arm.
Match Image 1's clean, refined warm-brown anime linework and gentle cel shading. Keep the glove surface mostly plain ivory, with subtle warm-gray folds only at the cuff connection. Use a nearly symmetrical front view so the same sprite can serve as either hand. This is one hand plus a short cuff only, not a pair and not an atlas.
Center the full sprite with clear safe margins, occupying about 75 percent of a 1374 by 1145 canvas. Fill the glove and fabric interiors with color. Everything outside the isolated hand and the natural lace gaps must remain true transparent alpha. No body, face, head, hair, long arm, extra hand, prop, keyboard, desk, floor line, ground shadow, glow, halo, scene, opaque backdrop, checkerboard pattern, text, label, watermark or border.
```
