# S9 · GameplayCue 的实际网络路径

> 结论先行
> 1. Cue 有**三条网络路径**：① 持续 cue 走 `ActiveGameplayCues` FastArray（可靠、状态型）；② Minimal 模式下模拟代理走 `MinimalReplicationGameplayCues`（COND_SkipOwner 属性）；③ 执行/添加事件走 **NetMulticast RPC（不可靠）**——三路由 `ReplicationMode` 与 cue 类型分派。
> 2. **不可靠事件没有补偿机制**：丢包即丢事件；Epic 只在 Todo 里写过"Could implement non-rpc method for replicating if desired"（GameplayCueManager.cpp:1529）。持续 cue 不会丢（状态型），但 Executed 型丢了不重发。
> 3. 逻辑/表现的边界画在 **EGameplayCueEvent 三事件 + UGameplayCueSet 查找表**这一层，是**约定不是类型强制**——ASC 有 `bSuppressGameplayCues` 全局闸门、GE 有 `bSuppressGameplayCues/bSuppressStackingCues`，但没有任何机制阻止 Cue 里写游戏逻辑。

## 9.1 分派与可靠性（坐标为证）

| 路径 | 触发 | 可靠性 | 坐标 |
|---|---|---|---|
| FastArray 持续 cue | AddGameplayCue（权威）→ ForceReplication + 容器 AddCue + NetMulticast_InvokeGameplayCueAdded_WithParams | 可靠（状态） | AbilitySystemComponent.cpp:1555-1615 · AddGameplayCue_Internal |
| Minimal cue（模拟代理） | 同上但写 MinimalReplicationGameplayCues（bMinimalReplication 容器） | 可靠（属性） | 同上 1575-1592（"Original Hack"区）；条件注册 1873/1885 |
| Executed 事件 | InvokeGameplayCueExecuted[_WithParams/_FromSpec] → PendingCue 队列 → FlushPendingCues → **NetMulticast RPC**（批量按 tag 数分单/多发） | **不可靠** | GameplayCueManager.cpp:1423-1580 · Invoke/Flush；CheckForTooManyRPCs 对 `net.MaxRPCPerNetUpdate` 报警（1020 一带） |
| 预测 cue | 非权威 + IsLocalClientKey → PredictiveAdd + 本地立即 OnActive/WhileActive | 本地 | AbilitySystemComponent.cpp:1607-1614 |
| GE 内嵌 cue | GE 应用路径的 InvokeGameplayCueAddedAndWhileActive_FromSpec / Executed_FromSpec（见 S3/S4 各调用点） | 混合 | GameplayEffect.cpp:1127-1156；GameplayCueManager.cpp:1326-1378 |

## 9.2 预测吸收（不重播的机制）

所有下行 cue RPC 实现统一守卫 `IsOwnerActorAuthoritative() || PredictionKey.IsLocalClientKey() == false`（AbilitySystemComponent.cpp:1652-1747 全系列）。Mixed 模式特例：服务器发的 server-initiated key 对拥有客户端跳过 OnActive RPC（1711-1716，等 FastArray 的真消息）——即 S8 引用的 "Original Hack"。

## 9.3 静态 vs Actor 型 Cue 的生命周期

- `UGameplayCueNotify_Static`/`Burst`：非实例化，OnActive/WhileActive/Executed 直接调 CDO。
- `AGameplayCueNotify_Actor`/`BurstLatent`/`Looping`：实例化 Actor，有回收池（`AbilitySystem.GameplayCueActorRecycle` 默认 1；`GameplayCueNotify_Actor.cpp:32` 的 ClearCueNotifyTimers；GameplayCue_Types.h:74 的 PreallocationInfo 池化）。
- 移除平衡：`GameplayCue.Fix.UseEqualTagCountAndRemovalCallbacks`（默认 true，GameplayCueInterface.cpp:32）修复"Add 回调与 Remove 回调数不等"的历史 bug；`AbilitySystem.GameplayCueNotifyTagCheckOnRemove`（默认 1）移除时复查目标已无该 tag。

## 9.4 Late join 补齐

持续 cue：状态随 FastArray 到达，补 WhileActive（GE 的 3 秒规则同样作用于内嵌 cue，GameplayEffect.cpp:2842-2858）；Executed 事件：**不补**（不可靠事件无历史）。`FMinimalGameplayCueReplicationProxy`（GameplayCueInterface.h:206）是最小复制模式的状态型替身。

## 9.5 意外发现与迁移含义

- 意外发现：① GameplayCueManager 的发送上下文（StartGameplayCueSendContext/FlushPendingCues，1487-1504）把一帧内多个 cue 合并冲刷——天然的"帧提交点"形状；② `AbilitySystem.GameplayCue.RunOnDedicatedServer`（默认 0）允许专用服也跑 cue（表现层越界的口子）；③ 弃用类 `UGameplayCueNotify_HitImpact` 的 ShortTooltip 让用户"改用 UFortGameplayCueNotify_Burst"——**Fortnite 类名泄漏进引擎源码**（GameplayCueNotify_HitImpact.h:20）。
- 迁移含义：目标引擎「表现事件」对应 Executed/WhileActive。GAS 的教训是**状态型持续 cue（可补）与事件型爆发 cue（不可补）必须显式分层**——ECS 权威存储下，把持续 cue 表达为快照投影、爆发 cue 表达为订阅流，正好复刻 GAS 的可用性而不继承其 RPC 丢失语义。类型系统强制（Cue 无游戏态写权限）在目标引擎可实现：Cue 事件只读快照。
