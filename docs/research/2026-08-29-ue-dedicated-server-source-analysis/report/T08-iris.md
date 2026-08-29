# T8 · Iris：Epic 自己对旧设计的重写

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **Iris 在 5.8 的形态是「Beta 插件 + 运行时模块 IrisCore + Engine 模块内的桥」三位一体**：插件壳 Engine/Plugins/Experimental/Iris（IsBetaVersion=true，EnabledByDefault=false）只是启动桩；真代码在 Engine/Source/Runtime/Net/Iris（IrisCore.Build.cs:5）；与引擎的缝合在 Engine 模块（Engine/Public/Net/Iris/EngineReplicationBridge.h 等 + Private/Net/Experimental/Iris/DataStreamChannel.cpp）。总开关 CVar `net.Iris.UseIrisReplication` **默认 0**（IrisConfig.cpp:16）；每 NetDriver 配置 `bCanUseIris`（BaseEngine.ini:346-347：GameNetDriver=true、DemoNetDriver=false）。TickFlush 层二选一：有 ReplicationSystem 走 `InternalIrisUpdateTransactional`，否则经典 `ServerReplicateActors`（NetDriver.cpp:1212-1231）；`net.iris.UpdateReplicationSystemWithNoConnections`（默认 true）让 Iris 在 0 连接时也推进——经典路径做不到（NetDriver.cpp:6282-6285）。
2. **架构上的四个正交件，每一件都对应旧系统的一个结构性病**：量化内部状态副本（FReplicationStateDescriptor 的 External/Internal 双偏移——外部游戏内存与线上量化态解耦）；change-mask 脏追踪为**默认**而非可选（对照 T4 push-model 默认关）；filtering 与 prioritization 是**独立注册的接口族**（UNetObjectFilter / UNetObjectPrioritizer，ReplicationSystem.h:32-33、42-44——对照 ReplicationGraph 把 relevancy/priority 焊进图结构）；DataStream 抽象把「写什么流、怎么响应包交付状态」下放（FDataStreamRecord + ProcessPacketDeliveryStatus，DataStream.h:35-43）。
3. **Epic 对旧系统的自我批评主要写在结构里，而非檄文里**（模块内无设计宣言文档，检索 Iris Public 全 *.h 无 legacy 大段陈述；最接近的自述是 NetTokenStore.h:21-32 对「Iris 与旧复制系统」的并列描述与 ReplicationFragment.h:129-133 的 `NeedsLegacyCallbacks/NeedsPoll` trait——旧系统「轮询+回调」被当作需要显式声明的兼容负担）。由结构反推（此段为**推断**，依据下列坐标）：Iris 承认的旧病 = 比较成本靠轮询、每连接序列化重复劳动（量化态可跨连接共享、delta 压缩按连接基线）、relevancy 语义与传输节拍耦合（filtering/prioritization 独立）、大数据无通道化出口（ChunkedDataStream）。

## 8.1 量化状态副本（对照 T4 影子状态）

FReplicationStateDescriptor（Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationState/ReplicationStateDescriptor.h:28-101+）：

- `FReplicationStateMemberDescriptor`：**ExternalMemberOffset / InternalMemberOffset 双表示**（:29-33）——游戏侧内存布局与量化后内部态是两个坐标系，序列化永远发生在内部态上。
- `FReplicationStateMemberSerializerDescriptor`：每成员 `FNetSerializer + Config`（:35-40）——序列化器是显式注册的对象，不是属性反射直通。
- `EReplicationStateMemberTraits`（:42-52）：`HasDynamicState / HasObjectReference / HasConnectionSpecificSerialization / HasRepNotifyAlways / UseSerializerIsEqual / HasPushBasedDirtiness`——**每连接序列化差异**（对照 ELifetimeCondition 的连接角色掩码）与**推送式脏**是一等 trait。
- `FReplicationStateMemberChangeMaskDescriptor`（:76-80）：每成员脏位（数组可多 bits）——脏是描述符驱动的默认，非 CVar 开关。
- 对象引用带解析策略（FNetReferenceInfo::EResolveType :82-97：ResolveOnClient / MustExistOnClient / ResolveOnlyWhenRecvd）——unmapped-object 的等待语义进了类型系统（对照旧系统 DataReplication 的 UnmappedGuids 队列）。

## 8.2 filtering / prioritization / delta 压缩 / 流

- Filter/Prioritizer：`UNetObjectFilter`、`UNetObjectPrioritizer`（ReplicationSystem.h:32-33）+ 各自 Handle 类型（:42-43）；FWorldLocations（:54）独立维护空间位置源——**AOI 判定的输入与复制节拍解耦**。
- Delta 压缩：`ENetObjectDeltaCompressionStatus`（:65）+ NetObjectDeltaCompressionBaselineStorage（模块内，见 T15 搜索记录）——按连接基线的增量压缩内建。
- DataStream（DataStream.h:22-54）：`EDataStreamWriteMode{Full, PostTickDispatch, DebugData}`——**调试数据不计带宽**是流抽象的一等能力（可观测性进协议）；每流自行管理 FDataStreamRecord 以响应 `ProcessPacketDeliveryStatus`（:35-43）——流知道包丢了该重写什么。分块流存在（ChunkedDataStreamCommon.h:17 的 LogIrisChunkedDataStream）——大体量数据（对体素有直接参考价值）有独立出口。
- Token 化：FNetTokenStore/StringTokenStore/FNameTokenStore（ReplicationSystem.h:51-53）——字符串/FName/结构化值（GameplayTags）在线上变 token；NetTokenStore.h:21-32 注释并列描述两代系统的 token 算法。
- 连接侧的 Iris 通道：DataStreamChannel（Engine/Private/Net/Experimental/Iris/DataStreamChannel.cpp，且直接读写 `Connection->QueuedBits`——Iris 复用连接级带宽预算变量，:303、405-413）。
- 协议不匹配的诊断协议：NMT_IrisProtocolMismatch / NMT_IrisNetRefHandleError(/WithDiagnosticMessage)（T3 表）——两端 Schema/句柄漂移有显式上报路径。

## 8.3 已接/未接（通用侧裁决）

| 子系统 | Iris 化 | 坐标 |
|---|---|---|
| Actor/组件复制 | ✅ 经 UEngineReplicationBridge（Public/Net/Iris/EngineReplicationBridge.h；EngineReplicationBridge.cpp:753/776 SetMaxTickRate 等） | |
| 移动 RPC | ✅ 有 Iris 感知路径（CMC 的 `GetIrisPackageMapToCaptureReferences`，CharacterMovementComponent.cpp:9098-9110） | |
| FastArray | ✅ IrisFastArraySerializer.h（ReplicationState 目录） | |
| RPC 队列 | ✅ net.UnreliableRPCQueueSize=10 / net.ReliableRPCQueueSize=4096 等（AttachmentReplication.cpp:23-33） | |
| 可靠性 | ⚠️ ReliabilityHandlerComponent（PacketHandlers 模块兄弟目录，Iris 路径使用） | |
| 回放（DemoNetDriver） | ❌ 显式禁用（BaseEngine.ini:347 bCanUseIris=false）；新 UReplayNetConnection 是为其预留的录制侧替代（T13） | |
| GAS | 交给 GAS 篇（本篇不展开） | |

## 意外发现

1. `UE_NUM_INLINE_REPLICATIONSYSTEMS` 默认 8（ReplicationSystem.h:22-24）——多复制系统实例（多世界/多房间设想）在 Iris 里是显式预留的（对照 T1 的经典 NetDriver 多世界缺陷）。
2. `net.iris.AllowParallelNetTick`（NetDriver.cpp:380，默认 false）——连接 tick 并行化在 Iris 路径上是开关化的（经典路径无此能力）。
3. Iris 仍写 `Connection->QueuedBits`（DataStreamChannel.cpp:303、405-413）——带宽预算这层两代共用，证明 T5.9 的 token bucket 是传输无关的正确抽象层。
4. LogIrisDirtyTracker / LogIrisCreationFlowLog 等专属日志分类（IrisLog.h:7-9 等）——新系统把可观测性当日建，对照旧系统 LogNetTraffic 的 Warning 级默认。

## 对目标环境的迁移含义

Iris 验证了目标引擎的两条既定方向并给出第三条：(1) **量化内部态与游戏内存分离**（External/Internal 双 offset）与「Schema 生成 + 规范化字节」同构，且 Iris 证明它可以与反射源共存——迁移期可用双表示过渡；(2) **脏追踪默认进描述符**而非可选 CVar——目标引擎的提交点天然产生全量脏集，等价且更强；(3) **filtering 与 prioritization 作为独立接口族 + 独立空间源（FWorldLocations）**是对 T6「relevancy 焊死在循环里」的官方否定——目标引擎的 AOI 应照此分离。DataStream 的 `DebugData` 免带宽通道与「流自管交付记录」两条，值得直接进传输 Profile 的需求清单。
