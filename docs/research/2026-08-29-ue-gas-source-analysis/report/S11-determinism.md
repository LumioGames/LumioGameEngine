# S11 · 确定性、求值顺序与状态哈希可行性

> 结论先行
> 1. **裸 GAS 不能产出稳定的状态哈希**。四类拦路设计：① 影响数值的迭代顺序依赖容器插入/删除历史（AGE 容器 RemoveAtSwap、聚合器 mod 数组 RemoveAllSwap、Override 首位取胜）；② 时间源全部是 wall-clock world time（FTimerManager + GameState 服务器时间），非固定步长；③ 进程级全局单例状态（预测键计数器、SetCurrentAppliedGE、FPredictionKeyDelegates、FScopedAggregatorOnDirtyBatch 全局集）不进任何快照；④ 浮点求和按数组序，无确定性归约。
> 2. 同帧多 Effect 的定序规则 = **容器数组序**（线性扫描决定堆叠命中与查询顺序，GameplayEffect.cpp:3384-3394/3687-3695），而数组序又被 RemoveAtSwap（4902）与 pending 链表并入顺序（DeletePendingActiveGameplayEffect，3029）塑造——即**到达顺序决定结果**。
> 3. 可序列化性逐类裁决：AGE 容器/Spec 容器/属性/预测键表 = 可（FastArray/属性）；任务挂起态/事件缓存表/委托注册 = **不可**；`FGameplayEffectContext`（TSharedPtr 多态）= 部分（Handle 形态可，内含 WeakObjectPtr 依赖对象映射）。

## 11.1 容器与迭代顺序证据汇总

| 容器 | 类型 | 顺序 | 坐标 |
|---|---|---|---|
| AGE 列表 | `TArray<FActiveGameplayEffect>`（FastArray 项） | 插入序；**RemoveAtSwap 破坏**；scope-lock 期新项走 PendingNext 链后并入 | GameplayEffect.h:1651 区声明；4902（RemoveAtSwap）；4314-4354（pending 链） |
| 聚合器通道 | `TMap<EGameplayModEvaluationChannel, Channel>` | 每次插入 KeySort 保升序——**确定性** | GameplayEffectAggregator.cpp:231-243 |
| 通道内 mod | `TArray<FAggregatorMod> Mods[Op::Max]` | 插入序；**RemoveAllSwap 破坏**（Override 赢家会换人） | GameplayEffectAggregator.h:174-178；.cpp:135-162 |
| Tag 计数 | `TMap<FGameplayTag, FGameplayTagCountItem>` | TMap 序（哈希序）——**不稳定** | GameplayEffectTypes.h:1101 |
| 拥有 tag 容器 | `TArray<FGameplayTag>` + 父展开 | 数组序 | GameplayTagContainer.h |
| 能力列表 | FastArray TArray<FGameplayAbilitySpec> | 插入序 + RemoveAtSwap（同族） | GameplayAbilitySpec.h:311-312 |

浮点：SumMods/MultiplyMods 直接 `+=`/`*=` 按数组序（GameplayEffectAggregator.cpp:216-229/12-25）；时长运算 `Duration - (WorldTime - StartWorldTime)`（5545-5548）无定点化。

## 11.2 时间驱动的可复现性障碍（逐坐标）

1. 时长/周期全走 `World->GetTimerManager()`（4481-4508）——帧率相关的调度、无漂移补偿（S3.8）。
2. 世界时间 `World->GetTimeSeconds()`；服务器钟 `GameState->GetServerWorldTimeSeconds()`（5351-5367）——墙钟，回放/对账需 RecordClientTimestamp 级别的重建。
3. 客户端时钟重定位 `StartWorldTime = WorldTime − (ServerWorldTime − StartServerWorldTime)`（2952-2955）吸收一次偏移，无持续校准。
4. 到期判定的浮点比较 `FMath::IsNearlyZero(..., KINDA_SMALL_NUMBER)`（5407）——容差随实现漂移。
5. `AbilityLastActivatedTime = LocalWorld->GetTimeSeconds()`（AbilitySystemComponent_Abilities.cpp:1986-1988）。

## 11.3 状态哈希裁决（反面教材清单）

若强行对 GAS 状态做哈希，必须先消除（每条给坐标）：
1. 容器顺序敏感（上表）——需要稳定排序键（如 handle 值）后再哈希；但 handle 是进程级计数器（ActiveGameplayEffectHandle.cpp:10-20），重放两次即不同——**需要换成内容寻址 ID**。
2. 预测键计数器（GameplayPrediction.cpp:189-197/243-249）与 delegate 注册表（GameplayPrediction.h:440-451）——未决预测状态进快照才能对账。
3. 全局"正在应用的 GE"指针（GameplayEffect.cpp:4233/4376）与全局聚合器脏集（GameplayEffectAggregator.h:408-412）——重放入口必须一致。
4. `NetUpdateID`/`GlobalFromNetworkUpdate`（3463-3498）——网络批次边界改变重算路径。
5. float 运算序（11.1）——需要确定性求值规范（排序后求和或定点）。
6. FName/FProperty 指针身份（AttributeSet.h:158-163 的 TFieldPath、FGameplayAttribute 哈希 131-135）——换内容 ID。

## 11.4 可序列化性逐类裁决

| 状态类 | 可/不可/部分 | 依据 |
|---|---|---|
| 属性（Base/Current） | **可** | UPROPERTY float（AttributeSet.h:48-53） |
| Active GE 全量 | **可** | FastArray 复制序列化已是证明（S7） |
| 已授予能力 spec | **可**（ReplicatedInstances 子对象除外依赖通道） | GameplayAbilitySpec.h:300-338 |
| Tag 计数 | **可** | 复制属性（新路径） |
| 预测键 ack 表 | **可** | FastArray（32 槽） |
| 挂起任务/目标数据缓存/事件缓存 | **不可** | TMap<TSharedPtr>/委托，无序列化（S10.4） |
| 未决预测副作用 | **不可**（进程内 delegate） | GameplayPrediction.h:440-451 |
| GameplayCue 持续态 | **可**（FastArray）；Actor 池**不可** | S9 |
| Montage 局部态 | 部分（本地结构无复制） | FGameplayAbilityLocalAnimMontage |

## 11.5 迁移含义

目标引擎「可复现求值顺序 + 可计算状态哈希」在 GAS 侧的对应改造成本是**有界的**：求值核心（通道公式）已是纯函数；主要工作是把三处顺序敏感容器换成显式定序（排序键）、把时间源换成 tick 计数、把进程级计数器换成内容 ID。GAS 没做这些不是因为技术上做不到，而是它的收敛模型（权威覆盖）容忍顺序抖动——**确定性是目标引擎的约束倒逼，不是 GAS 的设计目标**。这个反面教材的价值：它精确标出了"哪几类状态必须进快照、哪几类可以丢弃重建"（11.4 的表就是快照边界的清单）。
