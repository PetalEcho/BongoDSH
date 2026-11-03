# 大肥鱼线稿与提示词

生成日期：2026-11-02。使用内置 imagegen，线稿请求第一次成功。输出为 `assets/concepts/fish-maid-lineart.png`，1592 × 988，RGBA 透明背景。

2026-11-03，用户要求将现有线稿全部上色。六张身体和通用独立手部的彩色版本、配色与实际提示词见[角色上色记录](colored-characters.md)。以下保留线稿阶段记录。

身体参考：[dsh-bongo-fish 的 body.png](https://github.com/YUsaltyfish/dsh-bongo-fish/blob/main/assets/art/body.png)，读取时文件 SHA 为 `73ff868b5cdf4967f1ad5a82d95de35ef8644595`，原图 1222 × 758。

三套换装的手部参考：[hand-bounce.png](https://github.com/YUsaltyfish/dsh-bongo-fish/blob/main/assets/art/hand-bounce.png)，读取时文件 SHA 为 `42c317e58e71d1723260e21e463624887582bf46`，原图 210 × 179。按用户补充要求取其简单、饱满的横向椭圆轮廓，不画手指、拇指或袖口。线稿使用黑色外轮廓，内部透明。身体、左手和右手分别保存为独立 PNG。两张原始参考图均未复制到仓库。

## 逆向提示词

一只水平趴卧的蓝发 Q 版女仆鱼，大头位于画面左中部，身体宽扁贴地。圆润波波头搭配分束厚刘海，头顶有向右弯曲的月牙状呆毛，戴白色荷叶边女仆头饰，右侧别小蝴蝶结。两侧发束呈鱼鳍形，带白色扇形饰边。身体向右延伸，连接上翘的双叶鱼尾。奶油色小脸、圆点眼睛、ω 形小嘴、粉色腮红与短斜纹，粗圆深棕描边，柔和蓝色渐变和少量高光。构图宽扁、完整，透明背景，无手脚、无键盘、无文字。

线稿补充：保留上述轮廓、比例、姿势、表情和配饰。仅使用平滑黑色线条，外轮廓略粗，内部发束和褶皱线略细。去掉所有彩色色块、渐变、高光和阴影；眼睛和嘴巴可实心黑色，其余区域不填色。保留透明背景和安全留白。

## 实际生成提示词

```text
Use case: style-transfer. Edit target: the supplied reference PNG of a blue-haired chibi fish maid lying flat. Change ONLY its rendering into clean black-and-white line art. Preserve the exact character, wide low prone silhouette, large head at left-center, proportions, expression, pose, placement and costume. Keep the rounded bob haircut and separated thick bangs; the curved crescent-like ahoge/fin above the head; the scalloped frilly maid headband; the small bow at the right temple; the fin-like side hair with scalloped trim; the low body extending to the right and the raised two-lobed fish tail. Keep the round dot eyes, small omega-shaped mouth and simple short diagonal cheek marks. Draw smooth, confident, rounded black outlines: slightly heavier outer contours and slightly finer internal hair and fabric lines. Remove every colored fill, gradient, highlight, cast shadow, gray shading and texture; do not outline the painted shine spots as extra shapes. Only the tiny eyes and mouth may be solid black. Other enclosed areas remain unfilled and transparent. Preserve a genuinely transparent alpha background, with no white or checkerboard backdrop. The entire character must remain visible with a small clear safety margin on all sides. No new outfit, no hands, no limbs, no keyboard, no desk, no text, no watermark and no added objects.
```

## 三套装扮的独立 PNG

每套包含 `body.png`、`left-hand.png` 和 `right-hand.png` 三张独立透明图片。身体图中没有手，手部文件中只有一个椭圆轮廓。左右手采用同一张对称手形的原样副本，三套共九个 PNG 文件。

| 装扮 | 身体 | 左手 | 右手 | 身体尺寸 |
| --- | --- | --- | --- | --- |
| 水手 | [body.png](../assets/characters/sailor/body.png) | [left-hand.png](../assets/characters/sailor/left-hand.png) | [right-hand.png](../assets/characters/sailor/right-hand.png) | 1536 × 1024 |
| 睡衣 | [body.png](../assets/characters/pajamas/body.png) | [left-hand.png](../assets/characters/pajamas/left-hand.png) | [right-hand.png](../assets/characters/pajamas/right-hand.png) | 1642 × 958 |
| 魔法师 | [body.png](../assets/characters/wizard/body.png) | [left-hand.png](../assets/characters/wizard/left-hand.png) | [right-hand.png](../assets/characters/wizard/right-hand.png) | 1613 × 975 |

手部图片为 1374 × 1145，轮廓实际宽高比约 1.225，内部与外部均透明，没有袖口或指线。所有文件为 8 位 RGBA PNG。已在 macOS 预览中逐一检查三套身体与通用手形，并只读检查透明通道、尺寸和文件一致性。

全部使用内置 imagegen 生成或编辑；未用 CLI/API、程序绘图或像素修改。中间图集已移除，原始女仆鱼线稿保留。完整提示词见[三套线稿的实际提示词](fish-outfit-prompts.md)。本次仅增加素材与文档，未修改 Rust 源码，未重跑 Rust 测试。

## 来源与授权记录

这些是基于用户指定参考图生成的线稿研究稿，没有登记为程序内置角色。上游 [README 的许可说明](https://github.com/YUsaltyfish/dsh-bongo-fish#许可) 区分代码与素材许可，图片不会因为代码采用 MIT 而获得同等授权。本记录不对原始角色或派生美术授予额外许可。

先前生成的圆球鱼水手、睡衣、魔法师图集已按用户要求删除，没有提交。用户随后要求以当前鱼娘线稿重新制作这三套装扮，保留趴卧造型，并明确身体与双手须输出为独立图片文件。早先 `50.webp` 附件的生成已取消；后来单独指定的两张附件按[趴窝角色记录](prone-lineart-drafts.md)制作。
