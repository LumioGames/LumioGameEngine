# 外部技术调研报告（docs/research/）

五份并行的**纯外部技术调研**的交回物归档。每份一个目录，保持交付时的 zip 内结构原样（`README.md` / `report/` / `sources.md` / `appendix/`），不改写、不重排——**这是外部交回物，改动只能以新增批注文件的形式做，正文保持原样以便追溯。**

- 调研提示词在 [`../plans/`](../plans/)，与报告一一对应。
- 报告是**架构决策的输入**，不是决策本身。要把结论固化成公共语义，走 [`.spec/decisions/`](../../.spec/decisions/README.md) 的 ADR 流程。
- 报告里的论断带三级置信度（`Verified` / `Reported` / `Estimated`）。**引用报告做决策前先看它的置信度与「信息源可达性声明」**——同一份报告里源码级证据与二手转述的可信度差很远。

## 清单

| 主题 | 提示词 | 报告 | 状态 | 证据等级 |
|---|---|---|---|---|
| UE Gameplay Ability System | [`2026-08-29-ue-gas-research-prompt.md`](../plans/2026-08-29-ue-gas-research-prompt.md) | [`2026-08-29-ue-gas/`](2026-08-29-ue-gas/) | 已交回 2026-08-29 | **文档级**——指定的 UE 源码镜像不可达，全部源码级论断降为 `Reported`，无 permalink（见 [`source-access-log.md`](2026-08-29-ue-gas/appendix/source-access-log.md)） |
| UE Dedicated Server 架构 | [`2026-08-29-ue-dedicated-server-research-prompt.md`](../plans/2026-08-29-ue-dedicated-server-research-prompt.md) | [`2026-08-29-ue-dedicated-server/`](2026-08-29-ue-dedicated-server/) | 已交回 2026-08-29 | **文档级**——同样因源码镜像不可达而整体降级；另有**篇幅偏薄**问题，见下 |
| 前后端共用的对象组合式 ECS 框架 | [`2026-08-29-ecs-framework-research-prompt.md`](../plans/2026-08-29-ecs-framework-research-prompt.md) | [`2026-08-29-ecs-framework/`](2026-08-29-ecs-framework/) | 已交回 2026-08-29 | **混合**——GitHub 在线检索正常（Mirror 等仓库读到 ref + 行号）；经典 AOI 论文多数只到摘要，已降级 |
| 配表管线与运行时读表 | [`2026-08-29-config-table-pipeline-research-prompt.md`](../plans/2026-08-29-config-table-pipeline-research-prompt.md) | [`2026-08-29-config-table-pipeline/`](2026-08-29-config-table-pipeline/) | 已交回 2026-08-29 | **混合**——146 条来源带访问状态；格式选型矩阵与内存估算两张硬指标表都填满 |
| 体素地图存档（含 Minecraft 兼容） | [`2026-08-29-map-save-load-research-prompt.md`](../plans/2026-08-29-map-save-load-research-prompt.md) | [`2026-08-29-map-save-load/`](2026-08-29-map-save-load/) | 已交回 2026-08-29 | **混合**——逐条来源带访问状态；两张时序图与版本年表齐 |

## 已知的横向问题

### 1. UE 源码镜像不可达（已确认，影响两份）

`https://github.com/Go1c/UnrealEngine` 对网页版执行者打不开——GAS 与 DS 两份**独立尝试、独立失败**（GAS 三次尝试均 Cache miss；DS 报告写「无法可靠打开/搜索」）。两份都按提示词 R1 老实降级：不伪造 permalink、不切换到 `EpicGames/UnrealEngine` 冒充、源码级论断整体标 `Reported`。

**后果**：这两份报告的证据上限就是「Epic 官方文档 / API Reference + 社区资料」。API Reference 只给声明与公开语义，读不到函数体，因此**「内部先后顺序」「求值顺序」「实际实现细节」这类论断在两份报告里都拿不到一手证据**——恰恰是两份提示词标为最高优先级的那几章最需要的东西。

**已排除工具问题**：另外三份（ECS / 配表 / 地图存档）的 GitHub 在线检索**全部正常**——ECS 那份读到了 Mirror 等仓库的 ref + 行号，配表与地图存档都给出了带访问状态的逐条来源表。**所以不可达的是 `Go1c/UnrealEngine` 这个仓库本身，不是检索能力。**

**待办**：确认该仓库是公开还是私有。若私有，要么开放访问，要么接受这两份停留在文档级、并在引用其结论时始终按 `Reported` 对待。

### 2. DS 那份篇幅明显偏薄（五份横向对比）

| 报告 | 全文 | 章数 | 最厚三章 |
|---|---|---|---|
| 地图存档 | 约 157,500 字 | 19（A–S）| S 34,191 · B 14,401 · C 13,460 |
| GAS | 约 120,500 字 | 18（A–R）| H 19,794 · R 13,470 · I 12,824 |
| 配表管线 | 约 111,800 字 | 18（A–R）| C 18,031 · R 14,837 · D 8,779 |
| ECS 框架 | 约 96,200 字 | 16（A–P）| P 20,887 · G 11,703 · N 9,381 |
| **DS 服务器** | **约 42,100 字** | 13（A–M）| M 8,593 · E 5,171 · F 4,058 |

五份的章节数都与各自提示词要求一致，重点章也确实是最厚的几章。但 **DS 那份只有其余四份的三分之一到四分之一**——它最厚的 E 章（5,171 字）比其它四份的**平均单章**还薄，而提示词对 E / F 的验收要求是「深度达到可据此重新设计」。H（录像与回放，942 字）、K（安全与信任边界，1,301 字）明显没写开。

**处理方式（2026-08-29 决定）**：第一波先铺量，不退回、不做逐条验收核对；DS 这份的补写留到第二波。
