# S7 · 复制路径全景（重章）

> 结论先行
> 1. ASC 注册的复制属性共 **15 组**（GetLifetimeReplicatedProps 逐行见下），复制条件分五档：COND_Dynamic（AGE 容器，按 ReplicationMode 动态切换）/ COND_None / COND_ReplayOrOwner（已授予能力）/ COND_OwnerOnly（预测键表、输入阻塞）/ COND_SkipOwner（最小复制 Cue/Tag）。
> 2. 三种 ReplicationMode 的**全部**分支只有两处：`GetReplicationCondition()`（返回 COND_*）与 `NetDeltaSerialize()`（连接级判断），二者由 CVar `AbilitySystem.UseReplicationConditionForActiveGameplayEffects`（默认 true）选择走哪条；**Mixed 模式的著名陷阱在源码上就是 `ParentOwner->GetNetConnection() == Connection` 一族判断**——owner actor 必须被接收连接拥有。
> 3. GAS 的同步哲学：**状态为主、事件为辅的双层结构**——权威状态（属性 base、AGE FastArray、tag 计数、能力列表）全部走状态复制；事件（激活确认、cue 执行、目标数据）走可靠/不可靠 RPC，且**每个事件 RPC 都带 PredictionKey 做去重吸收**（`IsLocalClientKey()==false` 守卫）。逐字段证据见 7.6。

## 7.1 待证清单裁决表

| # | 预研说法 | 裁决 | 证据 |
|---|---|---|---|
| 7.1 | 属性到底复制什么 | **裁决：Base 与 Current 都在 UPROPERTY 里，但复制的是整个 AttributeSet 子对象通道上的属性值**；`FGameplayAttributeData{BaseValue,CurrentValue}` 双 float 均 BlueprintReadOnly 复制标记（AttributeSet.h:48-53）；客户端以服务器值为**新 Base/或反推 Base**重算 Current（legacy float 走 ReverseEvaluate，FGameplayAttributeData 走直接 base，GameplayEffect.cpp:3487-3495 的分支） | AttributeSet.h:48-53；AbilitySystemComponent.cpp:1940-1946（子对象复制）；GameplayEffect.cpp:3743-3772 · SetBaseAttributeValueFromReplication |
| 7.2 | FastArray 增量粒度 | **项级增量**：FActiveGameplayEffectsContainer::NetDeltaSerialize → FastArrayDeltaSerialize<FActiveGameplayEffect>（GameplayEffect.cpp:5264）；变项以 ReplicationKey 变化识别、整项序列化（项内属性级 delta 是引擎 FastArray 机制，详见 DS 篇）；**FGameplayAbilitySpecContainer 同构**（GameplayAbilitySpec.h:300-338，ShouldWriteFastArrayItem 按 ShouldReplicateAbilitySpec 过滤） | 同上两坐标 |
| 7.3 | 三种 ReplicationMode 分支 | **证实，仅两处分支**（GetReplicationCondition / NetDeltaSerialize 连接过滤），Minimal→false（不复制）、Mixed→仅 owner 连接（含 replay 例外）、Full→全部 | GameplayEffect.cpp:5183-5217 · GetReplicationCondition；5219-5269 · NetDeltaSerialize；AbilitySystemComponent.cpp:1849-1850（COND_Dynamic）+ 1879-1899（动态条件设置） |
| 7.4 | Mixed 模式挂载陷阱 | **源码 = `Owner->GetOwner()` 的所有权/连接判断**：`ParentOwner->IsOwnedBy(Connection->OwningActor) \|\| ParentOwnerNetConnection == Connection`（遍历 ChildConnection 兼容分屏）；不满足即不复制 AGE。**ASC 挂在 PlayerState（连接拥有）成立；挂在不被该连接拥有的 Actor 上则拥有客户端也收不到全量**——枚举注释自述"does not work for Owned AbilitySystemComponents (Use Mixed instead)"是反向的同一件事（AbilitySystemComponent.h:83-88） | GameplayEffect.cpp:5230-5261 · Mixed 连接过滤（5241-5258 所有权判断与子连接循环） |
| 7.5 | RPC batching | **证实**：恰好打包 TryActivate+TargetData+End 三调用，共享一个 PredictionKey；限制=必须走 FScopedServerAbilityRPCBatcher 窗口、乱序入队有放行规则、服务器侧用 FakeInfo（"probably bogus"自述） | AbilitySystemComponent_Abilities.cpp:4184-4252 · ServerAbilityRPCBatch 系；4254-4334 · 三个 CallServer* 入队 |
| 7.6 | 属性与 Effect 到达顺序不一致 | **源码防护存在且多层**：① `ReplicatedPredictionKeyMap` 声明为 ASC **最后一个**复制属性（头注释"has to come *last* ... to ensure OnRep/callback order"，AbilitySystemComponent.h:1951-1953）；② 客户端收 AGE 数组时 PreNetReceive 加容器锁（AbilitySystemComponent.cpp:1959-1975）；③ Cue 推迟到整个数组收完后（PostReplicatedReceive→HandleDeferredGameplayCues，GameplayEffect.cpp:5271-5309）；④ 属性 OnRep 与容器更新二路归一用 NetUpdateID 去重（GameplayEffect.cpp:3463-3498 大注释）；⑤ PostReplicatedAdd 里 `|DeltaServerWorldTime|<3s` 才补 OnActive cue（2842-2858） | 上述坐标 |

## 7.2 ASC 复制属性全景（逐行，GetLifetimeReplicatedProps）

坐标：Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/AbilitySystemComponent.cpp:1842-1877

| 属性 | 条件 | 机制 | 内容 |
|---|---|---|---|
| ActiveGameplayEffects | **COND_Dynamic**（CVar 开） | FastArray NetDelta | 每个 Active GE：Spec + PredictionKey + GrantedAbilityHandles + StartServerWorldTime（GameplayEffect.h:1416-1431）；Handle/StartWorldTime/bIsInhibited/ClientCachedStackCount/timer 全 NotReplicated 或裸字段（1434-1451） |
| SpawnedAttributes | COND_None | 子对象列表 | AttributeSet 实例（属性值在其中） |
| ActiveGameplayCues | COND_None | FastArray | 持续 cue（FActiveGameplayCueContainer，GameplayCueInterface.h:130） |
| RepAnimMontageInfo | COND_None | USTRUCT+NetSerialize | montage 复制态（GameplayAbilityRepAnimMontage.h:104） |
| OwnerActor / AvatarActor | COND_None | 对象引用 | 拥有者/化身 |
| GameplayTagCountContainer | COND_None | 属性（5.8 新路） | tag 计数（替代两条弃用旧路） |
| ReplicatedLooseTags | COND_None | 属性 | 旧路（UE_DEPRECATED 5.7） |
| ActivatableAbilities | **COND_ReplayOrOwner** | FastArray | 已授予能力 spec（GameplayAbilitySpec.h:300） |
| ClientDebugStrings / ServerDebugStrings | COND_ReplayOnly | 属性 | 调试串 |
| BlockedAbilityBindings | COND_OwnerOnly | 属性 | 输入阻塞计数 |
| **ReplicatedPredictionKeyMap** | **COND_OwnerOnly** | FastArray | 预测键确认表（必须最后注册） |
| MinimalReplicationGameplayCues | **COND_SkipOwner** | 属性+自定义条件 | 模拟代理 cue |
| MinimalReplicationTags | **COND_SkipOwner** | 属性 | 模拟代理 tag |

上行 RPC（客户端→服务器，全部 `_Validate` 返回 true 除 TargetData 验指针）与服务器校验点：ServerTryActivateAbility（真校验=重跑 InternalTryActivateAbility）、ServerSetInputPressed/Released、ServerEndAbility/ServerCancelAbility（NetSecurityPolicy 拦截）、ServerSetReplicatedTargetData（**只 ensure 指针有效，无游戏逻辑校验**，AbilitySystemComponent_Abilities.cpp:4033-4045）、ServerSetReplicatedEvent（3934）、ServerAbilityRPCBatch、ServerCurrentMontage*（校验客户端位置容差，3634-3719）、ServerSetReplicatedPredictionKey。

## 7.3 FastArray 三回调在 GAS 里的行为（乱序/丢包视角）

坐标：GameplayEffect.cpp:2767-2940

| 回调 | 做什么 | 乱序/丢包语义 |
|---|---|---|
| PreReplicatedRemove | 构造 RemovalInfo（含 premature 判定=还有剩余时间）；InternalOnActiveGameplayEffectRemoved（cue 抑制=!bIsInhibited） | 丢包重发幂等（IsPendingRemove 防重入） |
| PostReplicatedAdd | 预测吸收（同键则不重播 cue）；**时钟重定位** StartWorldTime = 本地时间 − (服务器时间 − StartServerWorldTime)（2952-2955）；3 秒新鲜度决定 OnActive cue；**客户端自铸 Handle**；InternalOnActiveGameplayEffectAdded(bInvokePredictedEffects=false) | late join 收到旧效果：cue 只补 WhileActive 不补 OnActive |
| PostReplicatedChange | CachedStartServerWorldTime 变 → 时长刷新；ClientCachedStackCount 变 → 堆叠变化；否则 → 全量重算该 GE 的聚合器 mod | 三类变更靠客户端缓存字段区分，不依赖服务器显式事件类型 |

## 7.4 时间同步（冷却剩余的客户端算法）

- 服务器侧记录 `StartServerWorldTime = GameState->GetServerWorldTimeSeconds()`（RestartActiveGameplayEffectDuration，5175-5181；无 GameState 退回本地时间，5351-5361）。
- 客户端重定位：`StartWorldTime = WorldTime − (ServerWorldTime − StartServerWorldTime)`（2952-2955）；时长查询 = `Duration − (WorldTime − StartWorldTime)`（GetActiveEffectsTimeRemaining，5530-5553）——**纯本地时钟对服务器钟差的一次性吸收**，无持续偏移跟踪；世界时间偏移突变时全表 `RecomputeStartWorldTimes`（3825 一带）。
- 到期判定在**各自端**的 FTimerManager 上（服务器权威触发移除，客户端副本到期仅影响本地显示）。

## 7.5 Late join 补齐路径

持续 GE：FastArray 全量状态（数组就是真相）+ PostReplicatedAdd 的 3 秒规则（老效果不补爆发 cue）。持续 Cue：Age 数组 + WhileActive。已授予能力：COND_ReplayOrOwner 的 spec 数组 + OnRep_ActivateAbilities（1492）。**预测键表不补**——晚加入客户端没有未决预测。目标数据/事件：**不补**（RPC 事件天然丢失，这正是 PredictionKey 文档承认的 triggered-event 缺口，GameplayPrediction.h:220）。

## 7.6 「同步状态还是同步事件」的裁决

**双层：权威状态层 + 键控事件层。** 状态层=7.2 全表（FastArray×3 + 属性组）；事件层=全部 NetMulticast/Server RPC，且每个下行事件自带 PredictionKey，接收端 `IsLocalClientKey()==false` 才执行（GameplayCue 系全部如此，AbilitySystemComponent.cpp:1652-1747 十余个实现的统一守卫）。**ECS 权威存储的引擎可继承的层**：状态层几乎整体可继承（把 FastArray 换成 ECS 快照 diff）；事件层不可继承（键控吸收依赖 GAS 的预测协议）——目标引擎应把事件层重表达为「权威状态变化的订阅投影」。

## 7.7 与通用复制系统的接缝（详见 DS 篇）

GAS 用了：ActorChannel 子对象复制（AttributeSet/能力实例）、FastArraySerializer（AGE/Spec/预测键三处）、NetDeltaSerialize 自定义过滤、COND_Dynamic/自定义条件、属性复制 OnRep 顺序依赖。GAS 提的前提：ASC 所在 Actor 必须复制（FlushNetDormancy 遍布应用/移除路径——Dormant 期间 FastArray 改动不被追踪，GameplayEffect.cpp:4843-4851 注释）。GAS 绕过的： relevancy/dormancy 本身不管（交给 owner actor）；事件不进复制流（纯 RPC）。

## 7.8 意外发现

1. `AbilitySystem.Fix.ReplicateAbilitiesToSimulatedProxies`（默认 false，AbilitySystemComponent.cpp:45）：把能力实例子对象从 COND_ReplayOrOwner 提到 COND_None 的开关——默认**模拟代理收不到能力实例**，只有 spec 列表。
2. `AbilitySystem.Fix.ActiveGEReplicationFix` 位掩码（默认 15，GameplayEffect.cpp:76）：一个 CVar 管 4 个 AGE 复制修复的行为开关集合。
3. Minimal 复制 Cue 的自定义条件（GetReplicatedCustomConditionStates:1879-1886 + MinimalGameplayCueReplicationProxy ShouldReplicate）——Iris 的 `MinimalGameplayCueReplicationProxyReplicationFragment` 是它的 Iris 化替身。
4. `PreNetReceive/PostNetReceive` 的容器锁（1959-1975）回答了 FastArray 回调里 `ensureMsgf(ScopedLockCount > 0)` 的来源——**AGE 数组的接收被当作一次容器作用域锁**。
