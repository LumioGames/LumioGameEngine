# UE GAS 源码级分析（2026-08-29）

## 这是什么

对 Unreal Engine **5.8.2** Gameplay Ability System 的**源码级**分析报告：把第一波预研（docs/research/2026-08-29-ue-gas/，外部交回物）停留在 Reported 的论断逐条变成 Verified-Src（每条带 路径:行号·符号 三件套坐标），并挖掘文档永远不会写的实现细节（求值顺序、失败出口、位布局、编译期分支、Epic 的注释告解）。

- **源码基线**：C:\Work\UE-Engine · git `ff8421f2b`（分支 5.8，2026-08-25）· Build.version 5.8.2
- **读取纪律**：函数体级亲读 ~8,000 行；机械盘点（类型/CVar/TODO）由子代理执行并抽查核验；全程检索日志在 appendix/search-log.md
- **姊妹篇边界**：通用网络复制机制（UNetDriver/FRepLayout/FastArray 内核/Iris 核心）归 DS 篇；本报告只写 GAS 对这套机制的用法与前提

## 与第一波的关系

第一波目录在本机不存在（全盘搜索零命中，见 search-log.md D 节）；本目录是 `docs/research/` 下的第一份内容。对第一波结论的修正以**本目录的勘误**形式给出（report/S15-corrections.md + appendix/corrections-to-wave1.csv，33 条裁决：证实 12 / 修正 14 / 证伪 2 / 源码中不存在 2 / 其余升级为 Verified-Src）。第一波正文（若后续入库）不做任何改动。

## 章节索引

| 章 | 文件 | 内容 |
|---|---|---|
| — | [report/00-executive-summary.md](report/00-executive-summary.md) | **执行摘要（可独立阅读）**：十个最重要的源码级发现 |
| S0 | [report/S00-baseline.md](report/S00-baseline.md) | 版本三件套、模块地图、类型清单、读取范围 |
| S1 | [report/S01-handles.md](report/S01-handles.md) | 句柄与实例模型：四层身份、进程级计数器、ABA |
| S2 | [report/S02-activation.md](report/S02-activation.md) | 激活调用链：失败出口穷举、CanActivate 九步、Commit |
| S3 | [report/S03-effects-and-modifiers.md](report/S03-effects-and-modifiers.md) | ★重章 Effect 应用与 Modifier 求值顺序 |
| S4 | [report/S04-stacking-duration-inhibition.md](report/S04-stacking-duration-inhibition.md) | ★重章 堆叠/时长/抑制时序 |
| S5 | [report/S05-attributes.md](report/S05-attributes.md) | 属性与聚合器：重算触发源、钩子分工、ECS 迁移 |
| S6 | [report/S06-tags.md](report/S06-tags.md) | Tag 内部表示、位布局、零校验错误路径 |
| S7 | [report/S07-replication.md](report/S07-replication.md) | ★重章 复制全景：15 组属性、三模式、Mixed 陷阱 |
| S8 | [report/S08-prediction.md](report/S08-prediction.md) | ★重章 预测键与收敛：乐观 vs 回滚终裁 |
| S9 | [report/S09-cues.md](report/S09-cues.md) | GameplayCue 三条网络路径 |
| S10 | [report/S10-tasks.md](report/S10-tasks.md) | AbilityTask 与跨帧挂起状态、任意帧快照裁决 |
| S11 | [report/S11-determinism.md](report/S11-determinism.md) | 确定性与状态哈希可行性（反面教材清单） |
| S12 | [report/S12-debug.md](report/S12-debug.md) | 调试设施：76 CVar + 30 命令全钉死 |
| S13 | [report/S13-iris-mass.md](report/S13-iris-mass.md) | Iris 适配现状与 Mass 零桥接（一手裁决） |
| S14 | [report/S14-surprises.md](report/S14-surprises.md) | Epic 的注释告解、废弃史、编译期差异 |
| S15 | [report/S15-corrections.md](report/S15-corrections.md) | 对第一波的勘误（33 条全裁决） |
| S16 | [report/S16-conclusions.md](report/S16-conclusions.md) | 源码级结论与可迁移性再判定 |

## 附表（硬指标）

| 文件 | 内容 | 行数 |
|---|---|---|
| [appendix/evidence-index.csv](appendix/evidence-index.csv) | 源码证据索引：编号/章节/论断/路径/行号/符号/置信度/是否改变预研 | 124 |
| [appendix/corrections-to-wave1.csv](appendix/corrections-to-wave1.csv) | 对第一波的勘误（机器可读） | 33 |
| [appendix/replication-map.csv](appendix/replication-map.csv) | S7 复制全景表 | 25 |
| [appendix/symbol-map.csv](appendix/symbol-map.csv) | S0 类型清单（UCLASS/USTRUCT/UENUM/UINTERFACE） | 276 |
| [appendix/cvar-and-commands.csv](appendix/cvar-and-commands.csv) | S12 准确名称清单（CVar + 命令） | 106 |
| [appendix/state-machine-crosswalk.csv](appendix/state-machine-crosswalk.csv) | S16.3 UE↔目标引擎状态机对照 | 20 |
| [appendix/search-log.md](appendix/search-log.md) | 检索日志（含「未命中即结论」） | — |
| [sources.md](sources.md) | 来源总表（30 条，全部一手） | — |

## 执行摘要全文

见 [report/00-executive-summary.md](report/00-executive-summary.md)。十条发现中至少四条（Modifier 求值语义、抑制机制重构、Tag 表零校验、Iris 适配现状）**修正或证伪**了预研/社区的常见结论。

## Known gaps

1. wave-1 原文不在本机 → S15 以任务书转述论断为裁决对象。
2. Iris NetSerializer 实现体未逐行读（结论不依赖；量化细节留待 Iris 专题）。
3. Sequencer×GAS、GameplayTasks 模块内部未展开（分工边界外）。
4. 网络通用机制内部归 DS 姊妹篇，交界处只给坐标。
5. `AbilityTask_StartAbilityState`（5.x 新增的显式状态段任务）只确认存在与用途，机制细节未展开——它与目标引擎冻结状态机同路，值得下一轮单独读。

## 使用约定

- 引用格式：`路径:起行-止行 · 符号名`（路径相对引擎根 C:\Work\UE-Engine）。
- 置信度四级：Verified-Src / Verified-Doc / Reported / Estimated（图例见主报告）。
- 本目录遵守 UE EULA：无源码整段粘贴，复杂控制流以伪代码/流程图表达。
