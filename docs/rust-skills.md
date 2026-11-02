# 适合 BongoDSH 的 Rust skills

核查日期：2026-11-02。下表核对了 skills.sh 页面、GitHub 仓库和对应 `SKILL.md`。安装量来自目录展示，star 数来自当日 GitHub，均会变化，不作为代码正确性的保证。

Apollo `rust-best-practices` 已按用户要求安装到个人目录 `~/.codex/skills/rust-best-practices`，来源为 `apollographql/skills` 中的 `skills/rust-best-practices`，本次桌面原型已应用其规则。其他候选尚未安装。

优先选一个通用 Rust 规范，再按需要补所有权与性能专项。BongoDSH 当前没有直接使用 Tokio，异步技能留到确实引入异步 I/O 时再用。

| 优先级 | 技能与来源 | 对本项目的用途 | 安装量 / 仓库 stars |
| --- | --- | --- | --- |
| 首选 | [rust-best-practices · apollographql/skills](https://www.skills.sh/apollographql/skills/rust-best-practices) | 日常 Rust 编写与审查、错误处理、Clippy、公共接口和文档；来源为 Apollo 团队维护的规则 | 22.0K / 123 |
| 按需 | [m01-ownership · actionbook/rust-skills](https://www.skills.sh/actionbook/rust-skills/m01-ownership) | 处理所有权、借用和生命周期问题，尤其是 UI 状态与输入线程交界处 | 3.2K / 1,567 |
| 按需 | [m10-performance · actionbook/rust-skills](https://www.skills.sh/actionbook/rust-skills/m10-performance) | 测量常驻内存、事件处理和动画开销，再决定优化方案 | 3.7K / 1,567 |
| 引入异步后 | [rust-async-patterns · wshobson/agents](https://www.skills.sh/wshobson/agents/rust-async-patterns) | Tokio 任务、channel、取消、超时和退出清理，可用于未来 DSH 通信 | 21.5K / 42,092 |
| 通用备选 | [rust-skills · leonardomso/rust-skills](https://www.skills.sh/leonardomso/rust-skills/rust-skills) | 覆盖所有权、并发、unsafe、API 与性能的广泛规则，可替代首选通用技能 | 6.6K / 662 |

`rust-skills` 当前文档以 Rust 1.96 为背景，本项目本地验证版本为 1.93.1。采用其示例时须核对 MSRV；两套通用规范不必同时加载。技能示例中的生产依赖、lint 偏好和测试提交建议不能覆盖本项目约定。

## 安装命令

以下命令用于自行选择安装，不是本次已执行的操作。默认安装到项目，`-a codex` 限定目标 agent；需要用户级安装时再添加 `-g`。使用前可通过 `npx skills add --help` 检查当前 CLI 支持的选项。

```sh
npx skills add apollographql/skills --skill rust-best-practices -a codex
npx skills add actionbook/rust-skills --skill m01-ownership -a codex
npx skills add actionbook/rust-skills --skill m10-performance -a codex
npx skills add wshobson/agents --skill rust-async-patterns -a codex
npx skills add leonardomso/rust-skills --skill rust-skills -a codex
```

## 原始技能文件

- [Apollo Rust best practices](https://github.com/apollographql/skills/blob/main/skills/rust-best-practices/SKILL.md)
- [Ownership & lifetimes](https://github.com/actionbook/rust-skills/blob/main/skills/m01-ownership/SKILL.md)
- [Performance optimization](https://github.com/actionbook/rust-skills/blob/main/skills/m10-performance/SKILL.md)
- [Rust async patterns](https://github.com/wshobson/agents/blob/main/plugins/systems-programming/skills/rust-async-patterns/SKILL.md)
- [Rust skills](https://github.com/leonardomso/rust-skills/blob/master/SKILL.md)

当前环境已有 `code-review`、`tdd` 和安全审查相关技能，可按任务类型使用，无需为了 Rust 重复安装通用技能。没有为本项目找到已充分核验、需要立即安装的 egui/eframe 专用技能；GUI 实现优先查阅项目官方文档。
