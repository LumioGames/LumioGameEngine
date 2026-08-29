# UE GAS 源码级分析 · 主报告（S0–S16）

**日期**：2026-08-29 · **基线**：UE 5.8.2（git `ff8421f2b`，分支 `5.8`） · **性质**：第一波（docs/research/2026-08-29-ue-gas/，外部交回物，本机未见原文）的源码验证与深化

## 版本三件套（R4）

| 项 | 值 |
|---|---|
| Build.version | Major 5 / Minor 8 / Patch 2 · BranchName "UE5" · CompatibleChangelist 55116800（Engine/Build/Build.version:1-10） |
| git | `ff8421f2b8cb4feb76fff57965a1effc53a6eb7b` · 分支 `5.8` · 最后提交 2026-08-25 "Localization Automation using CL 57313377" |
| 插件 descriptor | GameplayAbilities.uplugin：无成熟度字段（IsBetaVersion:false、无 IsExperimentalVersion）；模块 GameplayAbilities(Runtime/PreDefault) + GameplayAbilitiesEditor(UncookedOnly/PreDefault)；依赖 EngineAssetDefinitions/GameplayTagsEditor/Niagara/DataRegistry |

## 读取范围声明

- **函数体级亲读**（~8,000 行）：GameplayPrediction.h/.cpp 全文；GameplayEffectAggregator.h/.cpp 全文；GameplayEffect.cpp 核心 ~2,600 行；AbilitySystemComponent.cpp ~900 行；AbilitySystemComponent_Abilities.cpp ~1,300 行；GameplayAbility.cpp ~900 行；AbilityTask.cpp 全文；两个句柄文件全文；GameplayAbilitySpec.h 全文；AttributeSet.h/GameplayEffect.h 关键区；GameplayTagContainer.cpp 与 GameplayTagsManager.cpp 序列化区；GameplayCueManager.cpp 关键段。
- **部分读取**：GameplayEffectTypes.h（结构声明区）、ASC.h（分片）、Iris NetSerializer 声明。
- **未读（原因）**：GameplayAbilitiesEditor 全部（编辑器域）；Sequencer 集成；Iris 序列化器 .cpp 实现体（S13 结论不依赖）；GameplayTasks 模块内部（GAS 侧交互已读全）。
- **机械盘点（子代理执行 + 本人抽查核验）**：类型清单 272+3+0 项（symbol-map.csv）；CVar 63 + 命令 22（cvar-and-commands.csv，本人亲读 12 个注册点）；TODO/废弃/编译分支清单（S14，本人亲读约 1/3 所引坐标）。

## 置信度图例（四级）

| 级 | 含义 | 门槛 |
|---|---|---|
| **Verified-Src** | 亲自读到实现（函数体） | 必须带 路径:起行-止行·符号 三件套 |
| **Verified-Doc** | Epic 源码注释/文档明文 | 注释坐标或 URL |
| **Reported** | 社区/预研一致但未核一手 | 来源与版本 |
| **Estimated** | 推断 | 写清依据 |

本报告重章（S3/S4/S7/S8）论断以 Verified-Src 为主；124 条证据见 `appendix/evidence-index.csv`。

## 章节索引

| 章 | 文件 | 一句话 |
|---|---|---|
| S0 | [S00-baseline.md](S00-baseline.md) | 版本钉死、模块地图、类型清单、读取纪律 |
| S1 | [S01-handles.md](S01-handles.md) | 句柄四层身份、计数器形态、ABA、实例化策略 |
| S2 | [S02-activation.md](S02-activation.md) | 激活全链、失败出口穷举、CanActivate 九步、Commit、RPC 批处理 |
| S3 | [S03-effects-and-modifiers.md](S03-effects-and-modifiers.md) | GE 应用 14 步、通道求值算式、四象限捕获、周期时间基准 |
| S4 | [S04-stacking-duration-inhibition.md](S04-stacking-duration-inhibition.md) | 堆叠 12 步时序、到期策略、抑制新形态、移除路径 |
| S5 | [S05-attributes.md](S05-attributes.md) | 属性数据布局、重算触发源穷举、钩子分工、ECS 迁移分析 |
| S6 | [S06-tags.md](S06-tags.md) | 位布局、NetIndex 表构建、零校验错误路径、Schema 适配 |
| S7 | [S07-replication.md](S07-replication.md) | 15 组复制属性全景、三模式分支、Mixed 陷阱、时间同步、状态vs事件终裁 |
| S8 | [S08-prediction.md](S08-prediction.md) | 预测键生命周期、乐观收敛终裁、不预测清单、Epic 自述限制 |
| S9 | [S09-cues.md](S09-cues.md) | Cue 三路径与可靠性、吸收守卫、late join、表现边界 |
| S10 | [S10-tasks.md](S10-tasks.md) | Task 生命周期、目标数据链路与服务器验证强度、任意帧快照裁决 |
| S11 | [S11-determinism.md](S11-determinism.md) | 顺序敏感容器、时间源、状态哈希六条拦路清单、可序列化性裁决 |
| S12 | [S12-debug.md](S12-debug.md) | 63 CVar + 22 命令钉死、showdebug/GD/VisLog、断言分布、自动化测试 |
| S13 | [S13-iris-mass.md](S13-iris-mass.md) | Iris 适配现状（存在但未打通）、Mass 零桥接、uplugin 成熟度 |
| S14 | [S14-surprises.md](S14-surprises.md) | 33 条 Epic 告解、废弃史、编译期差异、Fortnite 类名泄漏 |
| S15 | [S15-corrections.md](S15-corrections.md) | 33 条待证全裁决：证实 12 / 修正 14 / 证伪 2 / 源码不存在 2 等 |
| S16 | [S16-conclusions.md](S16-conclusions.md) | 十条源码级洞察、两判断题终裁、状态机对照、不可照搬清单、从零重写观点 |

## 附表

- `appendix/evidence-index.csv` — 124 条源码证据（编号/论断/坐标/置信度/是否改变预研）
- `appendix/corrections-to-wave1.csv` — 33 条勘误（机器可读）
- `appendix/replication-map.csv` — S7 复制全景 25 行
- `appendix/symbol-map.csv` — 198 行类型清单
- `appendix/cvar-and-commands.csv` — 85 行准确名称清单
- `appendix/state-machine-crosswalk.csv` — S16.3 状态机对照 20 项
- `appendix/search-log.md` — 检索日志（含未命中即结论的搜索）
- `sources.md` — 30 条来源总表

## Known gaps

1. **wave-1 原文不在本机**（全盘搜索零命中，见 search-log D 节）：S15 以任务书第 4 节转述论断为裁决对象；拿到原文后可补章节号映射与逐条对照。
2. Iris NetSerializer 的 .cpp 实现体未逐行读（S13 结论——存在性、默认关闭、TagCount 未打通——全部来自 Build 接线/头文件/CVar/注释，不依赖实现体）；量化行为（Quantize/Dequantize 细节）留待 Iris 专题。
3. Sequencer 与 GAS 的 Cue 集成（MovieSceneGameplayCueTrack 族）只收录进 symbol-map，未展开机制。
4. GameplayTasks 模块（UGameplayTasksComponent 的任务队列）按分工边界未展开——GAS 侧交互（ActiveTasks/TaskOwnerEnded/OnGameplayTaskActivated）已读全。
5. 网络通用机制（FastArray 内部 delta 算法、属性复制顺序保证、Iris 核心）按分工归 DS 姊妹篇；本报告在交界处只给坐标。
