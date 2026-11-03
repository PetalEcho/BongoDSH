# 角色素材来源

本插件自 0.1.1 起使用 BongoDSH 项目内保存的六个彩色角色。首次素材来自用户指定的「制作并接入大肥鱼角色」会话，源提交为 `1eea77269830dd528991f83ca2dd02deb1635893`，2026-11-03 原样存入当前工作树。0.1.4 合入远端 `main` 的 `0f4e5ac` 素材更新，当时六个角色均使用各自服装配色的独立双手。0.2.0 使用内置 imagegen 按蓝女仆的描线和明暗质感精修前四款身体；其余身体、所有手部和原线稿不变。

当前源码移除基础女仆鱼，将长发蓝女仆更名为基础大肥鱼并设为默认，另三款更名如下。旧基础女仆鱼原图保留为历史素材，不再内嵌到页面或桌面程序。角色 ID 与素材路径沿用旧名以保留已有选择。

随后使用内置 imagegen，以基础大肥鱼为造型基准重绘水手鱼、睡衣鱼和魔法师鱼，将三款统一为长发、闭眼微笑和低趴造型，保留各自服装。三张新身体均为 1721×914 透明 PNG，原样保存；配套双手继续独立使用，并同步调整页面与桌面手部位置。本次重绘尚未发布。

| 角色 | 项目内的身体文件 | 独立双手 |
| --- | --- | --- |
| 基础大肥鱼 | `assets/concepts/blue-maid-prone-colored.png` | 同目录 `blue-maid-prone-left-hand-colored.png` / `blue-maid-prone-right-hand-colored.png` |
| 水手鱼 | `assets/characters/sailor/body-colored.png` | 同目录 `left-hand-colored.png` / `right-hand-colored.png` |
| 睡衣鱼 | `assets/characters/pajamas/body-colored.png` | 同目录 `left-hand-colored.png` / `right-hand-colored.png` |
| 魔法师鱼 | `assets/characters/wizard/body-colored.png` | 同目录 `left-hand-colored.png` / `right-hand-colored.png` |
| 克劳德 | `assets/concepts/orange-bride-prone-colored.png` | 同目录 `orange-bride-prone-left-hand-colored.png` / `orange-bride-prone-right-hand-colored.png` |

构建脚本将以上 PNG 原样编码为 data URI，内嵌到 `client.js`，兼容本机 DSH `0.2.1-rc.5`。以完整 SHA-256 去重，每款相同的左右手合并为一个资源，所以安装包共内嵌十个 PNG：五张身体和五款手部。解码后字节与源文件一致。原图、线稿和生成记录保留在仓库，不对像素进行裁剪、重绘或压缩。

素材由内置 imagegen 根据用户选定的图片生成或编辑。基础鱼娘及换装参考 [dsh-bongo-fish 的 body.png](https://github.com/YUsaltyfish/dsh-bongo-fish/blob/main/assets/art/body.png)，手形参考 [hand-bounce.png](https://github.com/YUsaltyfish/dsh-bongo-fish/blob/main/assets/art/hand-bounce.png)。长发蓝女仆和克劳德参考用户附件。所有角色无键盘图形。克劳德在早期生成记录中称为“橙发礼服少女”，0.1.3 起按用户要求改名。

原创代码采用 MIT，详见随包附带的 [LICENSE](LICENSE)。角色图片及其内嵌形式不包含在 MIT 授权中；原始角色与参考图片的权利仍然适用，不因代码许可或生成过程获得额外授权。当前未确认素材的完整再授权范围，使用或再分发前需确认相关权利。包以 `SEE LICENSE IN LICENSE` 标明代码与素材的许可边界，并保留 `private` 防止误发布到 npm。
