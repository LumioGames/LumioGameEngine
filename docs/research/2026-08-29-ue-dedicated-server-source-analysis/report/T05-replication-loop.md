# T5 · 服务器复制主循环与带宽调度（ServerReplicateActors 全链）

> 本章所有行号相对 UE 5.8.2（git ff8421f2b，CL 55116800）。置信度：除特别标注外均为 Verified-Src。

## 结论先行

1. **ServerReplicateActors 是一个每帧一次的「公共候选集构建 + 每连接独立调度」的两段式循环**：候选集（ConsiderList）按对象频率门限过滤，每帧构建一次、所有连接共享；排序与发送按连接独立进行。`对象数 × 连接数` 的乘积项只出现在 relevancy 判定、优先级计算与序列化阶段，不出现在 diff 比较阶段（diff 的共享性见 T4.1 裁决）。
2. **「字节预算」不是一个显式的字节计数器，而是一个每连接 token bucket**：`UNetConnection::QueuedBits` 以 `CurrentNetSpeed × dt` 的速率回填、每发一个包按位数扣减，`IsNetReady()` 即 `QueuedBits + SendBuffer位数 ≤ 0`。预算耗尽时 `ServerReplicateActors_ProcessPrioritizedActorsRange` 直接 `return j`——剩余对象**不丢弃**，由 `MarkRelevantActors` 打 `bPendingNetUpdate` 留到下一帧。
3. **饥饿避免是「优先级 × 自上次发送间隔」的隐式机制，没有显式的 aging 通道**：优先级 = `65536 × NetPriority × Time`，其中 `Time` 对已有通道取「距上次实际发送的秒数」、对新对象取 `SpawnPrioritySeconds`；长时间发不出去的对象 Time 线性增长，自然在排序中上升。另有自适应频率（`OptimalNetUpdateDelta`）把长期无变化对象的考虑频率从 `NetUpdateFrequency`（默认 100Hz）滑向 `MinNetUpdateFrequency`（默认 2Hz）。

## 待证清单裁决表（本章相关条目）

| 预研说法 | 裁决 | 坐标 |
|---|---|---|
| 「复制是一个带字节预算的调度问题」（视角） | **证实**（Verified-Src：预算=token bucket，调度=priority sort + 截断回流） | Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5687-5894 · `UNetDriver::ServerReplicateActors_ProcessPrioritizedActorsRange`；Engine/Source/Runtime/Engine/Private/NetConnection.cpp:2731-2751 · `UNetConnection::IsNetReady` |
| 「ServerReplicateActors 怎么排序和截断停在 Reported」 | **补齐为 Verified-Src**（排序见 PrioritizeActors，截断见 ProcessPrioritizedActorsRange 两处 IsNetReady 门） | Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5528-5679, 5687-5894 |
| 「对象数 × 连接数的成本」 | **修正**：乘积项仅在 relevancy/优先级/序列化；候选集构建 O(A) 与 diff（changelist 共享，见 T4.1）不随连接数线性放大 | Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5315-5445（每帧一次）；5562-5656（每连接） |

## 机制正文

### 5.1 调用点：复制挂在 TickFlush 上

- `UNetDriver::TickFlush` 中，仅当 `IsServer() && (ClientConnections.Num() > 0 || bUpdateReplicationSystemWithNoConnections) && !bSkipServerReplicateActors` 才推进（Engine/Source/Runtime/Engine/Private/NetDriver.cpp:1186）。
- 关键分岔：**若设置了 Iris 的 `ReplicationSystem`，走 `InternalIrisUpdateTransactional`；否则走经典 `ServerReplicateActors`**（NetDriver.cpp:1212-1231）。两者互斥，同一 driver 只会跑一套。`bUpdateReplicationSystemWithNoConnections` 是 Iris 专属补丁（无连接也推进复制系统），经典路径在 0 连接时直接返回（NetDriver.cpp:6282-6285）。
- ReplicationGraph 则在 `ServerReplicateActors` 内部第一行被接管：`if (ReplicationDriver) return ReplicationDriver->ServerReplicateActors(DeltaSeconds);`（NetDriver.cpp:6295-6298）。经典实现与 ReplicationGraph 的关系是「驱动替换」而非「内部共存」。

### 5.2 主入口的骨架

Engine/Source/Runtime/Engine/Private/NetDriver.cpp:6277-6473 · `UNetDriver::ServerReplicateActors`（整函数 `#if WITH_SERVER_CODE` 包裹，6281）：

```
ServerReplicateActors(dt):
  ReplicationFrame++                          # 使本帧的 unchanged 缓存失效（6303）
  NumClientsToTick = PrepConnections(dt)      # 6313-6329；0 → return 0
  ServerTickTime = 1/GEngine->GetMaxTickRate  # 6341
  bCPUSaturated = dt > 1.2 × ServerTickTime   # 6349
  BuildConsiderList(ConsiderList, ServerTickTime)   # 6362-6373，每帧一次
  for i, Connection in ClientConnections:      # 6384
    if net.DormancyValidate==2: Connection->ExecuteOnAllDormantReplicators(ValidateAgainstState)  # 6390-6405
    if i >= NumClientsToTick:                  # 6408：本帧不 tick 的连接
        给「通道落后于对象 NetUpdateTime」的对象打 bPendingNetUpdate（6412-6428）
        Connection->TimeSensitive = false      # 6430
    elif Connection->ViewTarget:               # 6432：无 ViewTarget 的连接不复制
        ServerReplicateActors_ForConnection(Params)  # 6434-6456
    if Connection->GetPendingCloseDueToReplicationFailure(): ConnectionsToClose.Add  # 6459-6462
  OnPostConsiderListUpdateOverride 多播        # 6470-6473
```

### 5.3 阶段一：PrepConnections（谁有资格被服务）

Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5198-5301 · `UNetDriver::ServerReplicateActors_PrepConnections`：

- **连接节流只对 listen server（或 `-limitclientticks` 启动参数）生效**（5204）：按 `GEngine->NetClientTicksPerSecond` 决定本帧 tick 几个客户端；DS 默认全 tick。代码注释自带 FIXME：`DeltaTimeOverflow` 是 static，多 NetDriver 并存会互相踩（5207）——多世界场景的隐患实证，见 T1。
- `net.MaxConnectionsToTickPerServerFrame`（默认 0=不限）可硬性限制每帧 tick 的连接数（5194 注册，5225-5228 生效）。这是引擎内置唯一的「连接数预算」。
- 每连接健康门：`OwningActor != null && State == USOCK_Open && (Driver->GetElapsedTime() - LastReceiveTime < 1.5)`（5243）。**1.5 秒没收到对端任何数据的连接会被视为 not-ready：ViewTarget 置空、不参与本轮复制**（5290-5297），但此处不关连接（关连接走 T14 的超时路径）。这就是「半开连接的降级表现」：复制静默停止，直到超时路径把它关掉。
- 只有找到至少一个 ready 连接才返回非 0（5300）——一个 ready 连接能拉动整个 ConsiderList 构建。

### 5.4 阶段二：BuildConsiderList（每帧一次的对象侧过滤）

Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5303-5455 · `UNetDriver::ServerReplicateActors_BuildConsiderList`。遍历 `GetNetworkObjectList().GetActiveObjects()`（5315）——全部已注册复制对象，复杂度 O(A)：

- **频率门**：`!bPendingNetUpdate && TimeSeconds <= NextUpdateTime` → 跳过（5319-5322）。
- **剔除**：PendingKill、`RemoteRole == ROLE_None`、NetDriverName 不匹配（beacon 等）、未完成初始化、**所在 Level 正在流送可见性变更 / 关联中**（5362-5367，流送与复制耦合的直接证据，见 T12）、初始休眠对象（5369-5377，直接从网络对象列表移除）。
- **自适应更新频率**（5384-5430）：
  - 首次复制：`OptimalNetUpdateDelta = 1/NetUpdateFrequency`（5387-5389）。
  - 距上次实际复制超过 `ScaleDownStartTime=2.0s` 后，在 5s 窗口内把 `OptimalNetUpdateDelta` 从 `1/NetUpdateFrequency` 线性插值到 `1/MinNetUpdateFrequency`（5391-5409）。
  - `MinNetUpdateFrequency == 0` 时被钳为 2.0（5398-5401）。
  - `bUseAdapativeNetFrequency` 开关 = `IsAdaptiveNetUpdateFrequencyEnabled()`（5311，绑定 CVar `net.UseAdaptiveNetUpdateFrequency`，注册点见 T16 表）。
- **抖动**：`NextUpdateTime = TimeSeconds + RandDelay + NextUpdateDelta`，`RandDelay = Frand() × ServerTickTime`（除非 `net.DisableRandomNetUpdateDelay`？实际 CVar 名以 T16 表为准；变量 `GNetDisableRandomNetUpdateDelay` 注册于 NetDriver.cpp:566-569）（5420-5425）。抖动的目的：防止同类对象同帧齐发造成包尺寸尖峰。
- **PreReplication 钩子每帧每对象只调一次**（5441-5444，`bCallPreReplication` 受 `net.CallPreReplication` 控制）——对象侧的「仅可见性相关字段修剪」就挂在这里，成本与连接数无关。
- `bPendingNetUpdate` 在此清零（5433）、「假定所有连接都能考虑它」；若下游某连接饱和，会由 MarkRelevantActors / ForceNetUpdate 重新置位——这是贯穿全循环的**回流机制**。

### 5.5 阶段三：PrioritizeActors（每连接的相关集 + 排序）

Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5528-5679 · `UNetDriver::ServerReplicateActors_PrioritizeActors`：

1. `NetTag++`，把 `SentTemporaries`（上帧已发的临时 actor）打上当前 tag（5534-5540）→ 后续 `Actor->NetTag != NetTag` 判重（5638）。
2. 对 ConsiderList 每个对象：
   - 无通道者先过 **Level 初始化门**（客户端没加载该 Level 不发，5582-5586）再过 **relevancy 门**（`IsActorRelevantToConnection` → 逐 viewer 调 `AActor::IsNetRelevantFor`，5458-5469、5588）。注释（5576-5579）明说历史包袱：relevancy 过去因含 line trace 很贵、被推迟到 prioritization 之后；现在 relevancy 便宜了就提前过滤以缩小排序集。
   - `bOnlyRelevantToOwner`（如 PlayerController）走属主判定（5602），非属主且超时则关通道（5609-5611，`EChannelCloseReason::Relevancy`）。
   - **休眠门**：`IsActorDormant`（对象级 `DormantConnections` 集合，5499-5503）→ 直接跳过；`ShouldActorGoDormant` 满足则 `Channel->StartBecomingDormant()`（5627-5633）——「属性追平后转入休眠」的入口。
   - 构造 `FActorPriority` 并计数（5644-5647）。
3. 追加「已销毁的 startup/dormant actor」销毁项（5658-5666，`FActorDestructionInfo`）。
4. `Algo::SortBy(Priority, TGreater<>)` 降序排序（5669）。排序对象数为「相关集 + 待销毁集」，不是全量对象集。

**优先级的计算**（Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5152-5162 · `FActorPriority::FActorPriority`）：

```
Time = Channel ? (ElapsedTime - Channel->LastUpdateTime)   # 自上次发送经过的秒数
               : Driver->SpawnPrioritySeconds              # 新对象的基础权重（UPROPERTY(Config)）
Priority = max over viewers of round(65536 × Actor->GetNetPriority(...))
```

`AActor::GetNetPriority` 默认实现（Engine/Source/Runtime/Engine/Private/ActorReplication.cpp:48-92）：

```
if (bNetUseOwnerRelevancy && Owner): 透传 Owner 的优先级
if (本对象是 ViewTarget 或其 Instigator): Time ×= 4
elif (可见且有 RootComponent):
    Dir = 位置 - 视点; DistSq = |Dir|²
    在视野正后方:  DistSq > 2000² → ×0.2；> 500² → ×0.4        # 背对
    正前方且 0.4·DistSq < |Dir·ViewDir|²（近似正对）且 < 8000²: ×2
    其他: DistSq > 3162² → ×0.4
return NetPriority × Time
```

距离阈值是编译期宏（Engine/Source/Runtime/Engine/Public/NetworkingDistanceConstants.h:8-15：CLOSEPROXIMITY=500、NEARSIGHT=2000、MEDSIGHT=3162、FARSIGHT=8000，注释自嘲 "magic number distances"）——**不可运行期配置**。饥饿避免的真相：`Time` 项使久未更新的对象优先级线性上升；`SpawnPrioritySeconds` 保证新实体首发；两者都是排序意义上的软保证，**没有硬性的「最迟发送期限」**。

### 5.6 阶段四：ProcessPrioritizedActorsRange（发送与截断）

Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5687-5894 · `UNetDriver::ServerReplicateActors_ProcessPrioritizedActorsRange`（按优先级序遍历）：

- **预算门 1（连接级）**：`!Connection->IsNetReady()` → `GNumSaturatedConnections++`，**return 0，本连接一个对象都不发**（5695-5700）。
- 销毁项：客户端未加载对应流送 Level 则跳过（5710-5714），否则 `SendDestructionInfo`（5719）。
- **relevancy 重查节流**：已有通道的对象每 `min(RelevantTimeout, 1.0)` 秒才重查一次 relevancy（5746-5751）——**已经相关的对象不逐帧判定**，这是 relevancy 的第二个降本层（第一个是候选集频率门）。
- **滞回**：`bIsRecentlyRelevant = bIsRelevant || (Channel && ElapsedTime - Channel->RelevantTime < RelevantTimeout) || ForceRelevantFrame >= LastProcessedFrame`（5773）。`RelevantTimeout` 默认 5s（Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h:922-928，`UPROPERTY(Config)`；头文件注释明示 ReplicationGraph 与 Iris 不用此值）。
- 通道创建（5789-5806）：需 `NetGuidCache->SupportsObject(Class)` 且（startup actor 直接过 / archetype 被支持）且客户端 Level 已初始化；低频对象（`NetUpdateFrequency < 1`）失败后 0.2s 随机重试（5801-5805）。
- **预算门 2（对象级）**：`Channel->IsNetReady()` 失败 → `Actor->ForceNetUpdate()` 强制下帧再考虑（5816-5866）；发送后再次检查 `Connection->IsNetReady()`，失败则 **return j**——**截断点**（5868-5873）。返回值 j 被 `ForConnection` 用来界定「未处理区间」。
- `Channel->ReplicateActor()` 成功后：`OptimalNetUpdateDelta = clamp(距上次复制间隔 × 0.7, 1/NetUpdateFrequency, 1/MinNetUpdateFrequency)`（5849-5856）——**频率自适应是双向的**：实际间隔大 → 允许的间隔放大（但乘 0.7 留出下调空间）。
- **失相关关通道**：`!bIsRecentlyRelevant || GetTearOff()` 且有通道 → `Channel->Close(Relevancy/TearOff)`（5877-5889）；非 startup actor 立即关（=客户端销毁该 actor），startup actor 保留通道（5884，注释 "Fixme: this should be a setting"）。

### 5.7 阶段五：MarkRelevantActors（截断后的回流）

Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5896-5934 · `UNetDriver::ServerReplicateActors_MarkRelevantActors`（对未处理区间 [LastProcessedActor, FinalSortedCount)）：

- 1 秒内相关过的（`Channel->RelevantTime` 新于 1.0s）→ `bPendingNetUpdate = true`（5912-5916）；
- 仍相关的 → `bPendingNetUpdate = true` 并刷新 `RelevantTime`（5917-5926）；
- 被 `ForceRelevantFrame` 强标的 → 顺延到 `ReplicationFrame+1`（5928-5932）。

**结论：截断不是丢弃**。没发出去的相关对象通过 `bPendingNetUpdate` 绕过频率门（BuildConsiderList 5319 的条件不满足 bPendingNetUpdate=true 者），下一帧必进 ConsiderList；配合优先级的 Time 项，被截断对象自然升优先级。玩家侧的可观测现象：饱和期间新状态延迟、但不丢失（对象还活着且相关时）。

### 5.8 ForConnection：连接视角集与移动纠正的插队

Engine/Source/Runtime/Engine/Private/NetDriver.cpp:5938-6016 · `UNetDriver::ServerReplicateActors_ForConnection`：

- 构建 `FNetViewer` 列表：本连接 + 子连接（children，5956-5973）；`ANoPawnPlayerController` 可把自己剔除出 viewer（5943-5954，纯旁观/上帝视角优化的钩子）。
- **`PlayerController->SendClientAdjustment()` 在属性复制之前调用**（5977-5990），注释明说：每包最多一个移动纠正，堆叠纠正没有价值——**移动纠正的优先级高于一切属性复制**（T10 引用）。
- `OnProcessConsiderListOverride` 委托（5994-5999）：项目可以完全替换每连接的处理循环（ReplicationGraph 内部旧版即用此族钩子）。

### 5.9 字节预算的真实形状：QueuedBits token bucket

- **扣减**：`UNetConnection::FlushNet` 每发一个包 `QueuedBits += (包字节数 + PacketOverhead) × 8`（Engine/Source/Runtime/Engine/Private/NetConnection.cpp:2562-2582）。
- **回填**：`UNetConnection::Tick` 每帧 `DeltaBits = CurrentNetSpeed × BandwidthDeltaTime × 8`，`QueuedBits -= DeltaBits`；`BandwidthDeltaTime` 被 clamp 到 `1/DesiredTickRate` 以防止 hitch 后突发（NetConnection.cpp:5112-5131）。
- **上限**：`AllowedLag = 2 × DeltaBits`，`QueuedBits` 不允许低于此值（5133-5144）→ **允许的突发至多为 2 帧配额**。
- **判定**：`IsNetReady() = QueuedBits + SendBuffer.GetNumBits() <= 0`（NetConnection.cpp:2731-2751）。回放连接恒 ready（2733-2736）；dev 构建可整体禁用节流（2738-2743）；`NetworkCongestionControl`（可选项，`TOptional`）存在时改用其 `IsReadyToSend`（2745-2748，5.8 新增的实验性拥塞控制，T2/T16 展开）。
- **CurrentNetSpeed 的来源**：`UPlayer::ConfiguredInternetSpeed`（`UPROPERTY(globalconfig)`，Engine/Source/Runtime/Engine/Classes/Engine/Player.h:30-36；默认值在 Engine/Config/BaseEngine.ini:1838-1840 `[/Script/Engine.Player] ConfiguredInternetSpeed=100000 / ConfiguredLanSpeed=100000`，即 100KB/s）；DS 侧对低于 2600/1800 的值有下限钳制（NetConnection.cpp:588-596）；客户端可通过 `NMT_Netspeed` 上行协商，服务器钳到 `[1800, MaxClientRate]`（Engine/Source/Runtime/Engine/Private/PlayerController.cpp:539-542，详见 T3）。
- **没有每对象/每类的字节配额**：预算只有连接级一层。对象级的「配额」间接来自 `NetUpdateFrequency`（频率上限）与优先级（争用时的顺序）。

### 5.10 成本公式（从实现推导）

设：A = 活跃复制对象总数，C = 连接数，R_c = 连接 c 的相关集大小，V_c = viewer 数，F = 帧率，dt = 帧间隔：

```
每帧 CPU ≈
  O(A)                        # BuildConsiderList：遍历 + 频率门 + PreReplication（5315-5445）
+ Σ_c O(R_c · log R_c)        # PrioritizeActors：相关集排序（5562-5669）
+ Σ_c O(R_c)                  # ProcessPrioritizedActorsRange：通道查找 + ReplicateActor（序列化）
+ Σ_c R_c × (relevancy 判定)   # 仅对「无通道对象」全量判定（5588）；有通道对象 ~1Hz 重查（5750-5751）
+ Σ_c O(包数) × 组包           # FlushNet

其中 ReplicateActor 内部 = changelist diff（共享性见 T4.1）+ FRepLayout 序列化 + bunch 组包
带宽 ≈ Σ_c min(Σ_{a∈R_c} size(a) × f_a, NetSpeed_c)     # f_a 受自适应频率钳制
```

判定：「对象数 × 连接数」在 **relevancy、优先级、序列化、通道管理**四项上成立（R_c × C 之和）；在 **候选集构建与 changelist 比较**上不成立（每对象每帧一次）。A×C 的主导项通常来自序列化（R_c × C 的字节数）而非 CPU diff——这正是 ReplicationGraph/Iris 分别从「空间分桶减少 R_c」与「按连接增量序列化」两个方向攻击的同一问题（T6/T8）。

### 5.11 与 tick 率的解耦点

复制的节拍由 `UNetDriver::TickFlush` 驱动（Engine/Source/Runtime/Engine/Private/NetDriver.cpp:1186-1233），其调用频率由 `GEngine->GetMaxTickRate` 决定的服务器 tick 控制（T9 展开准确 CVar 名）；对象级又有 `NetUpdateFrequency`（AActor 默认 100Hz，Engine/Source/Runtime/Engine/Private/Actor.cpp:295-296：`SetNetUpdateFrequency(100.0f)` / `SetMinNetUpdateFrequency(2.0f)`）独立于 tick 率。两层频率 + 自适应插值 = 「服务器 tick 率 ≠ 对象更新率」的完整解耦。

## 流程图

见 appendix/diagrams.md 图 2（每个框标注函数名与行号区间）。

## 源码里的意外发现

1. **`FIXME: DeltaTimeOverflow is a static, and will conflict with other running net drivers`**（NetDriver.cpp:5207）——Epic 自己标注的多 NetDriver（多世界）缺陷，T1 的「单进程多世界是勉强支持」的直接证据。
2. **`//@todo - ideally we wouldn't want to tick more clients with a higher deltatime...`**（NetDriver.cpp:5211-5212）：hitch 后的补偿逻辑可能放大上游带宽饱和——Epic 明知的正反馈风险。
3. **`// Fixme: this should be a setting`**（NetDriver.cpp:5883）：startup actor 通道不关是硬编码行为。
4. **1.5 秒无数据即静默停发**（NetDriver.cpp:5243）：不告警、不关连接、只是 ViewTarget 置空——生产上表现为「玩家卡住但连接还在」，必须靠 T14 的超时路径或应用层心跳才能暴露。
5. **`net.MaxConnectionsToTickPerServerFrame` 的描述文案与名字不符**（NetDriver.cpp:5194）：描述写的是 "maximum number of channels"，实际限制的是连接数——文档性 bug 级别的注释错位。
6. 饥饿可观测性：`USE_SERVER_PERF_COUNTERS` 下记录 `ActorsStarvedByClassTimeMap`（NetConnection.cpp 命名空间；NetDriver.cpp:5832-5837）——引擎知道自己在饿对象，并为此留了按类统计的钩子。

## 对目标环境的迁移含义

目标引擎的「固定步长 + 提交点 + 状态哈希」模型可以直接吸收本章的三条结构经验而抛弃其 Actor 假设：(1) **把「预算」做成连接级 token bucket + 对象级频率门的两层结构**，并且允许突发上限显式可配（UE 的 2 帧是隐式的，5133）；(2) **截断必须回流**——UE 的 `bPendingNetUpdate` + 优先级 Time 项证明了「丢弃 vs 留存」的正确答案是留存 + 升权，对体素世界意味着「没发出去的 chunk 差量进下一帧的优先队列，且优先级随等待时长上升」；(3) **relevancy 判定节流到 ~1Hz + 滞回窗口**（5750、5773）是 UE 用血泪换来的 AOI 抖动抑制，任何「进出视野即事件」的设计都必须内建等价物（详见 T6 的离散事件裁决）。反面教训：UE 的距离阈值是编译期宏（NetworkingDistanceConstants.h），目标引擎的 AOI 半径应进 Schema/配置而非代码常量。
