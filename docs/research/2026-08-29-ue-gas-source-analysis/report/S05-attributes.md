# S5 · Attribute 与 Aggregator

> 结论先行
> 1. `FGameplayAttributeData = { float BaseValue; float CurrentValue; }` 两个 BlueprintReadOnly UPROPERTY（AttributeSet.h:48-53）——**两个都随 AttributeSet 子对象复制**；客户端的"重建 Current"不是从头算，而是 OnRep/网络更新时把复制来的 Base 过一遍本地聚合器（详见下）。
> 2. 重算触发源只有一族：**聚合器变脏**（`FAggregator::BroadcastOnDirty` → `OnAttributeAggregatorDirty`）；脏的来源 = SetBaseValue/AddAggregatorMod/RemoveAggregatorMod/UpdateAggregatorMod 四个入口 + 全局批量器 `FScopedAggregatorOnDirtyBatch` 把作用域内的全部脏**延迟到作用域末尾**统一广播。
> 3. 钩子的分工（源码注释即答案，AttributeSet.h:196-234）：**Pre/PostGameplayEffectExecute 只在 execute（instant/periodic 改 base）时调用**；**PreAttributeChange 在任何修改前**（含 duration 效果、移除、免疫、堆叠——钳制写这里）；**PreAttributeBaseChange 只管 base 修改**。Clamp 写错位置的漏点：只写 PreGameplayEffectExecute 会漏掉 duration mod 引发的 Current 变化；只写 PreAttributeChange 会在 Pre 里做"触发额外逻辑"（注释明令禁止，聚合器可能因无数原因变化）。

## 5.1 FGameplayAttributeData 与复制标记

- 字段：`BaseValue`（"only includes permanent changes"）+ `CurrentValue`（"includes temporary buffs"）双 float（AttributeSet.h:48-53）；虚函数 SetCurrentValue/SetBaseValue 供子类（如 FGameplayEffectAttributeCaptureSpec 的快照）拦截。
- **复制方式**：AttributeSet 作为 **ASC 的复制子对象**走 ActorChannel（ASC::ReplicateSubobjects，AbilitySystemComponent.cpp:1927-1957：SpawnedAttributes + ReplicatedInstancedAbilities 逐个 ReplicateSubobject；ReadyForReplication 里 AddReplicatedSubObject，330-353；能力实例的 LifetimeCondition = bReplicateAbilitiesToSimulatedProxies ? COND_None : COND_ReplayOrOwner）。属性变化走属性复制的常规路径；**GAS 要求游戏侧用 `DOREPLIFETIME_CONDITION_NOTIFY(..., REPNOTIFY_Always)`**（GameplayPrediction.h:126-138 的示例代码）。
- 客户端收到属性 OnRep 后调 `GAMEPLAYATTRIBUTE_REPNOTIFY` 宏（AttributeSet.h:402-417 一带）：把 OnRep 的旧值塞回聚合器 base、重算 Current（SetBaseAttributeValueFromReplication，GameplayEffect.cpp:3743-3772，S8/S7 已详述）。

## 5.2 重算触发源穷举（OnAttributeAggregatorDirty 的上游）

坐标：GameplayEffect.cpp:3452-3511（OnAttributeAggregatorDirty）；GameplayEffectAggregator.cpp:585-651（BroadcastOnDirty）

| 触发源 | 坐标 | 立即/延迟 |
|---|---|---|
| SetBaseValue(Broadcast=true) | GameplayEffectAggregator.cpp:438-445 | 立即（除非在批量器内） |
| AddAggregatorMod / RemoveAggregatorMod / UpdateAggregatorMod | 同文件 487-521 | 同上 |
| ExecModOnBaseValue（instant 执行） | 同文件 481-485 | 同上 |
| **FScopedAggregatorOnDirtyBatch 作用域**（应用 GE、抑制切换、网络接收都是） | GameplayEffectAggregator.cpp:585-594（入 DirtyAggregators 集合）+ 668-718（作用域末尾统一广播） | 延迟到作用域末尾 |
| 网络更新（属性 OnRep 或 AGE 容器更新） | OnAttributeAggregatorDirty:3463-3501：`GlobalFromNetworkUpdate && Aggregator->NetUpdateID != 批次号` → ReverseEvaluate 反推 base（仅 legacy float 属性）→ **每个网络批次只反推一次**（NetUpdateID 去重，注释 3468-3485 详述两条到达路径为何会重复） | 网络批次末尾 |
| 循环依赖 | BroadcastOnDirty:609-626：MAX_BROADCAST_DIRTY=10 递归上限，超限只发 OnDirtyRecursive（更新 UProperty 不跑游戏回调）+ Warning"possible the resulting attribute values are not what you expect"；非 Shipping 下 TObjectIterator 全量扫描调试 | 立即（计数防护） |

客户端 IncludePredictiveMods=true（3500）——本地预测 mod 参与重算。

## 5.3 钩子调用点与顺序（穷举）

| 钩子 | 调用点 | 能钳什么 |
|---|---|---|
| PreGameplayEffectExecute | InternalExecuteMod:4112（apply 前，可 return false 丢弃整个 mod） | 本次 execute 的 ModData（可改 magnitude）；**只管 instant/periodic** |
| PostGameplayEffectExecute | InternalExecuteMod:4128（apply 后，游戏规则：掉血→死等） | 已改完，只能做反应 |
| PreAttributeChange | InternalUpdateNumericalAttribute 路径（经 SetAttributeBaseValue→SetBaseValue(broadcast)→OnDirty→InternalUpdateNumericalAttribute:3945-3984 广播前值已写入） | NewValue 引用可钳；**覆盖一切变化** |
| PostAttributeChange | 同上广播后 | 反应 |
| PreAttributeBaseChange / PostAttributeBaseChange | SetAttributeBaseValue（GameplayEffect.cpp:3986 起）内嵌 | 仅 base |
| OnAttributeAggregatorCreated | AttributeSet.h:237（声明） | 挂自定义 EvaluationMetaData |

属性变化通知链（InternalUpdateNumericalAttribute:3945-3984）：旧值取自 ASC → 写 UProperty → （非递归时）**先广播弃用的 AttributeChangeDelegates 再广播新的 AttributeValueChangeDelegates（带 Old/New/GEModData）**。RepNotify 与 delegate 的次序：RepNotify 由引擎属性复制触发在前，聚合器重算链在后（这就是为何需要 REPNOTIFY_Always + NetUpdateID 去重）。

## 5.4 AttributeSet 注册路径与宏

- 注册：ASC::AddSpawnedAttribute/GetOrCreateAttributeSubobject → SpawnedAttributes 数组（UPROPERTY Replicated+OnRep，AbilitySystemComponent.h:1931-1935）；InitStats 从 DataTable 初始化（"Not well supported"自述，AbilitySystemComponent.h:179）。
- `ATTRIBUTE_ACCESSORS`（AttributeSet.h:419-465 一带）展开为四件套：`GAMEPLAYATTRIBUTE_PROPERTY_GETTER`（静态 Get{X}Attribute()）、`GAMEPLAYATTRIBUTE_VALUE_GETTER`（Get{X}()）、`VALUE_SETTER`（Set{X}()——内部走 ASC 的 SetNumericAttributeBase 路径）、`VALUE_INITTER`（Init{X}()）。`GAMEPLAYATTRIBUTE_REPNOTIFY`（402）= 用旧值重算聚合器。
- IsNetAddressable/SetNetAddressable（239-240）：决定 AttributeSet 子对象能否按名字寻址（网络体积）。

## 5.5 意外发现

1. `FGameplayAttribute::GetTypeHash` 用**指针哈希**，旁边就是 Epic 自己的 FIXME："Use ObjectID or something to get a better, less collision prone hash"（AttributeSet.h:131-135）。
2. AttributeSet.h:226 @todo：计划**弃用裸 float 属性**只留 FGameplayAttributeData；GameplayEffectAggregator.h:105-108 注明 ReverseEvaluate"will be deprecated/removed soon with the transition to struct-based attributes"——**struct 属性化是既定方向**，BaseValue/CurrentValue 双字段本身即将升级为可复制结构体。
3. `InternalUpdateNumericalAttribute` 里 CurrentModcallbackData 与显式 ModData 并存（3956-3962 的 Warning）——跨函数传回调数据的两套机制并存。
4. `AttributeSet::RegisterReplicationFragments` 覆写（AttributeSet.h:261）——AttributeSet 已有 Iris 复制片段注册钩子。

## 5.6 迁移分析（ECS 化后聚合器还成立吗）

Aggregator 的前提是（坐标为证）：① 属性有**稳定地址**（FGameplayAttribute 是 FProperty 指针，AttributeSet.h:158-159）；② 能在属性旁挂 **delegate**（OnDirty，GameplayEffectAggregator.h:329-330）；③ **TSharedPtr 归属**（FAggregatorRef，378-388）允许快照与克隆；④ 全局批量器（397-413）依赖"作用域内聚合器不会被销毁"（注释自述存 raw 指针的风险）。
ECS 化（值语义、archetype 搬移、地址不稳定）后 ①③④ 全部失效。但**求值模型本身**（通道分桶 + 固定公式 + 全量重算）是纯函数，可以整体搬进系统：把 OnDirty 换成"改 mod 组件 → 标脏 attribute 索引 → 帧末统一重算批"。目标引擎的「唯一提交点」天然对应 FScopedAggregatorOnDirtyBatch 的形状——GAS 已经在用"作用域末尾批量重算"，只是作用域不是帧。代价：去掉 delegate 后，"依赖属性变化→重算依赖 GE"（OnMagnitudeDependencyChange，GameplayEffect.cpp:3513-3570）需要重建为显式依赖图遍历。
