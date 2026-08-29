# S12 · 调试设施与准确名称清单

> 结论先行
> 1. 全模块共 **76 个 CVar + 30 个控制台命令**（合计 106 行）被逐一钉死（名称/默认值/坐标见 `appendix/cvar-and-commands.csv`）；其中最重的家族是 `AbilitySystem.Fix.*`（11 个，全部是"行为修复开关"，默认值即"新行为开"）与 `AbilitySystem.PredictionKey.*`（4 个，语义随 UE5.4/5.5 演进）。
> 2. showdebug 入口是 `showdebug abilitysystem`（分类 Attributes / GameplayEffects / Ability，可 `AbilitySystem.Debug.SetCategory` 切换）；Gameplay Debugger 类别注册名是 **"Abilities"**（Shift+1..4 子开关）；自动化测试真实存在：PredictionKey 单测（**不依赖 World**）+ 世界级 GE/ASC 套件。
> 3. 断言分布即不变量文档：GameplayEffect.cpp 56 处、GameplayAbility.cpp 50 处、ASC_Abilities 35 处——最密集的是「FastArray 回调必须在容器锁内」「pending 链完整性」「activation 重入防护」。

## 12.1 名称清单的使用说明

- 完整 CSV：`appendix/cvar-and-commands.csv`（Name / Kind / DefaultValue / File / Line / Purpose，引擎根相对路径）。
- 预研所有「名称待核」项的裁决：`KeyRingBufferSize=32` ✅（GameplayPrediction.cpp:686）；`AbilitySystem.Fix.*` 系列 ✅；三个复制模式枚举名 ✅（AbilitySystemComponent.h:81-89）；`GameplayTags.PrintNetIndices` 等 ✅；**未在源码找到**的：无（本轮清单内全部命中；全部 106 项入 `appendix/cvar-and-commands.csv`）。注意拼写陷阱：`AbilitySystem.ClientActivateAbilityFailedPrintDebugThreshhold`（**Threshhold 双 h**，源码原文如此，AbilitySystemComponent_Abilities.cpp:2274）。

## 12.2 showdebug / Gameplay Debugger / VisLog

- showdebug：`AHUD::OnShowDebugInfo` 挂钩（GameplayAbilitiesModule.cpp:83）→ `UAbilitySystemComponent::OnShowDebugInfo`（AbilitySystemComponent.cpp:2524）→ DisplayDebug（2554）；类别数组 `{Attributes, GameplayEffects, Ability}`（2415-2430）。旁路常驻 HUD：`AbilitySystem.DebugBasicHUD` / `DebugAbilityTags` / `DebugAttribute` 等命令（AbilitySystemDebugHUD.cpp:615-645）。
- Gameplay Debugger：注册名 **"Abilities"**（GameplayAbilitiesModule.cpp:77），`FGameplayDebuggerCategory_Abilities`，Shift+1..4 = Tags/Abilities/Effects/Attributes（GameplayDebuggerCategory_Abilities.cpp:44-58）。
- VisLog：VLogAbilitySystem 类别贯穿激活/应用/拒绝热路径（ASC_Abilities 多处）；`FActiveGameplayEffectsContainer::DescribeSelfToVisLog`（GameplayEffect.cpp:5807-5861）与属性直方图（AttributeSet.cpp:23-30 的 `g.debug.vlog.AttributeGraph`）。

## 12.3 断言与校验保护的不变量（Top 文件 + 语义）

| 文件 | ensure/ensureMsgf/check/checkf 合计 | 保护的不变量（抽样） |
|---|---|---|
| GameplayEffect.cpp | 10+24+21+1 = 56 | FastArray 回调在锁内（2769/2806/2893）；pending 链不泄漏（4344）；spec/def modifier 数一致（3107/3238）；不删 PendingRemove 两次（4807） |
| GameplayAbility.cpp | 33+13+4 = 50 | ActorInfo 在场；CommitCheck 的三重有效性（658-667）；非实例化 CDO 调用拦截（1233 一带） |
| AbilitySystemComponent_Abilities.cpp | 10+9+16 = 35 | 服务器激活路径 ensure(Ability)（2079）；World 存在（1986）；montage 区段索引 |
| AttributeSet.cpp | 4+1+22 = 27 | 属性注册/查找的不变量 |
| GameplayEffectTypes.cpp | 4+5+8 = 17 | spec 序列化、tag 容器计数 |

（计数由 ripgrep 逐模式统计，含少量注释命中；模式见 search-log。）

## 12.4 自动化测试（脱离 World 的程度）

| 测试 | 全名 | 文件:行 | 是否需要 World |
|---|---|---|---|
| FGameplayPredictionKeyTest_UnitTest | System.AbilitySystem.PredictionKey.UnitTest | Private/Tests/PredictionKeyTests.cpp:15（RunTest:138） | **否**——纯键/委托语义，且覆盖 DepChainBehavior CVar 分支 |
| FGameplayPredictionKeyTest_ScopedPredictionsTest | System.AbilitySystem.PredictionKey.ScopedPredictions | 同文件:16（RunTest:432） | 是（测试 Pawn + PC） |
| FGameplayTagCountContainerTests | System.AbilitySystem.GameplayTagCountContainer | Private/Tests/GameplayTagCountContainerTests.cpp:6 | 否 |
| FGameplayEffectsTest | System.AbilitySystem.GameplayEffects | Private/Tests/GameplayEffectTests.cpp:790 | 是；子测 InstantDamage/ManaBuff/Aggregators/Periodic/StackLimit/SetByCaller 堆叠时长/GameplayCues（799-806） |
| FAbilitySystemComponentTest | System.AbilitySystem.AbilitySystemComponent | Private/Tests/AbilitySystemComponentTests.cpp:247 | 是；ActivateAbilityFlow/FailedAbilityFlow |
| GameplayTags 模块 | System.GameplayTags.* | Engine/Source/Runtime/GameplayTags/Private/Tests/ | 部分 |

**对目标引擎的启示**：键/委托/tag 计数这些纯数据机制被 Epic 单独拎出来做了免 World 单测——分层可测性是设计出来的，GAS 里恰好是"句柄层/求值层"这类无 UObject 依赖的部分可测。

## 12.5 预研「名称待核」清账结论

预研列出的待核名称在本轮全部核清（见 CSV 与 S0/S7/S8 各表）；无一条「源码中不存在」——但注意三个名称与常见社区拼写不同：`Threshhold`（双 h）、`AbilitySystem.Fix.RecalcuateTargetDataSourceOnApply`（**Recalcuate** 拼写错误进 CVar 名，GameplayAbilityTargetTypes.cpp:16，改不掉了）、`AbilitySystem.ServerRPCBatching.Log`（不是 AbilitySystem.LogServerRPCBatching，AbilitySystemComponent_Abilities.cpp:4161）。
