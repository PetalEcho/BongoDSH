# 大肥鱼线稿与提示词

生成日期：2026-11-02。使用内置 imagegen，线稿请求第一次成功。输出为 `assets/concepts/fish-maid-lineart.png`，1592 × 988，RGBA 透明背景。

参考：[dsh-bongo-fish 的 body.png](https://github.com/YUsaltyfish/dsh-bongo-fish/blob/main/assets/art/body.png)，读取时文件 SHA 为 `73ff868b5cdf4967f1ad5a82d95de35ef8644595`，原图 1222 × 758。原始参考文件未复制到仓库。

## 逆向提示词

一只水平趴卧的蓝发 Q 版女仆鱼，大头位于画面左中部，身体宽扁贴地。圆润波波头搭配分束厚刘海，头顶有向右弯曲的月牙状呆毛，戴白色荷叶边女仆头饰，右侧别小蝴蝶结。两侧发束呈鱼鳍形，带白色扇形饰边。身体向右延伸，连接上翘的双叶鱼尾。奶油色小脸、圆点眼睛、ω 形小嘴、粉色腮红与短斜纹，粗圆深棕描边，柔和蓝色渐变和少量高光。构图宽扁、完整，透明背景，无手脚、无键盘、无文字。

线稿补充：保留上述轮廓、比例、姿势、表情和配饰。仅使用平滑黑色线条，外轮廓略粗，内部发束和褶皱线略细。去掉所有彩色色块、渐变、高光和阴影；眼睛和嘴巴可实心黑色，其余区域不填色。保留透明背景和安全留白。

## 实际生成提示词

```text
Use case: style-transfer. Edit target: the supplied reference PNG of a blue-haired chibi fish maid lying flat. Change ONLY its rendering into clean black-and-white line art. Preserve the exact character, wide low prone silhouette, large head at left-center, proportions, expression, pose, placement and costume. Keep the rounded bob haircut and separated thick bangs; the curved crescent-like ahoge/fin above the head; the scalloped frilly maid headband; the small bow at the right temple; the fin-like side hair with scalloped trim; the low body extending to the right and the raised two-lobed fish tail. Keep the round dot eyes, small omega-shaped mouth and simple short diagonal cheek marks. Draw smooth, confident, rounded black outlines: slightly heavier outer contours and slightly finer internal hair and fabric lines. Remove every colored fill, gradient, highlight, cast shadow, gray shading and texture; do not outline the painted shine spots as extra shapes. Only the tiny eyes and mouth may be solid black. Other enclosed areas remain unfilled and transparent. Preserve a genuinely transparent alpha background, with no white or checkerboard backdrop. The entire character must remain visible with a small clear safety margin on all sides. No new outfit, no hands, no limbs, no keyboard, no desk, no text, no watermark and no added objects.
```

## 来源与授权记录

这是一张基于用户指定参考图生成的线稿研究稿，没有登记为程序内置角色。上游 [README 的许可说明](https://github.com/YUsaltyfish/dsh-bongo-fish#许可) 区分代码与素材许可，图片不会因为代码采用 MIT 而获得同等授权。本记录不对原始角色或派生美术授予额外许可。

用户取消了其他图片生成，并要求删除之前生成的水手、睡衣、魔法师图集；这三张图未保留在项目中，也未提交。附件少女 Q 版不再继续生成。
