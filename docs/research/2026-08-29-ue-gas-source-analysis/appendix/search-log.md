# 检索日志（search-log）

版本基线：UE 5.8.2 / git ff8421f2b（见 S0）。路径约定：`GA/` = Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/；`GT/` = Engine/Source/Runtime/GameplayTags/。工具：Claude Code Grep（ripgrep）+ PowerShell Select-String + Read（逐行读函数体）。

## A. 主线符号定位（全部命中并读函数体）

| 关键字 | 命中 | 读到 |
|---|---|---|
| `InternalTryActivateAbility` | GA/Private/AbilitySystemComponent_Abilities.cpp:1704 | 1704-1994 全读 |
| `TryActivateAbility` | 同上:1604 | 1604-1683 |
| `InternalServerTryActivateAbility` | 同上:2054 | 2054-2125 |
| `ClientActivateAbilityFailed/Succeed` | 同上:2279/2362/2367 | 全读 |
| `CanActivateAbility`（UGameplayAbility） | GA/Private/Abilities/GameplayAbility.cpp:457 | 457-575 + DoesAbilitySatisfyTagRequirements 349-443 + CheckCooldown/Cost 1064-1145 + Commit 家族 592-689 + End/PreActivate/CallActivate 741-1024 |
| `ApplyAbilityBlockAndCancelTags` | ASC_Abilities.cpp:1431 | 1431-1480 |
| `ServerAbilityRPCBatch` | ASC_Abilities.cpp:4184-4334 | 全读 |
| `ApplyGameplayEffectSpecToSelf` | GA/Private/AbilitySystemComponent.cpp:996 | 996-1179 全读 |
| `HasNetworkAuthorityToApplyGameplayEffect` | 同上:455 | 455-458 |
| `ApplyGameplayEffectSpec`（容器） | GA/Private/GameplayEffect.cpp:4171 | 4171-4561 全读 |
| `FindStackableActiveGameplayEffect` / `HandleActiveGameplayEffectStackOverflow` | 同上:3675/3701 | 全读 |
| `InternalRemoveActiveGameplayEffect` / `RemoveActiveGameplayEffectGrantedTagsAndModifiers`（头部） | 同上:4797/4949 | 4797-4960 |
| `CheckDuration` | 同上:5369 | 5369-5495 |
| `CanApplyAttributeModifiers` | 同上:5497 | 5497-5528 |
| `NetDeltaSerialize`（AGE 容器）/ `GetReplicationCondition` | 同上:5219/5183 | 全读 |
| `PreReplicatedRemove/Add/Change`（FActiveGameplayEffect） | 同上:2767/2804/2891 | 全读 |
| `OnAttributeAggregatorDirty` / `SetBaseAttributeValueFromReplication` / `OnStackCountChange` / `OnPredictiveGameplayEffectStackCaughtUp` / `OnMagnitudeDependencyChange` | 同上:3452/3743/3572/3593/3513 | 全读 |
| `ExecuteActiveEffectsFrom` / `PredictivelyExecuteEffectSpec` / `ExecutePeriodicGameplayEffect` / `InternalExecuteMod` / `ApplyModToAttribute` | 同上:3210/3069/3372/4090/4155 | 全读 |
| `IsServerWorldTimeAvailable` / `GetServerWorldTime` / `GetWorldTime` / `RestartActiveGameplayEffectDuration` | 同上:5334/5351/5363/5175 | 全读 |
| `UGameplayEffect::CanApply` / `OnAddedToActiveContainer` / `OnExecuted` / `OnApplied` | 同上:958/972/988/1001 | 全读 |
| `FActiveGameplayEffect` 字段区 | GA/Public/GameplayEffect.h:1414-1464 | 全读 |
| `SetActiveGameplayEffectInhibit` | GA/Private/AbilitySystemComponent.cpp:362 | 362-406 |
| `GetLifetimeReplicatedProps` / `ReplicateSubobjects` / `PreNetReceive/PostNetReceive` / `GetReplicatedCustomConditionStates` | 同上:1842/1927/1959/1879 | 全读 |
| `AddGameplayCue_Internal`（Original Hack 区） | 同上:1555-1615 | 全读 |
| `NetMulticast_InvokeGameplayCue*` 系 | 同上:1652-1747 | 全读 |
| `ServerSetReplicatedTargetData(+Validate/Cancelled)` | ASC_Abilities.cpp:4007-4063 | 全读 |
| `GameplayPrediction` 全文件 | GA/Private/GameplayPrediction.cpp（722 行）+ 头文件（625 行） | 全读 |
| `GameplayEffectAggregator` 全文件 | GA/Private/GameplayEffectAggregator.cpp（718 行）+ 头文件 | 全读 |
| `GenerateNewHandle`（两处） | GameplayAbilitySpecHandle.cpp:9 / ActiveGameplayEffectHandle.cpp:38 | 全读（各 10-55 行） |
| `FGameplayAbilitySpec` 结构 | GA/Public/GameplayAbilitySpec.h | 全读 |
| `AttributeSet.h` 关键区 | 14-273 + 宏区 402-465 | 读 |
| `AbilityTask.cpp` 全文件 | GA/Private/Abilities/Tasks/AbilityTask.cpp（416 行） | 全读 |
| `SerializeTagNetIndexPacked` / `FGameplayTagContainer::NetSerialize` / `FGameplayTag::NetSerialize_Packed` | GT/Private/GameplayTagContainer.cpp:69/1066/1572 | 全读 |
| `ConstructNetIndex` / `GetTagNameFromNetIndex` | GT/Private/GameplayTagsManager.cpp:767/839 | 全读 |
| `InvokeGameplayCueExecuted*` / `FlushPendingCues` | GA/Private/GameplayCueManager.cpp:1423-1580 | 读 |
| `FGameplayEffectSpec` 生命周期（Initialize/Capture*） | GameplayEffect.cpp:1686-1866/2570-2583 | 读关键段 |

## B. 未命中（本身就是结论）

| 关键字 | 搜索范围 | 结果 |
|---|---|---|
| `CheckOngoingTagRequirements` | GA/ 全模块 | **0 命中**（5.8 已不存在，组件化取代） |
| `InhibitActiveGameplayEffect` | GA/ 全模块 | **0 命中**（同上；现为 SetActiveGameplayEffectInhibit） |
| `GetNetworkGameplayTagNodeIndexHash` 的调用 | Engine/Source 全树 | **仅定义 1 处**（GameplayTagsManager.h:634），无消费者 |
| `Mass`（词边界） | GA/{Public,Private} | 1 命中 = GameplayPrediction.cpp:20 的英文单词 "massive"（误报）→ 真实引用 0 |
| `GameplayAbility|AbilitySystemComponent|GameplayEffect|GameplayAbilities` | Engine/Plugins/Runtime/MassGameplay 全部 .h/.cpp（~280 文件） | **0 命中**；`GameplayTag` 13 命中 |
| `Lyra|Fortnite|Paragon` | GA/ 全插件 | 类名命中 1 处：GameplayCueNotify_HitImpact.h:20 的 `UFortGameplayCueNotify_Burst`（tooltip 泄漏）；README.md 提及 Lyra 2 次 |
| `UE_VERSION_NEWER_THAN` | GA/ 全模块 | 0 命中（版本门控全靠 CVar） |
| `.spec.cpp` | GA/ | 0（无 spec 测试文件，用经典宏） |

## C. 机械化盘点（由 Explore 子代理执行、本人抽查核验）

- 类型清单（symbol-map.csv）：rg `UCLASS(|USTRUCT(|UENUM(|UINTERFACE(` 于三模块 Public；272+3+0 行；抽查 GameplayEffect.h:123-925 与 GameplayEffectTypes.h:111 的 UENUM 行号一致。
- CVar/命令清单（cvar-and-commands.csv）：rg `FAutoConsoleVariableRef|FAutoConsoleVariable|FAutoConsoleCommand|TAutoConsoleVariable|AutoRegisterConsoleCommand|IConsoleManager`；本人已亲读其中 12 个注册点（GameplayPrediction.cpp:23-41、ASC_Abilities.cpp:2274/1484/4161、GameplayEffect.cpp:76-130 的使用处等）。
- TODO/FIXME/废弃/编译分支清单（S14）：rg `TODO|FIXME|HACK|@todo|not ideal|workaround|deprecated|UE_DEPRECATED|WITH_EDITOR|UE_BUILD_SHIPPING`；S14 所引坐标中本人亲读约 1/3（GameplayPrediction.h 头注释、GameplayEffect.h:1440/2360、ActiveGameplayEffectHandle.h:30-70、AbilitySystemComponent.h:1921-1953、GameplayCueNotify_HitImpact.h:20 等）。

## D. 第一波报告（wave-1）定位尝试（均未命中）

| 搜索 | 范围 | 结果 |
|---|---|---|
| `2026-08-29-ue-gas*` | C:\Work 全盘（Depth 8） | 0 |
| `*gas*`（排除 UE-Engine） | C:\Work（Depth 4） | 仅 LumioGameEngineArchitecture 的 ADR/schema/fixtures（目标引擎自身文件） |
| `docs/research` 目录 | C:\Work 全盘 | 不存在（本次交付为该目录首个内容） |
| `ue-gas` 字符串 | LumioGameEngineArchitecture 全仓 | 0 |

结论：wave-1 原文不在本机。S15 以任务书第 4 节转述的预研论断为裁决对象。
