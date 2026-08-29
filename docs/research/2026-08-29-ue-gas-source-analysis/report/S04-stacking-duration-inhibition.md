# S4 · Stacking / Duration / Inhibition 时序（重章）

> 结论先行
> 1. **堆叠判定发生在应用路径最前段**（`ApplyGameplayEffectSpec` 的第 4 步，早于 capture、早于 duration 计算、早于 cue），时长刷新嵌在堆叠分支内，而取消/移除因 `GAMEPLAYEFFECT_SCOPE_LOCK` 全程被推迟到作用域结束——「堆叠、时长刷新、取消」三者的顺序是**结构上固定的**：堆叠合并 → 溢出处理 → Spec 整体替换 + StackCount 更新 → 时长策略 → 周期策略 → （锁外的）延迟取消执行（GameplayEffect.cpp:4171-4561）。
> 2. 堆叠被拒（溢出策略拒绝）时 `ApplyGameplayEffectSpec` 返回 **nullptr**、外层返回**默认构造的无效 handle**——调用方**无法区分**"堆叠拒绝/免疫/无权限/周期不可预测"，这些全都是同一个无效值（对比：成功 instant 返回的是 `GetInstantExecutedHandle()` 特殊哨兵，`WasSuccessfullyApplied()==true` 而 `IsValid()==false`）。
> 3. 抑制（Inhibition）= **把 mod 和 tag 从聚合器/计数容器里物理移除**（不是求值时跳过）；`bIsInhibited` 字段**不复制**（Epic 在字段注释里自述"Not sure if this should replicate or not"，GameplayEffect.h:1440-1442），客户端靠 TagCountContainer 驱动的组件回调**独立重算**出同样的抑制态。

---

## 4.1 待证清单裁决表

| # | 预研说法 | 裁决 | 证据 |
|---|---|---|---|
| 4.1 | 堆叠/时长刷新/取消三者先后顺序证据缺口 | **已钉死**（见 4.2 时序表） | Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/GameplayEffect.cpp:4171-4561 · FActiveGameplayEffectsContainer::ApplyGameplayEffectSpec |
| 4.2 | 堆叠被拒的返回值 | **证实：不可区分**。`HandleActiveGameplayEffectStackOverflow` 返回 false → `return nullptr` → `ApplyGameplayEffectSpecToSelf` 返回 `FActiveGameplayEffectHandle()`（Handle=INDEX_NONE, bPassedFiltersAndWasExecuted=false）；只有 VLog 一条 "Application of %s denied (StackLimit)" | GameplayEffect.cpp:4245-4252、4360-4365；AbilitySystemComponent.cpp:1080-1084；ActiveGameplayEffectHandle.h:21-60 · 哨兵语义 |
| 4.3 | Inhibition 进入/退出条件与表现 | **证实+结构修正**：5.8 的进入/退出入口是 `UAbilitySystemComponent::SetActiveGameplayEffectInhibit(handle, bInhibit, bInvokePredictedEffects)`；触发源是 GameplayEffectComponent 的 tag 事件（`TargetTagRequirementsGameplayEffectComponent` 注册 `RegisterGameplayTagEvent`）与 `OnAddedToActiveContainer` 返回 false。**抑制期间 Modifier 被移除出聚合器**（不是跳过） | AbilitySystemComponent.cpp:362-406 · SetActiveGameplayEffectInhibit；GameplayEffectComponents/TargetTagRequirementsGameplayEffectComponent.cpp:92（tag 事件注册）；GameplayEffect.cpp:972-986 · UGameplayEffect::OnAddedToActiveContainer；GameplayEffect.h:1436-1442 · bIsInhibited（默认 true，含"该不该复制"的存疑注释） |
| 4.4 | 抑制态在网络/快照中的表现 | **证实：字段不复制；客户端独立收敛**。NetDeltaSerialize 复制的 Spec 里没有 bIsInhibited 位；客户端在 `PostReplicatedAdd/Change` 后由 `HandleDeferredGameplayCues`（推迟到整个数组收完）+ 组件的 tag 回调在本地重算抑制；`bPendingRepOnActiveGC/bPendingRepWhileActiveGC` 两个 pending 标志就是"抑制未知，cue 先欠着"的机制 | GameplayEffect.h:1440-1442；GameplayEffect.cpp:2883-2885（PostReplicatedAdd 注释 "we don't know if this GE ends up inhibited or not"）；5271-5309 · PostReplicatedReceive/HandleDeferredGameplayCues |

## 4.2 堆叠应用完整时序（ApplyGameplayEffectSpec 内部顺序）

坐标：GameplayEffect.cpp:4171-4561。**先声明**：整个函数体在 `GAMEPLAYEFFECT_SCOPE_LOCK` 内，"取消"类副作用（另一个 GE 的 RemoveAbilityTags 触发的取消等）在这期间全部入 Pending 队列，锁释放后才执行——所以"取消"在时序上**必然最后**。

| 步 | 动作 | 行号 |
|---|---|---|
| 1 | Scope lock；ensure(Spec.Def) | 4175-4180 |
| 2 | 权威侧 `FlushNetDormancy` | 4184-4188 |
| 3 | `FindStackableActiveGameplayEffect(Spec)`（聚合键见 4.3） | 4191 |
| 4a | **堆叠分支**：预测堆叠开关检查（客户端不许预测时直接 nullptr；服务器作废键） | 4211-4225 |
| 4b | StackLimit 溢出判定：已在限额 或 将溢出 → `HandleActiveGameplayEffectStackOverflow`（false → **拒绝，nullptr**） | 4238-4252 |
| 4c | `UnregisterLinkedAggregatorCallbacks`；DynamicGranted/AssetTags 不同则 **ensureMsgf 告警**（@todo：未实现 diff） | 4259-4267 |
| 4d | ExtendDuration 策略：记录 `CarryOverDuration`（剩余时长） | 4271-4274 |
| 4e | **Spec 整体替换**（旧 GrantedAbilitySpecs 保留——"只在首次授予"）+ `SetStackCount(new)` | 4276-4288 |
| 4f | 时长策略：NeverRefresh（或客户端本地预测）→ 不设 timer；否则 `RestartActiveGameplayEffectDuration` | 4292-4302 |
| 4g | 周期策略：NeverReset → 不重置 period timer | 4304-4308 |
| 5a | **新 GE 分支**：`GenerateNewHandle(Owner)`；scope lock 下无 slack → 进 PendingGameplayEffect 链表 | 4310-4354 |
| 5b | 初始应用也过溢出检查（用 0 堆叠副本） | 4356-4365 |
| 6 | 公共尾：`SetCurrentAppliedGE`（全局指针）→ `GlobalPreGameplayEffectSpecApply` → 重取目标 tags → **`CaptureAttributeDataFromTarget` → `CalculateModifierMagnitudes`** | 4375-4388 |
| 7 | cue 用的 ModifiedAttribute 列表构建（duration无period / instant有period 两种才建） | 4390-4422 |
| 8 | `RegisterLinkedAggregatorCallbacks`（非快照依赖） | 4424-4425 |
| 9 | **时长计算**：Def 计算 → `CalculateModifiedDuration` → +CarryOver → MaxDuration 钳制 → ≤0 强制 0.1s → `SetDuration` → `OnDurationChange` 广播 → **duration timer 注册** | 4427-4493 |
| 10 | **period timer 注册**（`bExecutePeriodicEffectOnApplication` → next tick） | 4495-4508 |
| 11 | 预测注册（客户端本地预测 GE：MarkArrayDirty + CaughtUp/Rejected delegate） | 4510-4545 |
| 12 | 堆叠 → `OnStackCountChange(old,new)`（MarkItemDirty → UpdateAllAggregatorModMagnitudes → tag 计数通知 → OnStackChanged 广播）；非堆叠 → `InternalOnActiveGameplayEffectAdded` | 4547-4558 |
| — | （锁释放后）排队的取消/移除/添加执行 | 容器 scope lock 语义 |

**4.1 的直接回答**：堆叠合并（4a-4e）→ 时长刷新（4f，含 CarryOver 累加于 4446-4449）→ 周期重置（4g）→ 全部完成后锁外才轮到任何取消动作。`@note @todo`（4547-4548）承认此顺序"假设堆叠不改变抑制状态——复杂动态 tag 情况下可能不对"。

## 4.3 堆叠聚合键与溢出

- **聚合键**：`ActiveEffect.Spec.Def == Spec.Def`（**指针相等**，同一定义对象）+ 按 StackingType：`AggregateByTarget` 只看 Def；`AggregateBySource` 额外要求 `SourceASC == ActiveEffect.Spec.GetContext().GetInstigatorAbilitySystemComponent()`（GameplayEffect.cpp:3675-3699）。Instant 永不堆叠（3681）。容器线性扫描，注释承认"就算缓存 handle 也逃不掉线性定位（数组不稳定）"（3683-3685）。
- **溢出**（3701-3731）：OverflowEffects 逐个 `ApplyGameplayEffectSpecToSelf`（递归入应用路径）；`bDenyOverflowApplication && bClearStackOnOverflow` → 移除整个堆叠（**客户端不可预测移除**，3721-3728）；返回 `bAllowOverflowApplication || bRefreshToLimit`。行为微调 CVar：`AbilitySystem.ActiveGameplayEffectOverflowBehavior`（默认 3，GameplayEffect.cpp:90-93）。
- **部分移除**（4.2 的另一面）：`InternalRemoveActiveGameplayEffect` 若 `0 < StacksToRemove < 当前堆叠数` → 只减 StackCount + OnStackCountChange，**返回 false**（GameplayEffect.cpp:4834-4841）——"移除部分堆叠"和"移除整个 GE"复用同一个函数，返回值语义是"是否动了数组"。

## 4.4 到期路径

- **谁触发**：FTimerManager 定时器 → `UAbilitySystemComponent::CheckDurationExpired(handle)` → 容器 `CheckDuration(handle)`（GameplayEffect.cpp:5369-5495）。
- **CheckDuration 是防误删的二次确认**：timer 可能因 duration 被改而过期触发，函数内重算 `(StartWorldTime + Duration) < CurrentTime` 或差值≈0（5407）；未到期 → 只重设 timer 为剩余时间（5429-5433、5474-5491）。
- **到期 ≠ 移除**：到期后按 `EGameplayEffectStackingExpirationPolicy` 分叉——ClearEntireStack（StacksToRemove=-1）/ RemoveSingleStackAndRefreshDuration（=1 且刷新）/ RefreshDuration（**只刷新不删**，5417-5427）。
- **最后一次周期执行**：到期时若 period timer 恰好也到期且未抑制 → `InternalExecutePeriodicGameplayEffect` 先跑一轮再删（5436-5458）；执行可能反过来把 GE 杀了（如周期伤害致死清 buff），代码有 `IsPendingRemove` 早退保护（5446-5452）。
- **到期与移除是两件事**：到期只是 `InternalRemoveActiveGameplayEffect(..., bPrematureRemoval=false)` 的调用源之一；移除本体见 4.5。

## 4.5 Removal 与 Immunity 的检查点

- 应用流程的**免疫检查点**在第 5-6 步（S3.2 表）：`GameplayEffectApplicationQueries`（ASC 注册的查询委托，AbilitySystemComponent.h:73-77 声明）→ `Spec.Def->CanApply`（组件循环：Immunity/ChanceToApply/CustomCanApply 组件，GameplayEffect.cpp:958-970）。被免疫时：返回**无效 handle** + `FImmunityBlockGE` 广播（AbilitySystemComponent.h:73-74 声明该 delegate）。
- `RemoveActiveEffects(Query)`（按查询批量删）与 `RemoveActiveGameplayEffect(handle, stacks)`（单删）都汇聚到 `InternalRemoveActiveGameplayEffect`（4797-4918）：部分堆叠→只减计数；`FlushNetDormancy` 先行（"FastArray 的改动在 Dormant 时不被追踪"，4843-4851）；预测 GE 移除的 cue 抑制判断（4854-4865）；清 timer；`RemoveAtSwap`（4902）；`ForceNetUpdate()`（4908-4911 的 "Hack" 注释）。
- **RemovalTagRequirements / RemoveOtherGameplayEffectComponent**：`URemoveOtherGameplayEffectComponent` 在组件的 OnApplied/OnExecuted 时机按查询移除其它 GE（组件文件 Private/GameplayEffectComponents/RemoveOtherGameplayEffectComponent.cpp；声明见其头文件）。

## 4.6 Inhibition 机制（5.8 形态）

```
SetActiveGameplayEffectInhibit(Handle, bInhibit, bInvokePredictedEffects):
    查无此 GE → 返回无效 handle（Error 日志）
    bIsInhibited == bInhibit → 直接返回原 handle（幂等）
    置位 bIsInhibited
    ScopeLock + AggregatorOnDirtyBatch:
        bInhibit  → RemoveActiveGameplayEffectGrantedTagsAndModifiers  # mods/tags 摘出
        else      → AddActiveGameplayEffectGrantedTagsAndModifiers     # 重新挂回
    未被连带删除 → EventSet.OnInhibitionChanged.Broadcast(handle, bInhibited)
    被连带删除   → 返回无效 handle
```
（AbilitySystemComponent.cpp:362-406）

- 触发源 1：`OnAddedToActiveGameplayEffectAdded` 路径里 `UGameplayEffect::OnAddedToActiveContainer` 的组件投票（任一组件 false → 出生即抑制，GameplayEffect.cpp:972-986）。
- 触发源 2：`TargetTagRequirementsGameplayEffectComponent` 持有 OngoingTagRequirements，对每个 tag `RegisterGameplayTagEvent(NewOrRemoved)`（TargetTagRequirementsGameplayEffectComponent.cpp:92 一带），tag 计数变化→重评→SetActiveGameplayEffectInhibit。**旧的 `CheckOngoingTagRequirements`/`InhibitActiveGameplayEffect` 函数在 5.8 源码中已不存在**（全模块 grep 无命中；OngoingTagRequirements 字段在 UGameplayEffect 上已标 UE_DEPRECATED 5.3，GameplayEffect.h:2360-2361）。
- 周期 GE 在抑制期不执行（4765）；解除抑制后是否补一次由 `EGameplayEffectPeriodInhibitionRemovedPolicy`（GameplayEffect.h:737）决定。

## 4.7 意外发现

1. **`FActiveGameplayEffect::Handle` 客户端自铸**：PostReplicatedAdd 里 `Handle = FActiveGameplayEffectHandle::GenerateNewHandle(InArray.Owner)`，注释 "Handles are not replicated, so create a new one"（GameplayEffect.cpp:2870-2871）——句柄两端各玩各的，靠 (Def, PredictionKey, 数组身份) 对齐。
2. **客户端堆叠收敛用差值回退**：`OnPredictiveGameplayEffectStackCaughtUp` 比较 `Spec.GetStackCount()` 与 `ClientCachedStackCount`，差多少删多少（GameplayEffect.cpp:3593-3614）；注释承认"reject delegate 在坏网络下不保证被调，会剩下多余堆叠"（4540-4542）。
3. **堆叠替换 Spec 时 GrantedAbilitySpecs 单独搬回**（4276-4284）：能力只授一次，加堆不重授——"We only grant abilities on the first apply"。
4. Epic 对 `StackDurationRefreshPolicy` 缺 EditCondition 的 TODO 写着目标版本 **"5.11"**（GameplayEffect.h:2405，由告解扫描代理发现）——5.8 的堆叠字段还没做完数据校验。
5. 预测堆叠默认放行（`AbilitySystem.AllowPredictiveStackingGEs=true`，GameplayEffect.cpp:95）是**新默认**；关掉即回到"客户端完全不预测堆叠"的老行为（4213-4225 双分支都在）。

## 4.8 对目标环境的迁移含义

目标 Effect 状态机冻结为 `Pending → Active → Expired | Removed (+Rejected/RolledBack)`，且"堆叠/时长刷新是 Active 内事件"。对照 GAS：
- **抑制态必须显式建模**：GAS 把它做成"存在但 mods 摘除"的隐式态且不复制，客户端各自重算——目标引擎若有 Schema 封闭字段集，`bIsInhibited` 应该是**复制字段**（或由可复制的输入决定性推导），否则"快照后重放"无法恢复抑制语义。GAS 的"摘除 mods 而非求值跳过"这一选择值得照抄：它让抑制对求值顺序零影响。
- **堆叠合并 = Spec 替换**：GAS 的堆叠不是"事件流"而是"新 spec 覆盖旧 spec + 差额计数"。目标引擎若坚持"堆叠是 Active 内事件"，需要明确事件载荷 = (新 spec 快照, 新 stack count, 时长策略结果)，且**替换原子性**必须等于 GAS 的 scope-lock 原子性。
- **到期与移除解耦**（expiration policy 可以 RefreshDuration 无限续）与"Expired 是终态"冲突：目标引擎里 RefreshDuration 应表达为"Active 内的 duration 重置事件"，而不是阻止进入 Expired。
- 部分堆叠移除返回 false 的语义（"没动数组"）是 GAS 内部约定，目标引擎的 `Removed` 事件应携带移除堆叠数，避免这种布尔复用。
