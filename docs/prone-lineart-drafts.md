# 两张趴窝 Q 版线稿草稿

日期：2026-11-02。状态：等待用户确认线稿，尚未上色。用户明确要求确认后才按原图色系涂色。

## 图 1：蓝发女仆鱼

用户参考图为 `ba63bd6cb0f40d4a8d1491ffd1e90a5e.png`。保留蓬松长发、厚刘海、荷叶边女仆头饰、蝴蝶结、女仆裙和鱼尾特征。移除米饭、碗、勺子及吃饭动作，改成低矮趴窝姿态。

线稿文件：[blue-maid-prone-lineart.png](../assets/concepts/blue-maid-prone-lineart.png)。只包含角色身体，手部收在身体或衣袖下，没有额外的手部图集。

确认后上色参考：深蓝与亮蓝长发、蓝色鱼尾、深蓝女仆服、白色围裙及荷叶边、原图暖色肤色。原图星空背景不属于角色素材。

## 图 2：橙发礼服少女

用户参考图为 `6c064d0b389f8c352f501a89fe4bd825.png`。保留长发、分束刘海、侧面花饰与蝴蝶结、头纱、高领荷叶边、长袖礼服和束腰装饰。转换为大头小身体的趴窝 Q 版，长发和长裙向后铺开，双手藏在身体或袖内。去掉场景、其他人物与鼠标光标。

线稿文件：[orange-bride-prone-lineart.png](../assets/concepts/orange-bride-prone-lineart.png)。保留人类少女形象，不添加鱼尾或鱼鳍。

确认后上色参考：铜橙色长发、象牙白礼服和头纱、深色束腰与蝴蝶结、金色及暖棕色装饰，保留原图眼睛与肤色的色系。

## 生成与使用范围

两张均使用内置 imagegen，输出透明 PNG 黑色线稿。原始附件保留在用户本地，没有复制进仓库。这是待确认的角色线稿，没有替换已有三套装扮或登记为内置角色。

图 1 为 1721 × 914，使用三次内置生成或定向编辑；图 2 为 1774 × 887，一次生成成功。两张均为 8 位 RGBA，已在 macOS 预览中目视检查趴卧姿态、角色特征、食物移除和无额外手部轮廓，并只读核验透明通道及无白色填充。没有使用程序修改像素。

本轮只修改图片与文档，未修改 Rust 源码，因此不重跑 Rust 测试。素材的生成记录不改变原始角色或参考图片的许可。

## 实际生成提示词

### 图 1 初稿

```text
Use case: identity-preserve and pose change. Create one standalone BODY line-art PNG for approval before coloring.
Image 1 is the primary character identity and costume reference: the blue-haired chibi fish maid holding a rice bowl. Image 2 is ONLY a supporting reference for a low, wide prone composition and clean black outline treatment. Do not copy Image 2's short haircut or simplified costume.
Transform the character from Image 1 into a cozy belly-down resting pose. Her large chibi head is at the front, left of center, her chin very near the ground, and her small fully clothed torso and skirt lie low and extend back toward the right. The silhouette is broad and low, not seated, standing or kneeling. Preserve her long, fluffy, wavy hair spreading along the ground, thick bangs, frilled maid headband, side bow, fin-like side hair, layered maid dress with lace apron and frilled trim, and the recognizable fish tail curling gently upward at the rear. Keep her cheerful cute character with soft smiling closed-arc eyes and a tiny contented smile. Simplify small costume details only enough to keep the line art clear at chibi scale. Keep the original modest outfit.
Completely remove ALL rice, the rice bowl, the spoon, utensils, and the eating action. This is the BODY layer only: forearms and hands are naturally tucked beneath the resting body or hidden behind the sleeves, with NO visible attached hands and NO detached hand sprites. Legs and feet are tucked beneath the dress. Do not add separate accessories or other objects.
Draw clean smooth BLACK line art only, with bold rounded outer contours and slightly finer interior hair, fabric and lace lines. No colored areas, no white fills, no gray shading, gradients, hatching, texture or highlights. All enclosed interiors and the entire background must remain genuinely transparent, except the small black facial ink marks. No blue starry backdrop, floor line, cast shadow, keyboard, desk, text, labels, watermark or border.
Show the full single character body on a wide landscape canvas, centered with clear transparent safety margin on every side. Do not crop the hair, headband, costume or tail. This is an uncolored approval drawing, not a colored render and not a sprite atlas.
```

### 图 2

```text
Use case: identity-preserve.
Input image 1 is the sole character and costume reference: the adult orange-haired woman in an ornate high-neck bridal dress. Transform her into a cute approximately two-head-tall chibi, retaining her recognizable character design. Keep her very long flowing hair and separated thick bangs, the large flower ornament with dark ribbon and hanging decoration on the side of her head, the long light veil, high ruffled collar, long sleeves, lace-edged full dress, and the structured corset with a small central jewel. Simplify tiny lace and embroidery into a few clear elegant contour details suitable for a small chibi. Keep a gentle composed expression and simplified expressive anime eyes. Do not replace her with a generic short-haired character.
Pose and composition: one low, wide, horizontally resting prone chibi, lying flat on her stomach as if comfortably nesting. Her oversized head is at the front left-center with her chin close to the ground. Her tiny chest and belly lie horizontally behind the head toward the right. The complete long dress, veil and long hair pool and fan gently behind and around her body toward the right. Legs are folded low beneath the dress and fully concealed by it. Sleeves are tucked naturally beneath the chest; hands are fully hidden, with no visible hands or separate hand sprites. The silhouette must read as belly-down and resting low, never standing, seated upright or kneeling. Preserve full modest coverage and the original dress design. She is entirely human: no fish tail, fins, animal ears or other animal body parts.
Output exactly one standalone BODY illustration, not an atlas. Clean polished BLACK line art only, with a smooth slightly heavier outer outline and finer internal hair and clothing lines. Keep all closed areas unfilled and genuinely transparent, including hair, face, eyes' interiors, veil, corset and dress. Small pupils and facial marks may be black, but do not fill the corset, ribbons, hair or clothing with black or gray. Genuine transparent alpha background and transparent interiors. This is the line-art approval stage only: do not apply orange hair color, any other colors, grayscale shading, gradients, hatching, highlights or textures.
Keep the whole character and flowing hair, veil and dress visible inside a wide canvas with a modest clear transparent safety margin. Remove the original architecture, background people and cursor. No scenery, floor line, cast shadow, keyboard, desk, props, food, text, label, border, watermark or extra characters.
```

### 图 1 透明线稿修正

```text
Use case: precise-object-edit. The supplied image is the EDIT TARGET: the approved design of a long-haired chibi maid fish resting belly-down. Make exactly two corrections while preserving her pose, proportions, smiling closed eyes, mouth, long wavy hair, maid headband, side bow, lace dress, fish tail, full silhouette and canvas.
1. Remove ALL white and light-gray interior fills and paper-like texture. Retain only the existing clean black ink strokes. Every enclosed area between the strokes, including the face, hair, headband, bows, sleeves, dress and tail, must be truly transparent alpha, exactly like the empty background. Do not replace white with black, gray or another opaque fill. Black contour strokes and facial ink marks remain visible.
2. Remove the small exposed hands/skin shapes between the two front ruffled sleeve cuffs under her chin. The forearms and hands should be tucked completely underneath her resting body and hidden from view. Fold the front sleeves inward naturally so the drawing reads as one complete body without any visible hand, finger, palm or wrist. Keep the puffed sleeves as clothing. Do not create detached hands or any new objects.
The result is one standalone BODY PNG, with black linework only on transparent alpha, no white fill anywhere, no gray shading or color. Keep the full character with safety margins. No rice, bowl, spoon, food, utensils, keyboard, desk, text, watermark or background. No coloring yet.
```

### 图 1 前景手部轮廓移除

```text
Precise localized removal edit. The supplied transparent line-art PNG is the only EDIT TARGET.
Delete BOTH of the large rounded glove-like or puffed-sleeve shapes in the lower foreground directly below the girl's chin, including both of their small frilled cuffs. Remove their outlines completely. Do not merely remove fingers or expose less skin: the entire paired round masses and cuffs must disappear. No pair of paws, mittens, hands, folded forearms, or bulbous sleeves may remain in front of her face.
Let the bottom of her face be a simple natural chin contour very close to the ground, with her long wavy hair reaching the ground at both sides of the face. Her maid sleeves and arms are tucked behind the large head and underneath her resting body, completely hidden from the foreground. Retain her modest covered costume. Do not replace the removed shapes with a cushion, pillow, bow, ruffle pair or another object.
Keep the same cute smiling closed eyes, small mouth, face, long fluffy wavy hair, thick bangs, frilled maid headband, side bow, fin-like side hair, maid skirt and lace apron, rear fish tail, low horizontal belly-down pose and wide canvas. Show one standalone BODY layer only. Preserve the clean black outlines and truly transparent interiors/background. No white fill, gray shading, colors, texture or shadow. No rice, food, bowl, spoon, utensils, keyboard, desk, text, border, watermark or detached elements. No coloring. The entire character stays visible with safe margins.
```
