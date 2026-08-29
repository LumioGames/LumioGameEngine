# S0 · 基线、模块地图与读取纪律

> 结论先行
> 1. 本次分析的版本钉死于 **UE 5.8.2**（git `ff8421f2b`，分支 `5.8`，2026-08-25），全报告行号相对该版本。
> 2. GAS 代码分居三处：**GameplayAbilities 插件**（Engine/Plugins/Runtime/GameplayAbilities，模块 GameplayAbilities + GameplayAbilitiesEditor）、**GameplayTags**（Engine/Source/Runtime/GameplayTags，运行时模块）、**GameplayTasks**（Engine/Source/Runtime/GameplayTasks）。注意 GameplayTags/GameplayTasks 的 UHT 类型大多在 `Classes/` 而非 `Public/`（Public 下分别只有 3 个与 0 个 USTRUCT）。
> 3. 依赖方向：GameplayAbilities → {GameplayTags, GameplayTasks, NetCore, DataRegistry, Niagara(私有), MovieScene, PhysicsCore, DeveloperSettings}，且调用了 `SetupIrisSupport(Target)`（Iris 适配已在模块层接线）。

## 0.1 版本三件套（R4）

| 项 | 值 | 来源 |
|---|---|---|
| MajorVersion / MinorVersion / PatchVersion | 5 / 8 / 2 | Engine/Build/Build.version:2-4 |
| BranchName / CompatibleChangelist | "UE5" / 55116800 | Engine/Build/Build.version:8-6 |
| git commit | `ff8421f2b8cb4feb76fff57965a1effc53a6eb7b` | `git rev-parse HEAD` |
| git 分支 / 最后提交 | `5.8` / `ff8421f2b 2026-08-25 "Localization Automation using CL 57313377"` | `git log -1` |
| 插件 descriptor | FileVersion 3，**无 "IsExperimentalVersion" 字段，IsBetaVersion: false**；EnabledByDefault: false；模块 = GameplayAbilities(Runtime, PreDefault) + GameplayAbilitiesEditor(UncookedOnly, PreDefault)；插件依赖 EngineAssetDefinitions/GameplayTagsEditor/Niagara/DataRegistry | Engine/Plugins/Runtime/GameplayAbilities/GameplayAbilities.uplugin:1-47 |

（S13 勘误点：预研标「当前源码的成熟度字段待核」——5.8 源码的 uplugin **不含** Experimental/Beta 成熟度标记；对照 Iris 插件显式写了 `IsBetaVersion: true`，GameplayAbilities 没有。）

## 0.2 模块地图

```
Engine/Plugins/Runtime/GameplayAbilities/Source/
├─ GameplayAbilities/            (Runtime 模块)
│  ├─ Public/  (含子目录 Abilities/, Abilities/Tasks/, Abilities/Async/,
│  │            GameplayEffectComponents/, Serialization/(Iris), Sequencer/)
│  └─ Private/ (含 Abilities/, GameplayEffectComponents/, Serialization/,
│               Tests/, Sequencer/)
└─ GameplayAbilitiesEditor/      (UncookedOnly 模块, 7 个 UCLASS)

Engine/Source/Runtime/GameplayTags/   (UHT 类型主要在 Classes/, Iris 适配在 Public/)
Engine/Source/Runtime/GameplayTasks/  (UHT 类型主要在 Classes/)
```

**Build.cs 依赖（读到原文）**：
- GameplayAbilities（Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/GameplayAbilities.Build.cs:11-42）：Public = Core, CoreUObject, NetCore, Engine, GameplayTags, GameplayTasks, MovieScene, PhysicsCore, DeveloperSettings, DataRegistry；Private = Niagara（+编辑器下 EditorFramework/UnrealEd/Slate/SequenceRecorder）；`SetupGameplayDebuggerSupport` + `SetupIrisSupport`（:40-42）。
- GameplayTags（Engine/Source/Runtime/GameplayTags/GameplayTags.Build.cs）：Public = Core, CoreUObject, Engine, DeveloperSettings；Private = Projects, NetCore, Json, JsonUtilities（+编辑器 Slate）。
- GameplayTasks（Engine/Source/Runtime/GameplayTasks/GameplayTasks.Build.cs）：Public = Core, CoreUObject, Engine, NetCore（注释掉的 GameplayTags 条目仍在文件里）；Private = EditorFramework, UnrealEd（仅编辑器，UnrealEd 同时在 CircularlyReferencedDependentModules）。

## 0.3 类型清单

完整清单（276 行，含 UCLASS/USTRUCT/UENUM/UINTERFACE、文件、行号、一句话职责）见 `appendix/symbol-map.csv`。统计：
- GameplayAbilities/Public：UCLASS 113、USTRUCT 114、UENUM 41、UINTERFACE 4，共 **272** 个 UHT 类型（127 个文件）。
- GameplayTags/Public：USTRUCT 3（真正的 Tag 类型在 Classes/GameplayTagContainer.h 等）。
- GameplayTasks/Public：0（类型在 Classes/）。
- GameplayAbilitiesEditor：UCLASS 7。

注意：`FAggregator`、`FAggregatorModChannel`、`FActiveGameplayEffectHandle` 的计数器等**非 UHT 类型不在该表**——它们是纯 C++ 结构（GameplayEffectAggregator.h），恰恰是求值顺序所在。

## 0.4 读取范围声明

亲自通读（函数体级）的文件（相对引擎根，`GA/` = Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/）：
- GA/Private/GameplayPrediction.cpp（全文 722 行）+ GA/Public/GameplayPrediction.h（全文含 264 行设计注释）
- GA/Private/ActiveGameplayEffectHandle.cpp、GameplayAbilitySpecHandle.cpp（全文）；对应头文件全文
- GA/Public/GameplayAbilitySpec.h（全文）、GA/Public/GameplayEffectAggregator.h（全文）、GA/Private/GameplayEffectAggregator.cpp（全文 718 行）
- GA/Private/GameplayEffect.cpp 的核心 ~2600 行（应用/堆叠/移除/到期/复制回调/NetDelta/世界时间/CanApply/捕获）
- GA/Private/AbilitySystemComponent.cpp 的 ~900 行（GE 应用入口、抑制、复制注册、Cue RPC、目标数据）+ 头文件关键区
- GA/Private/AbilitySystemComponent_Abilities.cpp 的 ~1300 行（激活全链、失败/确认/拒绝、Tag 阻塞、RPC 批处理）
- GA/Private/Abilities/GameplayAbility.cpp 的 ~900 行（CanActivateAbility/Commit 家族/End/PreActivate/Cooldown/Cost）
- GA/Private/Abilities/Tasks/AbilityTask.cpp（全文 416 行）
- GA/Public/AttributeSet.h、GA/Public/GameplayEffect.h（FActiveGameplayEffect/容器区）、GA/Public/Serialization/GameplayEffectContextNetSerializer.h
- Engine/Source/Runtime/GameplayTags/Private/GameplayTagContainer.cpp（序列化区）、GameplayTagsManager.cpp（ConstructNetIndex 区）

部分读取（声明为部分）：GameplayCueManager.cpp（Invoke/Flush 区）、GameplayEffectTypes.h（结构声明区）、ASC.h（分片）。
未读（原因）：GameplayAbilitiesEditor 全部（编辑器域，超出范围）；Sequencer 集成；Iris NetSerializer 的 .cpp 实现体（只读了声明与 Build 接线，S13 结论不依赖其内部）；GameplayTasks 模块内部（UGameplayTask 基类交互已从 GAS 侧读全）。

## 0.5 检索纪律与命中概况

完整检索日志见 `appendix/search-log.md`。要点：
- 命中的关键符号：InternalTryActivateAbility、ApplyGameplayEffectSpec、FAggregatorModChannel::EvaluateWithBase、ConstructNetIndex、SerializeTagNetIndexPacked、KeyRingBufferSize、SetActiveGameplayEffectInhibit 等（全部有正文行号）。
- **未命中（本身就是结论）**：`CheckOngoingTagRequirements`、`InhibitActiveGameplayEffect`（5.8 已不存在，被组件化取代）；`GetNetworkGameplayTagNodeIndexHash` 的任何调用者（Tag 表哈希只算不用）；Mass 在 GAS 内的任何真实引用（唯一命中是注释里的英文单词 "massive"）。
