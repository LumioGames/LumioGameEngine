# T4 · 属性复制内核：RepLayout / changelist / shadow state / push model

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **T4.1 裁决：影子状态与逐属性比较是「每对象（每 NetDriver）一份、全连接共享」，不是每连接一份。** 影子缓冲挂在 `FRepChangelistState::StaticBuffer`（Engine/Source/Runtime/Engine/Public/Net/RepLayout.h:470-471），由 `FReplicationChangelistMgr` 持有；该 Mgr 存放在 **UNetDriver 的 `ReplicationChangeListMap`**（Engine/Source/Runtime/Engine/Private/NetDriver.cpp:7917-7926 · `UNetDriver::GetReplicationChangeListMgr`），一个对象一个实例。每连接每对象只有 `FSendingRepState`——一组指向共享环形历史的游标（RepLayout.h:567-629），不是影子副本。比较工作每帧每对象最多做一次（RepLayout.cpp:1285-1318 的复用门 + DataReplication.cpp:1970 注释 "this will re-use work done by previous connections"）。预研的「每连接影子状态」说法对本版本不成立——那是对 UE4 早期实现（或对客户端接收侧 `FReceivingRepState::StaticBuffer`，RepLayout.h:538-551，它确实是每连接一份）的记忆错位。
2. **成本模型的正确形状：比较 O(A_changed)（每对象每帧一次，与连接数无关），序列化 O(Σ R_c)（每连接独立写流）**。`GShareShadowState`（默认开）+ `GShareInitialCompareState` 让第二个及以后的连接直接跳过比较（STAT_NetSkippedDynamicProps，RepLayout.cpp:1301/1315）；每连接仍需一次「role-only」比较（RemoteRole 是按连接语义的属性，RepLayout.cpp:1294-1299、1311 注释）。这就是 T5 成本公式里「diff 不随连接数放大」的实现根据。
3. **Push Model 是完整实现但默认关闭**：`Net.IsPushModelEnabled` 默认 **false**（Engine/Source/Runtime/Net/Core/Private/Net/Core/PushModel/PushModel.cpp:434-439），`Net.MakeBpPropertiesPushModel` 默认 true（:441-446，蓝图属性默认按 push-model 标记）。脏标记是 per-NetDriver per-object 的位图（`FPushModelPerNetDriverState`），由 `MARK_PROPERTY_DIRTY*` 宏族在游戏代码写入点打（PushModel.h:95 起），比较阶段据此跳过未脏属性的逐字节比较（RepLayout.cpp:1803-1823）。默认关的原因在宏的使用纪律：**漏打一次标记 = 该属性永远不再被轮询发现**（需 `bForceCompare` 或 `Net.IsPushModelValidateProperties` 类校验兜底），这是 Epic 用默认值表达的态度。

## 待证清单裁决表

| # | 预研说法 | 裁决 | 依据（坐标） |
|---|---|---|---|
| 4.1 | 服务器为每个连接维护影子状态、逐属性比较 | **证伪（对 5.8）**：影子+changelist 每对象一份共享；每连接只有游标 + role-only 复查 | RepLayout.h:433-500（FRepChangelistState/FReplicationChangelistMgr 及注释 "used by all connections, to share the compare work"）；NetDriver.cpp:7917-7926；RepLayout.cpp:1275-1335 |
| 4.2 | poll-and-diff 的 CPU 成本公式 | **修正后证实**：真实构成 = 每帧每已注册对象一次 O(属性数) 比较（可被频率门与 push-model 位图短路）+ 每连接每相关对象一次序列化。无「每连接 × 全属性」项 | NetDriver.cpp:5315-5445（频率门）；RepLayout.cpp:1285-1318（比较复用）；DataReplication.cpp:2007-2016（比较与序列化分离） |
| 4.3 | Push Model 官方文档说明不足，只做保守结论 | **补齐为 Verified-Src**（见结论 3；开启 = CVar + 每属性 bIsPushBased 标记 + 写入点打标） | PushModel.cpp:434-446；RepLayout.cpp:6088、6341、6414（bIsPushBased 路径）；PushModel.h:95-217（宏用法文档） |
| 4.4 | ELifetimeCondition 全枚举与语义 | **补齐**：18 项全表见下文，求值点在比较期（condition map 由 RepFlags 构建） | CoreUObject/Public/UObject/CoreNetTypes.h:16-36；RepLayout.h:587-588（BuildConditionMapFromRepFlags） |
| 4.5 | RepNotify 触发时机 | **证实并钉死**：客户端在**整个 bunch 的属性全部写入后**、`PostNetReceive` 之后逐属性触发；每 bunch 一次批处理 | DataReplication.cpp:1592-1610 · `FObjectReplicator::PostReceivedBunch`；:2431-2453 · `CallRepNotifies` |

## 机制正文

### 4.1 分层与所有权（谁持有谁）

```
UNetDriver (每世界一个 GameNetDriver)
 ├─ RepLayoutMap: TMap<UClass*, TSharedPtr<FRepLayout>>        # 类级布局缓存
 │    └─ NetDriver.cpp:7878 · RepLayoutMap.Add(Class, CreateFromClass(...))
 ├─ ReplicationChangeListMap: TMap<UObject*, TSharedPtr<FReplicationChangelistMgr>>
 │    └─ NetDriver.cpp:7917-7926 · GetReplicationChangeListMgr（惰性创建）
 │         └─ FRepChangelistState
 │              ├─ ChangeHistory[64]（环形 changelist，MAX_CHANGE_HISTORY=64，RepLayout.h:450-453）
 │              ├─ StaticBuffer ←——「shadow state」本体（RepLayout.h:470-471 注释：仅服务器且启用 shadow 时使用）
 │              ├─ SharedSerialization（FRepSerializationSharedInfo，RepLayout.h:473-474）
 │              └─ PushModelObjectHandle（WITH_PUSH_MODEL 时，RepLayout.h:478-491）
 └─ ClientConnections[]
      └─ UNetConnection
           └─ UActorChannel（每连接每对象一个）
                └─ FObjectReplicator（DataReplication.h:73）
                     ├─ RepState: TUniquePtr<FRepState>（每连接）      # :334
                     │    ├─ FSendingRepState（游标：HistoryStart/End、LastChangelistIndex、LastCompareIndex、NumNaks，RepLayout.h:567-629）
                     │    └─ [客户端] FReceivingRepState（StaticBuffer + RepNotifies 列表，RepLayout.h:538-563）
                     ├─ CheckpointRepState（回放检查点专用，DataReplication.h:335）
                     └─ ChangelistMgr: TSharedPtr（共享句柄，DataReplication.cpp:126-128、724-728）
```

**内存成本**：影子状态 = 每复制对象一份全属性深拷贝（O(属性字节数)，动态数组指针存容器）；每连接追加的是 O(1) 游标 + NAK 记录。两者差一个数量级——这正是 4.1 勘误的成本含义。

### 4.2 FRepLayout：反射 → 扁平命令表

- 构建：`FRepLayout::CreateFromClass` → `InitFromClass`（RepLayout.cpp:6071-6104+）读 `UClass::ClassReps`（`SetUpRuntimeReplicationData()`，:6101），把继承链上全部 `Replicated` 展开为 `FRepParentCmd`（属性级）× `FRepLayoutCmd`（字段级量化命令）两张扁平数组（Engine/Source/Runtime/Engine/Public/Net/RepLayout.h:780-856）。**每类一份，按 NetDriver 缓存**（NetDriver.cpp:7878）。
- 命令表同时携带量化信息（内联类型直接写位流、NetSerialize 属性挂 `NetSerializeLayouts`、CustomDelta（FastArray 类）单独编组 :6098 `TempNetSerializeLayouts`）。
- 这就是「C++ 反射驱动复制」的成本核心：一次性把 UClass 压成两张数组，运行期比较/序列化只走数组下标，不再碰反射 API。

### 4.3 一次属性复制（FObjectReplicator::ReplicateActor 的属性段）

DataReplication.cpp:1940-2090 主干：

```
ReplicateActor(通道, Bunch):
  # 1. 更新共享 changelist（复用本帧早前连接的工作）
  UpdateResult = FNetSerializeCB::UpdateChangelistMgr(RepLayout, SendingRepState, *ChangelistMgr,
                        Object, Driver->ReplicationFrame, RepFlags, bForceCompare)   # :2007
      → FRepLayout::UpdateChangelistMgr（RepLayout.cpp:1275-1335）
          if 共享门命中(LastReplicationFrame == ReplicationFrame && GShareShadowState ...):
              仅 role-only 比较（bNetInitial 或该连接首次）→ return          # :1285-1303
          else: CompareProperties(...)                                        # :1320
  # 2. 序列化：把「该连接尚未发过的 changelist 增量」写入 Writer
  RepLayout->ReplicateProperties(SendingRepState, ChangelistState, ObjectData, ...)   # :2016
  # 3. CustomDelta（FastArray 等）单独走 NetDeltaSerialize
  ReplicateCustomDeltaProperties(Writer, RepFlags, ...)                        # :2033
  # 4. ResendAllData（回放检查点 SinceOpen/SinceCheckpoint）旁路常规状态更新   # :2073-2087
```

`CompareProperties`（RepLayout.cpp:1777-1881+）本体：

- `RepChangelistState->CompareIndex++`（:1791）→ 取环形历史下一格（:1794-1796）；
- `StackParams.ShadowData = RepChangelistState->StaticBuffer.GetData()`（:1834）——**比较 = 对象数据 vs 共享影子**；
- 递归比较 `CompareParentProperties → CompareProperties_r / CompareProperties_Array_r`（:1369-1380 声明族；数组首元素记长度、句柄递增的内联表示见 RepLayout.h:321-331 FRepChangedHistory 注释）；
- 条件过滤：condition map 由 `FReplicationFlags`（bNetInitial/bNetOwner/bClientReplay/bReplay…）经 `UE::Net::BuildConditionMapFromRepFlags` 构建（RepLayout.h:587-588），COND_* 求值发生在比较期——不满足条件的属性既不比较也不序列化；
- 变化的属性句柄（uint16 RelativeHandle）append 到 `NewHistoryItem.Changed`（:1800、1365）；影子随后被写回新值。

**ELifetimeCondition 全表**（Engine/Source/Runtime/CoreUObject/Public/UObject/CoreNetTypes.h:16-36，逐项语义照抄注释）：

| 值 | 名 | 语义 |
|---|---|---|
| 0 | COND_None | 无条件，变化即发 |
| 1 | COND_InitialOnly | 仅初始 bunch |
| 2 | COND_OwnerOnly | 仅发 actor 属主 |
| 3 | COND_SkipOwner | 除属主外都发 |
| 4 | COND_SimulatedOnly | 仅 simulated proxy |
| 5 | COND_AutonomousOnly | 仅 autonomous proxy |
| 6 | COND_SimulatedOrPhysics | simulated 或 bRepPhysics |
| 7 | COND_InitialOrOwner | 初始或属主 |
| 8 | COND_Custom | 运行时经 SetCustomIsActiveOverride 开关 |
| 9 | COND_ReplayOrOwner | 回放连接或属主 |
| 10 | COND_ReplayOnly | 仅回放连接 |
| 11 | COND_SimulatedOnlyNoReplay | simulated 但非回放 |
| 12 | COND_SimulatedOrPhysicsNoReplay | 同 6 但非回放 |
| 13 | COND_SkipReplay | 不发回放 |
| 14 | COND_Dynamic（Hidden） | 运行时可改条件，默认总是复制 |
| 15 | COND_Never（Hidden） | 永不 |
| 16 | COND_NetGroup（Hidden） | 子对象按 group 归属连接，不可用于属性 |
| 17 | COND_Max | 哨兵 |

**「字段只发所有者」能不能做到？** 能：`COND_OwnerOnly`/`COND_SkipOwner`（+ bOnlyRelevantToOwner 的通道级配合，见 T6）。粒度是「属性 × 连接角色」，不是加密意义上的隐藏——该属性对其他连接的网络坐标仍存在于类布局中（布局不按连接裁剪，只是不序列化）。

### 4.4 RepNotify 的确切时机（4.5 裁决展开）

客户端收到 bunch → `UActorChannel::ProcessBunch` → `FObjectReplicator::ReceivedBunch`（属性写入 `FReceivingRepState::StaticBuffer`）→ **`PostReceivedBunch`（DataReplication.cpp:1592-1610）：先 `PostNetReceive()`（:1604），再 `CallRepNotifies(true)`（:1609）**。即：**同一 bunch 内全部属性先落盘，再按收集到的 RepNotify 列表逐个回调**（列表在接收时累积于 `FReceivingRepState::RepNotifies`，RepLayout.h:556-557）。`REPNOTIFY_Always`（CoreNetTypes.h:39-43）可强制「值未变也回调」。注意 `bSkipIfChannelHasQueuedBunches` 参数（:2431）：通道还有排队 bunch 时可延迟回调——保证同一帧多个 bunch 的通知次序可控。

### 4.5 Push Model 细节（4.3 裁决展开）

- 开关：`Net.IsPushModelEnabled`（注意前缀是 `Net.` 不是 `net.`），默认 **false**；`Net.MakeBpPropertiesPushModel` 默认 true（PushModel.cpp:434-446）。
- 编译期：`WITH_PUSH_MODEL`（模块级开关）；属性级 `bIsPushBased` 来自 `UPROPERTY(... bIsPushBased=true ...)`——蓝图的复制属性由上述 CVar 默认标为 push-based。
- 运行期：脏位图是 **per-NetDriver per-object**（`FPushModelPerNetDriverHandle` 挂在 FRepChangelistState，RepLayout.h:478-491；per-NetDriver 意味着多驱动不共享脏信息）。写入点必须手工调用 `MARK_PROPERTY_DIRTY` / `MARK_PROPERTY_DIRTY_FROM_NAME` 宏族（PushModel.h:95-217 带完整用法示例）。
- 比较期短路：`PushModelState`/`PushModelProperties` 进入 FComparePropertiesSharedParams（RepLayout.cpp:1348-1349、1803-1823）；未脏属性跳过逐字节比较。校验兜底 `GbPushModelValidateProperties`（:1824）。
- FastArray 的脏由 `MarkItemDirty` 驱动（分配 ReplicationID/ReplicationKey，FastArraySerializer.h:202-223），不走属性位图。
- **为什么不是默认**（推断，依据上述结构）：宏的正确性依赖全代码路径无遗漏打标，漏标的症状是「属性永远停在旧值」且无报错；Epic 保留轮询默认值以换取安全性，同时用蓝图默认 push-based 扩大覆盖。迁移成本 = 逐属性审计写入点。

### 4.6 FastArray（NetDeltaSerialize 的增量数组）

- 定义：Engine/Source/Runtime/Net/Core/Classes/Net/Serialization/FastArraySerializer.h（NetCore 模块，不在 Engine）。头注释（:50-55、202-223）直陈设计代价：**需要游戏代码显式 MarkItemDirty**；**「list 的顺序在客户端与服务器之间不保证一致」**；增删改以 ReplicationID（int32，按变更顺序分配）寻址，客户端按 ID→本地索引的映射应用。
- 服务器端序列化：结构级 `NetDeltaSerialize`（STRUCT_NetDeltaSerializeNative 判定，DataReplication.h:29-38），走 `FObjectReplicator::ReplicateCustomDeltaProperties`（DataReplication.cpp:2033）。
- 接收端回调时机：与 RepNotify 同批——`PostReceivedBunch → CallRepNotifies` 路径中，FastArray 的客户端回调（PostReplicatedAdd/Remove/Change）受 `bCallFastArrayClientCallbacks` 门控（DataReplication.cpp:1575-1581，tick 与回调被禁用期间不触发——**加载/关卡切换期静默吞事件**是已知的可观测行为）。
- ID 空间与溢出：ID/Key 均为 int32 顺序分配（FastArraySerializer.h:223）；环绕与超长数组的处理[未在源码中核实到专门的上限常量：搜索 MaxRepArraySize 于 RepLayout.cpp 0 命中；net.MaxRepArraySize 的存在性见 T16 表]。乱序/丢包：增量走可靠通道（DataChannel 层保证），未收齐时挂 unmapped/排队路径。

### 4.7 子对象与 ReplicateSubobjects 的手工负担

`UActorChannel::ReplicateActor` 末段显式调 `Actor->ReplicateSubobjects(this, &Bunch, &OutRepFlags)`（DataChannel.cpp:4007）并遍历 replicated components 的 ReplicateSubobjects（:4224）。**默认实现只覆盖「注册过的组件」**；组件之外的子对象（纯 UObject）必须由项目在每个 actor 的 ReplicateSubobjects 重载里手工 `CreateSubObjectChannel`/序列化——漏掉即不复制，无警告。这是引擎把「对象图遍历」外包给游戏代码的位置（对照：目标引擎的 ECS 天然有实体清单，这一整层负担不存在）。

### 4.8 迁移裁决：Schema 生成 + 字段集封闭 + 规范化字节 能否达到同等表达力？

**能覆盖的**（UE 模型的实质）：扁平字段命令表（= Schema 生成的字段描述符）、共享 changelist + 每连接游标（T4.1 的正确成本模型）、按字段条件掩码（= ELifetimeCondition 的可枚举子集：none/owner-only/initial-only/skip-owner 足以覆盖绝大多数用例）、量化编码（UE 用 FRepLayoutCmd 的量化命令，目标引擎用 Schema 的规范化编码）、增量数组（FastArray 的 ID 寻址 ≈ 字段集封闭下的数组 diff）。

**会丢的（逐条）**：

1. **运行时可变的条件语义**：COND_Custom（SetCustomIsActiveOverride）与 COND_Dynamic 允许运行时改条件（CoreNetTypes.h:26、32）——字段集封闭模型里条件必须静态进 Schema，或退化成两类字段。
2. **按属性挂接的任意序列化钩子**：NetSerialize/NetDeltaSerialize 是「每属性一个 C++ 函数指针」（RepLayout.cpp:6098 TempNetSerializeLayouts）——Schema 模型需把等价物表达为「每字段的编解码器 ID」，规范化字节要求会禁止 per-project 自由函数（除非编解码器也在封闭集合内）。
3. **反射驱动的动态类**：UE 的布局由 UClass 运行期反射生成（含蓝图动态类）；Schema 先行 = 布局编译期冻结。UE 靠这个支持「同一个类在不同项目模块里扩展复制字段」，目标引擎明确放弃此项（换来哈希可计算）。
4. **对象图内的隐式子对象**：UE 复制的是 UObject 图（PackageMap 把对象引用编成 NetGUID）；ECS 模型只复制组件值。凡 UE 项目把「复制单位」设为子对象的（如武器实例），迁移时必须改成实体或内联结构。
5. **shadow state 的语义松散**：UE 影子是「上次比较时的浅拷贝 + 容器指针」（RepLayout.h:366-374 注释明言动态内存仍在堆上），比较语义含指针比较——规范化字节模型下「同状态逐字节相同」比 UE 的「逐属性 equals」更强，可直接用于哈希，这是目标引擎的优势而非损失。

## 意外发现

1. RepLayout.h:594-599 注释：`SavedRemoteRole/SavedRole` 按连接缓存是为绕开 FScopedRoleDowngrade 引发的 UE-66313 族 bug——Role/RemoteRole 这两个「每连接语义」的属性是共享 changelist 架构里的特例税。
2. `GShareInitialCompareState` 两条互斥复用路径（RepLayout.cpp:1285 vs 1305）注释里写明了历史演进：旧规则要求每连接至少一次全属性比较（:1310-1311），新规则用 bRolesOnly 精确复查——「共享比较」是逐步收紧的优化，不是天生设计。
3. DataReplication.cpp:2052-2068：CustomDelta 的 change-index 对「跳过条件属性」的连接也要前进，否则 NAK 后无法跳过——共享状态与每连接状态的边界情况有专门注释，说明此处出过真实 bug。
4. FRepChangedHistory::OutPacketIdRange + Resend（RepLayout.h:356-363）：changelist 记住自己被装进哪些包，NAK 时精确重发——**增量复制的可靠性单位是 changelist 条目，不是包**。

## 对目标环境的迁移含义

目标引擎应原样继承「**共享 changelog + 每连接游标**」这一层（它同时解决了 diff 成本与断线续传的增量定位），把 UE 的三处历史包袱换掉：(1) 影子浅拷贝 → 提交点后的规范化状态（比较退化为哈希比对，成本 O(1) 而非 O(属性数)）；(2) 条件枚举 → Schema 的静态可见性档位（放弃运行时可变条件，换取两端一致）；(3) 子对象手工遍历 → ECS 实体清单天然穷举。Push Model 的教训必须吸收：**脏标记作为优化必须是「可回退到重算」的旁路而非唯一事实源**——目标引擎的 fail-stop + 整帧重算模型天然免疫漏标问题（代价是 CPU，不是正确性）。
