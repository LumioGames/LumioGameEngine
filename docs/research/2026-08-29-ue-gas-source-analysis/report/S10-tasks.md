# S10 · AbilityTask 与跨帧挂起状态

> 结论先行
> 1. AbilityTask 是 **UObject 子对象**（GC 管理），由能力实例持有（`Ability` 弱引用 + `AbilitySystemComponent` 弱引用），全局有 **1000 个并发上限**（`AbilitySystem.AbilityTask.MaxCount`，超限 ensure + 自动 dump，AbilityTask.cpp:33-110）。
> 2. 典型任务的跨端语义分三类：**纯本地**（WaitDelay/WaitInputPress）、**双向同步**（WaitTargetData：客户端产生→服务器验证→回发）、**权威驱动**（WaitGameplayEvent 等 tag 事件双端各自触发）。等待目标数据的链路里，**服务器的"验证"只有指针有效性检查**——目标数据本身原样信任（S7 表 R16）。
> 3. 跨帧挂起状态存在**任务 UObject 的成员 + ASC 的三张缓存表**（AbilityTargetDataMap / AbilityReplicatedDataCache / generic event map，键 = `FGameplayAbilitySpecHandleAndPredictionKey`）；这些是 TMap<TSharedPtr> 形态，**不可整体序列化**——任意帧快照无法覆盖 GAS 的挂起能力。

## 10.1 生命周期与所有权

- 创建：`NewAbilityTask`（模板）→ InitTask → Ability->OnGameplayTaskActivated（挂进 ActiveTasks，GameplayAbility.cpp:1570-1578）。
- 销毁：EndTask/OnDestroy（AbilityTask.cpp:114-142，计数回收 + `Ability=nullptr` 释放 GC）；能力结束时 `Task->TaskOwnerEnded()` 反向清场（GameplayAbility.cpp:851-860，倒序遍历）。
- 复制期销毁：`PreDestroyFromReplication` 只清 Ability 指针（AbilityTask.cpp:166-170）。
- **广播闸门**：`ShouldBroadcastAbilityTaskDelegates() = Ability && Ability->IsActive()`（197-207）——能力已结束时任务回调静默吞掉（可选警告 CVar `AbilitySystem.AbilityTaskWarnIfBroadcastSuppress`）。
- 挂起状态位：`WaitStateBitMask`（WaitingOnGame/WaitingOnUser/WaitingOnAvatar，224-270）+ `Ability->NotifyAbilityTaskWaitingOnPlayerData` 通知能力。

## 10.2 目标数据链路（客户端选目标 → 服务器）

```
客户端 AbilityTask_WaitTargetData:
  TargetActor->ConfirmTargetByActor(...)
  → ServerSetReplicatedTargetData(handle, 原始键, TargetData, ApplicationTag, 当前键)   [可入批]
服务器:
  FScopedPredictionWindow(当前键)
  AbilityTargetDataMap.FindOrAdd({Handle, 原始键})->TargetData = 客户端数据    [原样存储]
  TargetSetDelegate.Broadcast(数据)                                            [能力逻辑自行消费]
客户端（预测路径）:
  CallServerSetReplicatedTargetData 后本地 OnTargetDataReady 立即回调（本地先跑）
```
- **验证强度（一手）**：`ServerSetReplicatedTargetData_Validate` 只做 `ensure(TgtData.IsValid())` 指针检查（AbilitySystemComponent_Abilities.cpp:4033-4045）；服务器不重算命中、不校验范围——**反作弊边界完全取决于能力逻辑怎么用这份目标数据**。
- 缓存消费：`CallReplicatedTargetDataDelegatesIfSet`（4090 一带）与 `ConsumeClientReplicatedData`（服务器激活前清残档，2096）。
- `AbilityTask_WaitTargetData` 的按键确认/取消走 EAbilityGenericReplicatedEvent（confirm/cancel 两个事件位，ServerSetReplicatedEvent，3934-3947）。

## 10.3 典型任务网络语义表

| 任务 | 客户端 | 服务器 | 挂起载体 |
|---|---|---|---|
| WaitTargetData | 产生+预测回调+上行 | 存缓存+广播 | ASC::AbilityTargetDataMap |
| PlayMontageAndWait | 本地播+记录 | RepAnimMontage 属性复制+位同步 | FGameplayAbilityLocalAnimMontage（本地）+ RepAnimMontageInfo（复制） |
| WaitGameplayEvent/Tag/Attribute | 各端独立监听（权威为准） | 同 | ASC 的 tag/attribute 委托注册表 |
| WaitDelay | 本地 timer | 本地 timer（两端各自计时，误差容忍） | FTimerHandle |
| NetworkSyncPoint | 显式 OnlyServerWait/OnlyClientWait/BothWait 同步点 | 同 | EAbilityGenericReplicatedEvent |

## 10.4 任意帧快照的覆盖性裁决

「任意帧一致性快照能不能覆盖 Ability」——**裸 GAS：不能**。证据：
1. 挂起状态分散在 UObject 成员（Task/Ability 实例）、ASC 的 `TMap<FGameplayAbilitySpecHandleAndPredictionKey, TSharedRef<FAbilityReplicatedDataCache>>`（AbilityTargetDataMap，4012 一带 FindOrAdd）、`FGameplayTagCountContainer` 的 TMap、`FPredictionKeyDelegates` 的静态 TMap（GameplayPrediction.h:440-451）。
2. 多数为 TSharedPtr/TWeakPtr/委托绑定，无序列化路径；`FGameplayAbilitySpec::GameplayEventData` 甚至是无 UPROPERTY 的 TSharedPtr（GameplayAbilitySpec.h:234）。
3. 唯一系统性可恢复的是状态型 FastArray（AGE/Spec/Cue）+ 属性；事件型与挂起型全部依赖两端进程内状态。
→ 目标引擎要实现「任意帧快照」，必须把这三张缓存表换成可序列化的纯数据结构（键 = (handle, key) 的值对，值 = 目标数据值），这正是 Schema 封闭线协议能做到而 UObject 世界做不到的事。

## 10.5 意外发现

1. AbilityTask 全局计数与按类统计本身是**泄漏检测器**（PrintCounts 命令 + 超限自动 dump，AbilityTask.cpp:62-110）——目标引擎可以直接抄这个"资源上限+自动归因"模式。
2. `AbilityTask_Repeat` 的 TODO：TimeBetweenActions/TotalActionCount 从不校验（AbilityTask_Repeat.cpp:32）；`AbilityTask_MoveToLocation` 自评"an awful way to do this"（AbilityTask_MoveToLocation.cpp:53）。
3. `AbilityTask_StartAbilityState`（5.x 新增）：命名状态段 + 结束/中断时的定向清理——Epic 在往「显式状态」方向补课，与目标引擎的冻结状态机同路。
