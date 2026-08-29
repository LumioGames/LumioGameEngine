# sources.md · 证据总表

| ID | 类型 | 标题或符号 | 定位 | 实际访问状态 | 支撑章节 |
|---|---|---|---|---|---|
| SRC-01 | 引擎源码 | Engine/Build/Build.version | C:\Work\UE-Engine\Engine\Build\Build.version | 已读（全文） | S0 |
| SRC-02 | 引擎源码 | GameplayAbilities.uplugin | Engine/Plugins/Runtime/GameplayAbilities/GameplayAbilities.uplugin | 已读（全文） | S0/S13 |
| SRC-03 | 引擎源码 | GameplayAbilities.Build.cs | Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/GameplayAbilities.Build.cs | 已读（全文） | S0/S13 |
| SRC-04 | 引擎源码 | GameplayTags.Build.cs / GameplayTasks.Build.cs | Engine/Source/Runtime/GameplayTags/ · GameplayTasks/ | 已读（依赖清单） | S0 |
| SRC-05 | 引擎源码 | GameplayPrediction.h/.cpp | GA/Public/GameplayPrediction.h + GA/Private/GameplayPrediction.cpp | 已读（两文件全文，含 264 行设计注释） | S8/S1/S14 |
| SRC-06 | 引擎源码 | AbilitySystemComponent.cpp（GE 应用/抑制/复制/Cue 区） | GA/Private/AbilitySystemComponent.cpp:330-2000 分片 | 已读（~900 行函数体） | S3/S4/S7/S9 |
| SRC-07 | 引擎源码 | AbilitySystemComponent.h | GA/Public/AbilitySystemComponent.h:240-530、1880-1990 | 已读（分片） | S1/S7/S8 |
| SRC-08 | 引擎源码 | AbilitySystemComponent_Abilities.cpp | GA/Private/AbilitySystemComponent_Abilities.cpp:1416-1510、1604-2545、4007-4334 | 已读（~1300 行函数体） | S2/S7/S8/S10 |
| SRC-09 | 引擎源码 | GameplayAbility.cpp | GA/Private/Abilities/GameplayAbility.cpp:86-130、349-691、741-1250 | 已读（~900 行函数体） | S2/S8 |
| SRC-10 | 引擎源码 | GameplayEffect.cpp | GA/Private/GameplayEffect.cpp:958-1024、2767-2960、3069-3736、3945-4561、4564-4743（头部）、4744-4960、5175-5600 | 已读（~2600 行函数体） | S3/S4/S7/S8 |
| SRC-11 | 引擎源码 | GameplayEffect.h（FActiveGameplayEffect/容器/UGameplayEffect 区） | GA/Public/GameplayEffect.h:58-1030、1354-1560、1651-2050 分片 | 已读（关键段） | S3/S4/S7 |
| SRC-12 | 引擎源码 | GameplayEffectAggregator.h/.cpp | GA/Public + Private 全两文件 | 已读（全文） | S3/S5/S11 |
| SRC-13 | 引擎源码 | GameplayAbilitySpec.h | GA/Public/GameplayAbilitySpec.h | 已读（全文） | S1/S7/S10 |
| SRC-14 | 引擎源码 | ActiveGameplayEffectHandle.h/.cpp | GA/Public + Private | 已读（全文） | S1 |
| SRC-15 | 引擎源码 | GameplayAbilitySpecHandle.h/.cpp | GA/Public + Private | 已读（全文） | S1 |
| SRC-16 | 引擎源码 | AttributeSet.h | GA/Public/AttributeSet.h:14-273、402-465 | 已读（关键段） | S5 |
| SRC-17 | 引擎源码 | GameplayTagContainer.cpp（序列化区） | GT/Private/GameplayTagContainer.cpp:40-170、1066-1180、1299-1630 | 已读（序列化全段） | S6 |
| SRC-18 | 引擎源码 | GameplayTagsManager.cpp（NetIndex 区） | GT/Private/GameplayTagsManager.cpp:340-360、700-880 | 已读（ConstructNetIndex 全函数） | S6 |
| SRC-19 | 引擎源码 | GameplayCueManager.cpp（Invoke/Flush 区） | GA/Private/GameplayCueManager.cpp:1326-1580 | 已读（关键段） | S9 |
| SRC-20 | 引擎源码 | AbilityTask.cpp | GA/Private/Abilities/Tasks/AbilityTask.cpp | 已读（全文） | S10 |
| SRC-21 | 引擎源码 | GameplayEffectContextNetSerializer.h 等 Iris 适配 | GA/Public/Serialization/*.h（9 个） | 已读声明（GameplayEffectContextNetSerializer.h 全文；其余经 symbol-map 收录） | S13 |
| SRC-22 | 引擎源码 | Iris.uplugin | Engine/Plugins/Experimental/Iris/Iris.uplugin | 已读（全文） | S13 |
| SRC-23 | 引擎源码 | MassGameplay 插件全树 | Engine/Plugins/Runtime/MassGameplay/**（~280 文件） | grep 级访问（0 命中 GAS 符号；GameplayTag 13 命中） | S13 |
| SRC-24 | 引擎源码 | Tests：PredictionKeyTests.cpp / GameplayEffectTests.cpp / AbilitySystemComponentTests.cpp / GameplayTagCountContainerTests.cpp | GA/Private/Tests/ | 注册行与描述串已读（代理核验+抽读） | S12 |
| SRC-25 | 引擎源码 | CVar/命令注册点（76+30 处） | 见 cvar-and-commands.csv | 代理机械提取 + 本人亲读 12 处注册点 | S12 |
| SRC-26 | 引擎源码 | TODO/FIXME/UE_DEPRECATED/WITH_EDITOR 清单 | GA/ + GT/ 全模块 | 代理机械提取 + 本人亲读 ~1/3 所引坐标 | S14 |
| SRC-27 | 引擎源码 | symbol-map 类型清单（272+3+0 项） | 三模块 Public | 代理机械提取 + 本人抽查行号一致 | S0 |
| SRC-28 | 官方文档（引擎内嵌） | GameplayPrediction.h:22-264 Epic 设计自述 | 同 SRC-05 | 已读（全文，属源码注释=一手） | S8/S14 |
| SRC-29 | 官方文档（引擎内嵌） | GameplayAbilities 插件 README.md | Engine/Plugins/Runtime/GameplayAbilities/README.md | 代理检索（Lyra 提及处） | S14 |
| SRC-30 | 委托方文档 | 目标引擎画像（任务书第 3 节）+ LumioGameEngineArchitecture schemas/gas-lifecycle.schema.json | 委托方仓库 | 已读（schema 全文 + ADR 文件名） | S16 |

说明：本报告的 Verified-Doc 级证据仅 SRC-28/29（源码头注释与插件内 README，按定义归官方文档级）；其余全部论断以 Verified-Src（函数体亲读）为主，Reported 级仅出现在对预研转述的引述处（S15 表「预研置信度」列）。外部社区资料零引用。
