# S13 · 与 Iris / Mass 的实际集成现状（一手裁决）

> 结论先行
> 1. **Iris：GAS 模块层已接线、运行时默认关闭、且关键一环自认未完工**。Build.cs 调 `SetupIrisSupport(Target)`；Public/Serialization 下有 9 个 NetSerializer 适配（EffectContext、TargetData、TagCountContainer、MinimalCueProxy、RepAnimMontage 等）；但 `AbilitySystem.Fix.ReplicateTagCountContainerWithIris` **默认 false**，源码注释直说"we do not have working Tag Count replication through GameplayTagCountContainerNetSerializer"。
> 2. **Mass：双向零桥接**。GameplayAbilities 模块 grep `Mass` 的唯一命中是注释里的英文单词 "massive"（误报）；反向在 MassGameplay 插件全部 ~280 个 .h/.cpp 里 grep `GameplayAbility|AbilitySystemComponent|GameplayEffect` = **0 命中**；两者唯一共享的底座是 GameplayTags 模块（MassGameplay 有 13 处 GameplayTag 引用）。
> 3. GameplayAbilities.uplugin **不含任何成熟度标记**（无 IsExperimentalVersion；IsBetaVersion: false）——它按普通运行时插件发布；对照 Iris 插件显式 `IsBetaVersion: true`（Engine/Plugins/Experimental/Iris/Iris.uplugin）。

## 13.1 Iris 适配清单（读到原文）

| 适配 | 文件 | 备注 |
|---|---|---|
| 模块接线 | Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/GameplayAbilities.Build.cs:42 · `SetupIrisSupport(Target)` | 编译期启用 |
| FGameplayEffectContextNetSerializer | Public/Serialization/GameplayEffectContextNetSerializer.h:8-24 | 量化尺寸 528/496 字节（按 UE_WITH_REMOTE_OBJECT_HANDLE 分支，:17-21） |
| FGameplayEffectContextHandleNetSerializer / FGameplayAbilityTargetDataHandleNetSerializer / FGameplayAbilityTargetingLocationInfoNetSerializer / FGameplayAbilityRepAnimMontageNetSerializer / FMinimalGameplayCueReplicationProxyNetSerializer(+ReplicationFragment) / FMinimalReplicationTagCountMapNetSerializer / FGameplayTagCountContainerNetSerializer(+cpp) | Public/Serialization/ 同目录（symbol-map.csv 收录行号） | 除 TagCountContainer 外均头文件内声明 |
| AttributeSet 复制片段 | Public/AttributeSet.h:261 · `RegisterReplicationFragments` 覆写 | Iris fragment 注册钩子 |
| 预测键 NetSerializer | Public/GameplayPrediction.h:17 · `UE::Net::FPredictionKeyNetSerializer` 前置声明（friend，:402） | 与 NetSerializer 协同 |
| ASC 的 Iris 相关 CVar | GameplayEffectTypes.cpp:47（TagCountContainer 走 Iris，默认 false）；MinimalGameplayCueReplicationProxyReplicationFragment.cpp:19（`Net.GameplayCues.ShouldMarkStructDirtyAfterRemoval`） | 行为开关 |
| 明确的自认未完工 | GameplayEffect.cpp:4669-4670、4977-4978（同段注释两处）："CVarReplicateTagCountContainerWithIris indicates that we do not have working Tag Count replication through GameplayTagCountContainerNetSerializer. Instead, we replicate using the legacy path of separated containers (Minimal/Loose)" | **Tag 计数在 Iris 下仍走旧双容器路径** |

**Iris 插件成熟度（.uplugin 读原文）**：Engine/Plugins/Experimental/Iris/Iris.uplugin —— `IsBetaVersion: true`、`EnabledByDefault: false`、VersionName 0.1、目录在 Experimental 下。

**裁决**：预研若写「GAS 无 Iris 适配」（社区常见说法在 5.3/5.4 时代成立）——在 5.8 **已过时**：适配存在且在推进；但若写「GAS 已 Iris-ready」——也**不成立**：核心的 TagCountContainer 序列化器未打通、默认关闭、Iris 本体仍是 Beta。准确表述：**迁移进行中、按结构逐个接管、Tag 计数是当前缺口**。

## 13.2 Mass：搜索过程与零结论

| 方向 | 搜索 | 结果 |
|---|---|---|
| GAS → Mass | rg -i `\bMass` 于 Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/{Public,Private} | 1 命中 = GameplayPrediction.cpp:20 注释 "...for a **massive** local player count"（单词 massive，误报）→ **真实引用 0** |
| Mass → GAS | PowerShell Select-String `GameplayAbility|AbilitySystemComponent|GameplayEffect|GameplayAbilities` 于 Engine/Plugins/Runtime/MassGameplay 全部 .h/.cpp（~280 文件，含 MassActors/MassCommon/MassMovement/MassReplication/MassSpawner 等 10 模块） | **0 命中** |
| 共享底座 | 同上 grep `GameplayTag` | 13 命中（Mass 只用 GameplayTags 字符串标签体系） |

**结论**：UE 5.8 引擎源码中不存在任何 GAS↔Mass 桥接类、组件或模块依赖。社区若流传"Epic 在把 GAS 搬上 Mass"，就引擎源码而言**没有证据**；Lyra/项目层是否有桥接不在本次源码树范围内（未在本源码树中检索到项目层）。

## 13.3 对目标环境的迁移含义

Iris 适配清单就是一份**「Epic 自己认为 GAS 状态里哪些结构需要一等线协议表达」的官方答案**：EffectContext、TargetData、TagCount、CueProxy、AnimMontage——恰好是目标引擎 Schema 该定义的头几个消息。而 TagCountContainer 未打通的事实也提示：**带计数语义的集合状态比想象的难序列化**（delta、键序、量化），目标引擎设计 tag 计数的线协议时应预留 delta 语义而不是整包重发。
