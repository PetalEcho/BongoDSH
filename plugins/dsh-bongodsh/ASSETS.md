# 角色素材来源

本插件自 0.1.1 起使用 BongoDSH 项目内保存的六个彩色角色。首次素材来自用户指定的「制作并接入大肥鱼角色」会话，源提交为 `1eea77269830dd528991f83ca2dd02deb1635893`，2026-11-03 原样存入当前工作树。0.1.4 合入远端 `main` 的 `0f4e5ac` 素材更新，六个角色现在均使用各自服装配色的独立双手。0.2.0 使用内置 imagegen 按蓝女仆的描线和明暗质感精修前四款身体，记录见仓库 `docs/character-quality.md`；其余身体、所有手部和原线稿不变。

| 角色 | 项目内的身体文件 | 独立双手 |
| --- | --- | --- |
| 基础女仆鱼 | `assets/concepts/fish-maid-colored.png` | 同目录 `fish-maid-left-hand-colored.png` / `fish-maid-right-hand-colored.png` |
| 水手装 | `assets/characters/sailor/body-colored.png` | 同目录 `left-hand-colored.png` / `right-hand-colored.png` |
| 睡衣装 | `assets/characters/pajamas/body-colored.png` | 同目录 `left-hand-colored.png` / `right-hand-colored.png` |
| 魔法师装 | `assets/characters/wizard/body-colored.png` | 同目录 `left-hand-colored.png` / `right-hand-colored.png` |
| 长发蓝女仆 | `assets/concepts/blue-maid-prone-colored.png` | 同目录 `blue-maid-prone-left-hand-colored.png` / `blue-maid-prone-right-hand-colored.png` |
| 克劳德 | `assets/concepts/orange-bride-prone-colored.png` | 同目录 `orange-bride-prone-left-hand-colored.png` / `orange-bride-prone-right-hand-colored.png` |

构建脚本将以上 PNG 原样编码为 data URI，内嵌到 `client.js`，兼容本机 DSH `0.2.1-rc.5`。以完整 SHA-256 去重，每款相同的左右手合并为一个资源，所以安装包共内嵌十二个 PNG：六张身体和六款手部。解码后字节与源文件一致。原图、线稿和生成记录保留在仓库，不对像素进行裁剪、重绘或压缩。

素材由内置 imagegen 根据用户选定的图片生成或编辑。基础鱼娘及换装参考 [dsh-bongo-fish 的 body.png](https://github.com/YUsaltyfish/dsh-bongo-fish/blob/main/assets/art/body.png)，手形参考 [hand-bounce.png](https://github.com/YUsaltyfish/dsh-bongo-fish/blob/main/assets/art/hand-bounce.png)。长发蓝女仆和克劳德参考用户附件。所有角色无键盘图形。克劳德在早期生成记录中称为“橙发礼服少女”，0.1.3 起按用户要求改名。

原始角色与参考图片的许可仍然适用，不因代码许可或生成过程获得额外授权。包以 `UNLICENSED`、`private` 声明，不宣称这些素材采用 MIT。仓库中的 `docs/image-generation.md`、`docs/fish-outfit-prompts.md`、`docs/prone-lineart-drafts.md`、`docs/colored-characters.md` 和 `docs/character-hands.md` 保留具体来源、提示词和检查记录。
