# S3 · GameplayEffect 应用路径与 Modifier 求值顺序（重章）

> 结论先行
> 1. **单通道内的求值公式是固定的**：`Override` 先查（**数组序第一个符合条件的赢**，直接返回）；其余按 `((Base + Additive) × Multiplicitive ÷ Division × CompoundMultiply) + FinalAdd` 一步算出（GameplayEffectAggregator.cpp:76-99）。乘法类 mod 不是连乘，而是 **`(1 + Σ(m−1))` 的加性聚合**（SumMods 带 bias，GameplayEffectAggregator.cpp:216-229）。
> 2. **通道间是升序串行**：`ModChannelsMap` 是 TMap，但每次 `FindOrAddModChannel` 插入新通道后立刻 `KeySort(TLess<...>)` 重排（GameplayEffectAggregator.cpp:231-243），求值按枚举值升序把上一通道输出当下一通道 base（250-261）。通道内数组序 = 插入序，但 `RemoveAllSwap` 会打乱（150-162）。
> 3. Instant 与 Duration 的分叉点在 `ApplyGameplayEffectSpecToSelf`：`DurationPolicy != Instant` 才进容器走 Aggregator；Instant 直接 `ExecuteGameplayEffect` 改 **BaseValue**（AbilitySystemComponent.cpp:1078-1101、1158-1162）。预测的 Instant 被临时改造成 INFINITE_DURATION 走容器（1066、1112-1117）。

---

## 3.1 待证清单裁决表

| # | 预研说法 | 裁决 | 证据 |
|---|---|---|---|
| 3.1 | Modifier 顺序「基础值→各聚合通道→当前值」只有推断 | **证实+重大细化**：确为「base → 通道升序 → final」，但通道**内部**不是逐 mod 顺序应用，而是按固定公式聚合（四种主 op + 5.8 实际有 9 种 op：Additive/Multiplicitive/Division/Override/AddFinal/MultiplyCompound/AddBase/MultiplyAdditive/DivideAdditive，见 StaticExecModOnBaseValue 的 switch） | Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/GameplayEffectAggregator.cpp:76-99 · FAggregatorModChannel::EvaluateWithBase；447-479 · FAggregator::StaticExecModOnBaseValue（9 个 op 分支）；250-261 · FAggregatorModChannelContainer::EvaluateWithBase |
| 3.2 | 同组 Effect 不同应用顺序结果是否相同未确证 | **裁决：数学上多数可交换，三个例外**。① **Override：先加者赢**（数组序第一，GameplayEffectAggregator.cpp:78-84），且 `RemoveAllSwap`（150-162）会改变"谁在第一位"→ 应用/移除历史影响结果；② Division 总和≈0 时被强制置 1（92-96）；③ float 加法在 SumMods 中按数组序累加（216-229），末位差异依赖插入序。**没有 Priority 字段参与 Modifier 排序**（GE 定义里的优先级概念不存在；Override 的"优先级"就是插入序） | 同上坐标 |
| 3.3 | ModChannel 容器与遍历顺序 | **证实：TMap<枚举,通道>，插入时 KeySort 保升序**；求值迭代 Map（排序后的元素链）。结论：通道顺序确定；通道内顺序=插入序（RemoveAllSwap 破坏） | GameplayEffectAggregator.h:272-276 · ModChannelsMap 声明；GameplayEffectAggregator.cpp:231-243 · FindOrAddModChannel（含"resort the map to preserve key order"注释）；135-148 · AddMod（append） |
| 3.4 | 同优先级 tie-break | **裁决：没有稳定排序，tie-break 就是容器数组序**；插入序影响 Override 赢家与 float 求和次序 | GameplayEffectAggregator.cpp:137-148（AddMod append）；150-162（RemoveAllSwap） |
| 3.5 | 快照×来源/目标四象限 | **证实**：Source 侧快照在 **Spec 创建时**（MakeOutgoingSpec→Initialize→CaptureDataFromSource）；Target 侧捕获在**应用时**（ApplyGameplayEffectSpec 内 `CaptureAttributeDataFromTarget(Owner)`）；非快照属性通过 `RegisterLinkedAggregatorCallbacks` 挂活聚合器回调实时重算 | GameplayEffect.cpp:1838-1859 · FGameplayEffectSpec::CaptureDataFromSource；4387 · 应用点 CaptureAttributeDataFromTarget；2474-2486 · RegisterLinkedAggregatorCallback；2570-2583 · CaptureAttributes（快照/非快照分支） |

## 3.2 应用路径总控制流（ApplyGameplayEffectSpecToSelf）

坐标：Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/AbilitySystemComponent.cpp:996-1179 · UAbilitySystemComponent::ApplyGameplayEffectSpecToSelf

```
ApplyGameplayEffectSpecToSelf(Spec, PredictionKey):
  1  FScopedActiveGameplayEffectLock + FScopeCurrentGameplayEffectBeingApplied(全局"正在应用"指针)
  2  Spec.Def == null                        → 返回无效 handle
  3  !HasNetworkAuthorityToApplyGameplayEffect(Key) → 无效 handle   [预测权限门]
  4  预测键有效 && Period>0:
       服务器 → 作废键继续;  客户端 → 直接返回（周期不可预测）        [早退]
  5  GameplayEffectApplicationQueries 逐个投票 → 任一 false 即拒      [外部注册的免疫查询]
  6  Spec.Def->CanApply(容器, Spec) → 组件循环 CanGameplayEffectApply
       （ChanceToApply / CustomCanApply / Immunity 组件在此）          [免疫检查点]
  7  Modifiers 逐个检查 Attribute.IsValid() → 空属性即拒
  8  bTreatAsInfiniteDuration = 非权威 && IsLocalClientKey && Instant  [预测改造]
  9  if Duration != Instant || bTreatAsInfiniteDuration:
        ActiveGameplayEffects.ApplyGameplayEffectSpec(...) → 失败返回无效 handle
        MyHandle = AppliedEffect->Handle; OurCopyOfSpec = 容器内副本
       else: 本地复制 Spec → GlobalPreGameplayEffectSpecApply → CaptureAttributeDataFromTarget
 10  bTreatAsInfiniteDuration → SetDuration(INFINITE_DURATION, lock)
 11  堆叠命中且未抑制 → 补播 OnActive/WhileActive cue（RPC，"没有复制'重新触发'的手段"）
 12  if bTreatAsInfiniteDuration: 预测 Execute cue（不执行 modifier！）
     elif Instant:               ExecuteGameplayEffect → ExecuteActiveEffectsFrom [真正执行]
 13  Spec.Def->OnApplied（组件 OnGameplayEffectApplied）
 14  OnGameplayEffectAppliedToSelf/ToTarget delegates；返回 MyHandle
```

要点：
- **Instant 的执行定义**：`ExecuteGameplayEffect` 有 `check(Spec.GetDuration()==INSTANT_APPLICATION || Period!=NO_PERIOD)`（AbilitySystemComponent.cpp:1216）——只有 instant 或 periodic 的 spec 允许进执行路径。
- 执行主循环 `ExecuteActiveEffectsFrom`（GameplayEffect.cpp:3210-3370）：重取目标 tags → `CalculateModifierMagnitudes` → **Modifiers 循环**（tag 要求满足才执行，由 `AbilitySystem.Fix.UseModTagReqsOnAllGE`（默认 true，UE5.4 修复）控制，3243-3256）→ **Executions 循环**（Exec CDO → 输出 mods 按堆叠数自动放大（3290-3299）→ 条件 GE 排队）→ cue 触发判定（`bRequireModifierSuccessToTriggerCues`、执行手动接管）→ 条件 GE `ApplyGameplayEffectSpecToSelf`（3360-3366）→ `Def->OnExecuted`。
- **单 modifier 执行**（InternalExecuteMod，GameplayEffect.cpp:4090-4153）：`PreGameplayEffectExecute`（返回 false 即跳过）→ `ApplyModToAttribute`（对 **BaseValue** 执行 `StaticExecModOnBaseValue`，4155-4169）→ 累计 ModifiedAttribute.TotalMagnitude → `PostGameplayEffectExecute`。

### Instant 为什么改 Base、Duration 为什么走 Aggregator

- Instant：`InternalExecuteMod → ApplyModToAttribute → SetAttributeBaseValue` —— 直接写聚合器的 BaseValue，不留痕（GameplayEffect.cpp:4115-4123）。
- Duration/Infinite：`ApplyGameplayEffectSpec → InternalOnActiveGameplayEffectAdded → AddActiveGameplayEffectGrantedTagsAndModifiers → 对每个 Modifier `Aggregator->AddAggregatorMod(...)` 进通道`（GameplayEffect.cpp:4564-4743；GameplayEffectAggregator.cpp:487-493）——mod 进聚合器，Base 不动，CurrentValue = Evaluate。
- 分叉的**那一行**：AbilitySystemComponent.cpp:1078 `if (Spec.Def->DurationPolicy != EGameplayEffectDurationType::Instant || bTreatAsInfiniteDuration)`。

## 3.3 Modifier 求值顺序（可照抄级细节）

FAggregatorModChannel::EvaluateWithBase（GameplayEffectAggregator.cpp:76-99）伪代码：

```
EvaluateWithBase(Base, Params):
    for mod in Mods[Override]:            # 数组序
        if mod.Qualifies(): return mod.EvaluatedMagnitude   # 首个命中即整体返回
    Add    = SumMods(Mods[Additive],      bias=0)    # = Σ m
    Mult   = SumMods(Mods[Multiplicitive],bias=1)    # = 1 + Σ (m-1)   ← 加性聚合!
    Div    = SumMods(Mods[Division],      bias=1)    # = 1 + Σ (m-1)/m
    Final  = SumMods(Mods[AddFinal],      bias=0)
    Cmpd   = Π Mods[MultiplyCompound]                # 唯一真连乘
    if |Div|≈0: Div=1 (Warning)
    return ((Base + Add) * Mult / Div * Cmpd) + Final
```

- Qualifies 的计算在求值前统一做一遍（`UpdateQualifies`：预测 mod 默认排除、IgnoreHandles、来源/目标 tag 过滤——过滤解析还要查 `ActiveHandle.GetOwningAbilitySystemComponent()` 的活 tags，GameplayEffectAggregator.cpp:28-74）。
- 通道间：`for (ChannelEntry : ModChannelsMap) ComputedValue = Channel.EvaluateWithBase(ComputedValue)`（250-261）——升序链式。
- **ReverseEvaluate**（客户端反推 base）：通道逆序，遇 Override 直接放弃返回 FinalValue（101-133；"This is the case we can't really handle due to lack of information"）。注释标明"随 struct-based attributes 转型将被废弃"（GameplayEffectAggregator.h:105-108）。

### EGameplayModOp 的真实取值集（5.8）

`StaticExecModOnBaseValue` 的 switch 覆盖：Override / AddBase / AddFinal / MultiplyAdditive / MultiplyCompound / DivideAdditive（GameplayEffectAggregator.cpp:447-479），加上通道公式用到的 Additive / Multiplicitive / Division —— **9 种操作符**，不是社区常说的 4 种。其中 AddBase/MultiplyAdditive/DivideAdditive 属于 base 值直接操作族（instant 执行用），AddFinal/MultiplyCompound 是为求值通道引入的新 op。

## 3.4 Attribute Capture 四象限

| | Source | Target |
|---|---|---|
| **快照** | Spec 创建时 `CaptureDataFromSource`（GameplayEffect.cpp:1838-1859）拍进 `CapturedSourceTags`/capture specs | 应用时拍（ApplyGameplayEffectSpec:4387 `CaptureAttributeDataFromTarget`；预测执行路径 3095-3098 重取目标 tag） |
| **非快照** | `RegisterLinkedAggregatorCallbacks`（GameplayEffect.cpp:4424-4425）挂源 ASC 聚合器 OnDirty → `OnMagnitudeDependencyChange` 两遍式重算（3513-3570） | 同机制挂目标聚合器；重算时机 = 任意相关聚合器变脏 |

- `CaptureAttributes`（GameplayEffect.cpp:2570-2583）：`InCaptureSource==Source && bSnapshot` → `AttemptGetAttributeAggregatorSnapshot`（拷贝聚合器）；否则 `CaptureAttributeForGameplayEffect`（抓当前 Evaluate 值）。快照的载体是**整份 FAggregator 拷贝**（TakeSnapshotOf，GameplayEffectAggregator.cpp:579-583、653-666），不是单 float。
- 捕获定义在 Spec Initialize 时静态收集（SetupAttributeCaptureDefinitions，GameplayEffect.cpp:1777-1831）。

## 3.5 FGameplayEffectSpec / FGameplayEffectContext 存了什么（网络成本）

FGameplayEffectSpec 主字段（GameplayEffect.h 的 USTRUCT 定义；本表按字段职责归纳）：
- `Def`（UGameplayEffect*，类引用走路径序列化）、`Level`、`Duration`（float，可被 SetByCaller/curve 改写）、`Period`、`StackCount`（int32，`FStackCountData` 实际是 int+locked 位）
- `Modifiers: TArray<FModifierSpec>`（每项 = evaluated magnitude float）
- `ModifiedAttributes: TArray<FGameplayEffectModifiedAttribute>`（cue 用 magnitude 信息）
- `CapturedSourceTags / CapturedTargetTags`（含 actor tags + 处理过的 spec tags）
- `CapturedRelevantAttributes: FGameplayEffectAttributeCaptureSpecContainer`（快照聚合器）
- `SetByCallerMagnitudes: TMap<FGameplayTag,float>`（**非 UPROPERTY，不上网**）
- `DynamicGrantedTags / DynamicAssetTags`、`EffectContext`（FGameplayEffectContextHandle）
- 网络成本：`FGameplayEffectSpecForRPC`（GameplayEffectTypes.cpp:2225-2247 一带）是复制用瘦身版；FastArray 里 AGE 的 NetSerialize 序列化 Spec 时按 NetIndex 过滤 Modifier（见 S7）。

FGameplayEffectContextHandle（GameplayEffectTypes.h）：`TSharedPtr<FGameplayEffectContext>`——**指针语义、跨端不可直接比较**；Context 里 Instigator/Source/Target 的 `TWeakObjectPtr` 需要客户端对象映射完成后才有效（PostReplicatedReceive 的 unmapped 检查即为此，GameplayEffect.cpp:5290-5305）。

## 3.6 Execution / ModMagnitudeCalculation 的调用时机与可改面

| | 调用时机 | 能改什么 |
|---|---|---|
| UGameplayEffectExecutionCalculation | ExecuteActiveEffectsFrom 的 Executions 循环（GameplayEffect.cpp:3271-3325；预测变体 3138-3172） | 只能输出 `FGameplayModifierEvaluatedData`（mod 集合）；可读 capture、可标记手动接管 cue/堆叠；**不能**改其它属性/GE |
| UGameplayModMagnitudeCalculation | CalculateModifierMagnitudes（GameplayEffect.cpp:2033-2047）在应用/执行前统一算 | 只产出 float 幅度；依赖属性变化时经 `AttemptRecalculateMagnitudeFromDependentAggregatorChange` 重算（1271-1293） |

## 3.7 Meta Attribute 模式

**源码无专门机制，纯约定**：meta 属性 = 只被 Execution 修改、且 AttributeSet 在 Pre/PostGameplayEffectExecute 里把它转写进真实属性的属性。证据：
- GameplayPrediction.h:228-235（Epic 自述："Meta attributes only work on instant effects, in the back end ... Pre/PostModifyAttribute ... not called when applying duration-based"）。
- `CanApplyAttributeModifiers` 只对 **Additive** op 检查 `CurrentValue + Cost < 0`（GameplayEffect.cpp:5497-5528）——meta 属性的存在感只在钩子约定里。

## 3.8 周期 Effect 的时间基准

- **FTimerManager 驱动**，不是 Tick、不是惰性检查：`TimerManager.SetTimer(PeriodHandle, Delegate=ExecutePeriodicEffect, Period, bLoop=true)`（GameplayEffect.cpp:4496-4508）；`bExecutePeriodicEffectOnApplication` → 额外 `SetTimerForNextTick`（4502-4505）。
- 到期同样是 timer：`SetTimer(DurationHandle, CheckDurationExpired, FinalDuration, false)`（4481-4492）。
- **周期漂移无补偿**：SetTimer 是标准 FTimerManager 定时，World 时间驱动；CheckDuration 的兜底逻辑会重设 duration timer 为剩余时间（5474-5491），但对 period 没有累积误差修正。
- 时间源：`GetWorldTime()` = `World->GetTimeSeconds()`；`GetServerWorldTime()` = `GameState->GetServerWorldTimeSeconds()`（无 GameState 时退回本地时间，GameplayEffect.cpp:5334-5367）。AGE 同时记录 `StartWorldTime`（本地）与 `StartServerWorldTime`（服务器），客户端 PostReplicatedAdd 时做时钟重定位（见 S7）。
- 抑制期间的周期：`InternalExecutePeriodicGameplayEffect` 开头 `if (!ActiveEffect.bIsInhibited)`（4762-4794）；解除抑制是否补执行由 `EGameplayEffectPeriodInhibitionRemovedPolicy`（GameplayEffect.h:737）决定——**policy 枚举存在**，这正是周期×抑制交互的显式配置点。

## 3.9 意外发现

1. **`SetCurrentAppliedGE` 全局指针**（GameplayEffect.cpp:4233、4376；AbilitySystemComponent.cpp:1006 的 FScopeCurrentGameplayEffectBeingApplied）：进程级"正在应用的 GE"单值栈——重入/递归应用 GE 时的上下文靠它传递，非线程安全、非快照友好（S11 直接引用）。
2. **应用期的容器锁 + 挂起链表**：`GAMEPLAYEFFECT_SCOPE_LOCK` 期间新 GE 不进 TArray 而是进 `PendingNext` 侵入式链表（GameplayEffect.cpp:4314-4354，含三处 `[#1][#2][#3] If you change this, please change` 的同步注释）——为了 scope lock 下不搬移别人正持有的指针。
3. **Duration 被 modifier 改到 ≤0 时强制 0.1s 并 Error 日志**（GameplayEffect.cpp:4463-4467），"We cannot mod ourselves into an instant or infinite duration effect"。
4. **MaxDuration**（4451-4472）：spec 级 MaxDurationMagnitude，超出即钳制——5.8 新增的防叠层膨胀机制。
5. TODO @ 3200/3352："Right now we will replicate every execute via a multicast RPC"——执行 cue 的复制策略未按 GE 配置过滤，Epic 自己标注。
6. `bUseModifierTagRequirementsOnAllGameplayEffects`（GameplayEffect.cpp:99-102，默认 true）：UE5.4 起 Instant/Periodic 也检查 modifier tag 要求，注释明说"Duration effects 用 FAggregatorMod::UpdateQualifies 是另一条路"。

## 3.10 对目标环境的迁移含义

- 求值公式的**形状**（分组聚合 + 固定算子优先 + 通道升序）可直接移植为纯函数，且天然适合「规范化字节」：只要规定通道枚举封闭、每组内按注册序（或显式 sort key）定序、浮点和用确定性归约（如 Kahan 或定点），S3.2 列的三个顺序脆弱点全部可消除——GAS 没做不是因为难，而是因为它不承诺确定性。
- **Override 首加者赢**语义对目标引擎是个危险默认值：与「堆叠刷新是 Active 内事件」的冻结语义组合时，建议改为「最后写入赢 + 显式优先级字段」，并把 RemoveAllSwap 式的隐式重排序禁掉（ECS 侧用稳定索引）。
- Execution 输出 mod 集合的**单向数据流**（计算只产 mod、应用只消费 mod）值得保留；它正是目标引擎「公式虚拟机推迟后」的安全替代形状。
- 周期效果用引擎 Timer 在目标引擎不可用（非确定时间源）——应改为逻辑帧计数（period 以 tick 数表达），这在 GAS 侧对应物是 `bExecutePeriodicEffectOnApplication` + timer 的组合语义，需要显式重建。
