# T6 · Relevancy / Dormancy / ReplicationGraph

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **Relevancy 是「无通道对象逐帧判、有通道对象 ~1Hz 判 + 5 秒滞回」的连续判定，不是离散事件**。判定本身是 O(每对象×每 viewer) 的分支链，距离剔除是平方距离比较（`DistSquared < NetCullDistanceSquared`），真正贵的历史版本（line trace）已被移出默认路径——判定在排序之前完成（NetDriver.cpp:5576-5579 的历史注释）。
2. **「进入/离开视野」的引擎语义 = 通道开/关，且天然迟到**：开通道发生在首次判定相关后的 ProcessPrioritizedActorsRange；关通道发生在「不相关状态持续超过 `RelevantTimeout`（默认 5s）」之后（NetDriver.cpp:5877-5889）。没有独立的 AOI 进出事件 API；startup actor 关通道后客户端对象还活着只是停更。**把 AOI 进出绑成实体生命周期钩子的设计，在 UE 里的对应物带着 5 秒不确定窗口 + ~1Hz 抖动抑制，这是必须内建的语义，不是可选项。**
3. **ReplicationGraph 是「把 relevancy/priority 的运行时判定换成图结构的构建期决策」**：`UReplicationGraphNode::GatherActorListsForConnection` 是纯虚入口（ReplicationGraph.h:98），节点按空间网格/频率桶/always-relevant 预分组，运行时每连接只做「取列表→距离频率二次剔除→合并排序」。代价：**IsNetRelevantFor/GetNetPriority 被整体弃用**（ReplicationGraph.h:20 注释明言），项目必须继承并实现 4~5 个虚函数（InitGlobalActorClassSettings / InitGlobalGraphNodes / RouteAddNetworkActorToNodes / RouteRemoveNetworkActorToNodes / InitConnectionGraphNodes，ReplicationGraph.h:31-44）——即把 UE 的每对象调优问题换成每类×每节点的显式图编程。

## 待证清单裁决表

| 待证 | 裁决 | 坐标 |
|---|---|---|
| IsNetRelevantFor 默认实现逐分支 | **补齐**（见 6.1） | ActorReplication.cpp:388-419 |
| 距离剔除用什么距离 | 平方距离 `< NetCullDistanceSquared`（属性，Actor.h:899-900），且受 `AGameNetworkManager::bUseDistanceBasedRelevancy` 总开关（默认 true，GameNetworkManager.cpp:54） | ActorReplication.cpp:383-386、417-418 |
| relevancy timeout 实际值与来源 | `RelevantTimeout` 默认 5.0s，`UPROPERTY(Config)`（NetDriver.h:922-928；BaseEngine.ini:1864）；注释明示 ReplicationGraph/Iris 另有配置 | NetDriver.cpp:5773、NetDriver.h:922-928 |
| 通道开关是不是进出事件 | **证实**（就是，且带滞回），见结论 2 | NetDriver.cpp:5789-5798（开）、5877-5889（关） |
| 边界抖动有没有滞回/超时 | 有：`RelevantTimeout` 滞回窗 + 重查节流 `min(RelevantTimeout,1.0)` + `RelevantTime` 随机化 `0.5*FRand()` | NetDriver.cpp:5750-5751、5773、5813 |
| ReplicationGraph 节点清单 / 调用链 / 成本迁移 | **补齐**（见 6.4） | ReplicationGraph.h:69-1284；ReplicationGraph.cpp:1112、1292-1297、1706、2254 |

## 6.1 IsNetRelevantFor 默认实现的分支序（ActorReplication.cpp:388-419）

```
IsNetRelevantFor(RealViewer, ViewTarget, SrcLocation):
 1. bAlwaysRelevant || IsOwnedBy(ViewTarget) || IsOwnedBy(RealViewer)
    || this == ViewTarget || ViewTarget == GetInstigator()      → true    # :390
 2. bNetUseOwnerRelevancy && Owner → 透传 Owner 的判定                     # :394-397
 3. bOnlyRelevantToOwner                                                  → false   # :398-401
 4. 挂接在别人的 SkeletalMesh / 同 Owner 的挂接父 → 透传父 Owner 判定       # :402-405
 5. IsHidden() && (无 Root || 无碰撞)                                     → false   # :406-409
 6. 无 RootComponent → Warning 日志 + false                                # :411-415
 7. !bUseDistanceBasedRelevancy → true；否则 DistSquared(SrcLocation, ActorLocation)
    < GetNetCullDistanceSquared()                                         # :417-418 + :383-386
```

APawn 重载了同函数（Pawn.cpp:1290-1300 附近），view 目标自己是 pawn 时恒相关。viewer 的 `SrcLocation` 来自 `FNetViewer`（连接 ViewTarget 的位置；每帧在 ForConnection 里构建，见 T5.8）。

**短路点**：`bAlwaysRelevant` 在第一行——它同时跳过距离与所有其他判定，这也是 NetCullDistanceSquared 对它无效的原因。`bOnlyRelevantToOwner` 的通道级行为见 T5（PrioritizeActors 里先做属主判定，非属主且超时关通道，NetDriver.cpp:5597-5617）。

## 6.2 Dormancy：状态、进入/退出、经典 bug 的源码形状

**ENetDormancy 全表**（EngineTypes.h:3595-3611，语义照抄注释）：

| 值 | 名 | 语义 |
|---|---|---|
| 0 | DORM_Never | 永不休眠 |
| 1 | DORM_Awake | 可休眠但当前醒着；由游戏代码决定何时休 |
| 2 | DORM_DormantAll | 想对所有连接完全休眠 |
| 3 | DORM_DormantPartial | 按连接决定，GetNetDormancy() 被逐 viewer 调用 |
| 4 | DORM_Initial | 地图放置对象的初始休眠 |

**进入**：经典管线里 `ShouldActorGoDormant`（NetDriver.cpp:5506-5526）→ `Channel->StartBecomingDormant()`（:5627-5633）——**先追平属性再关通道**；`UActorChannel::BecomeDormant` 最终 `Close(EChannelCloseReason::Dormancy)`（DataChannel.cpp:4597-4603）。对象级记录在 `FNetworkObjectInfo::DormantConnections`（NetDriver.cpp:5499-5503 的 IsActorDormant 即查此集合）。
**退出**：只有两个入口——`AActor::SetNetDormancy(<=DORM_Awake)`（Actor.cpp:3051-3099：NotifyActorDormancyChange + 重新 AddNetworkActor + FlushActorDormancy）与 `AActor::FlushNetDormancy`（Actor.cpp:3102-3135，DORM_Initial 先降级为 DORM_DormantAll 再强制一次复制）。
**经典 bug 的源码形状**：休眠期间对象属性的任何写入都**不触发任何机制**——没有把「属性变更」与「唤醒」绑定的钩子；忘了调 FlushNetDormancy 的直接后果是客户端永远停在旧值。Epic 自己知道这一点：`net.DormancyValidate=2` 会用 `FObjectReplicator::ValidateAgainstState` 对全部休眠对象做「关灯前对账」（NetDriver.cpp:6390-6405）——一个专门为「忘了唤醒」准备的调试武器。
**Initial dormant**：地图放置对象在 BuildConsiderList 里被直接移出网络对象列表（NetDriver.cpp:5369-5377）——它复活的唯一途径就是 FlushNetDormancy 的降级路径。

## 6.3 ReplicationGraph：图结构 replaces 运行时判定

- 插件：Engine/Plugins/Runtime/ReplicationGraph（Beta、默认禁用，见 T0 表）。入口：`UReplicationGraph::ServerReplicateActors`（ReplicationGraph.cpp:1112）接管整个循环。
- 节点类型清单（ReplicationGraph.h）：`UReplicationGraphNode`（抽象，:69，`GatherActorListsForConnection` PURE_VIRTUAL :98）→ ActorList（:188，列表容器，子节点递归 :3674-3680）→ ActorListFrequencyBuckets（:239，频率桶）→ DynamicSpatialFrequency（:322，动态空间+频率）→ ConnectionDormancyNode（:428）/ DormancyNode（:485，休眠专门节点）→ GridCell（:535）/ GridSpatialization2D（:579，2D 网格空间化）→ AlwaysRelevant（:771）/ AlwaysRelevant_ForConnection（:827）→ TearOff_ForConnection（:888）。连接侧 `UNetReplicationGraphConnection : UReplicationConnectionDriver`（:1284）。
- 调用链：ServerReplicateActors（:1112）→ 每连接遍历节点 `Node->GatherActorListsForConnection(Parameters)`（:1292-1297）→ 距离/频率二次剔除 → 合并排序 → `ReplicateActorsForConnection`（:1706）→ 单 actor `ReplicateSingleActor`（:2044）；饥饿处理 `HandleStarvedActorList`（:2254）。
- **成本被挪到哪里**：头文件注释（ReplicationGraph.h:18-27）说得很直白——IsNetRelevantFor/GetNetPriority 不再被调用，影响复制的三条路变成：图结构本身、FGlobalActorReplicationInfo（每 actor 全局关联数据）、FConnectionReplicationActorInfo（每连接每 actor 关联数据）。**手工调优的量化**：子类必须实现 31-44 行列出的 4~5 个虚函数；每类 actor 的策略在 InitGlobalActorClassSettings 里显式声明。以 Fortnite 级项目论，这是数千行的图组装代码（Estimated，依据：节点数量 × 每类策略；引擎内示例见同目录测试）。
- 对象按节点分桶后，**每连接的相关集不再是「全对象×判定」而是「命中的桶列表」**——这是它对 T5 成本公式中 ΣR_c 项的攻击方式。

## 6.4 对体素 Chunk 的直接启示（每条挂坐标）

1. **粒度错配**：UE 的复制/相关性/dormancy 单位是 Actor（含组件子对象），NetCullDistanceSquared 是每 actor 属性（Actor.h:899-900）。一秒几万次写入的体素世界若以 chunk=actor 建模：NetworkObjectList 的每次遍历 O(A)、每连接通道管理 O(R_c×C)、FRepChangelistState 每 chunk 一份影子——三项都会被 chunk 数量线性放大。**结论：chunk 不能走 actor 通道**；UE 内正确对应物是 FastArray/NetDeltaSerialize 这类「一个 actor 内的增量数组」（FastArraySerializer.h:50-55）或 Iris 的 DataStream 批量化（T8）。
2. **RelevantTimeout 滞回 + 1Hz 重查**（NetDriver.cpp:5750、5773）是边界抖动的工程答案，体素 AOI 必须等价物（例如「chunk 进出 AOI 有 2×半径差的滞回带」）。
3. **Dormancy 的「追平后停发」**（StartBecomingDormant，NetDriver.cpp:5627-5633）对静态 chunk 语义上是免费的带宽优化，但 UE 把唤醒责任全推给游戏代码（6.2）——目标引擎应把「写入自动唤醒」做成提交点内建行为。
4. **GridSpatialization2D**（ReplicationGraph.h:579）证明 Epic 对大规模空间相关性的答案就是网格分桶——体素世界的 chunk 分桶与之同构，但 UE 的桶仍以 actor 为成员，收益被 actor 粒度封顶。

## 意外发现

1. ReplicationGraph.h:20：`The main impact here is that virtual functions like IsNetRelevantFor and GetNetPriority are not used by the replication graph.`——同一个引擎内两套 relevancy 语义并存且互不兼容；游戏若两者混用（部分 actor 走图、部分走经典）行为差异没有任何文档级提示，只有这条头注释。
2. `NetCullDistanceSquared` 存的是**平方**（属性名自带 Squared，Actor.h:900），蓝图里直接填线性距离是常见的项目级坑——引擎只提供 Get/Set 访问器缓解（UE_DEPRECATED 5.5 提示，Actor.h:898-899）。
3. 隐藏且无碰撞的 actor 直接不相关（ActorReplication.cpp:406-409）——「用隐藏做视觉优化」会顺带杀死复制，这是复用渲染状态做网络判定的耦合实例。
4. `IsReplayRelevantFor` 默认直接转调 `IsNetRelevantFor`（ActorReplication.cpp:421-424）——回放相关性复用实时相关性，没有独立的「观战视角」语义（可重载）。

## 对目标环境的迁移含义

目标引擎的 AOI 应做成显式的三层而非 UE 的隐式堆叠：(1) **空间索引层**（chunk 网格桶，对应 GridSpatialization2D 的思路但以 chunk 为单位）；(2) **判定层带滞回与节流**（进/出 AOI 半径差 + 判定频率上限，对应 RelevantTimeout/min(RelevantTimeout,1.0)）；(3) **事件层与生命周期解耦**——UE 把「离开视野」实现成关通道（startup actor 甚至不销毁对象，NetDriver.cpp:5884），证明「视野事件」不应绑定实体销毁。体素世界的正确语义是：离开 AOI = chunk 副本进入冷冻（停发差量但保留状态摘要与版本号），进入 AOI = 从最近快照 + 差量链追平——这恰好是 UE dormancy 想做而受限于 actor 模型做不干净的事。
