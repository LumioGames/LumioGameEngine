# T12 · 世界流送的服务器端与「未加载 ≠ 不相关」

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **服务器流送是可选开关且默认关**：WorldPartition 的 `wp.Runtime.EnableServerStreaming` 默认 0、`wp.Runtime.EnableServerStreamingOut` 默认 false（WorldPartition.cpp:128-138）——**DS 默认把整个世界全部加载常驻**，内存换简单性。开启后服务器按流送源逐 cell 加载/卸载（`UWorldPartition::IsServerStreamingEnabled/IsServerStreamingOutEnabled`，WorldPartition.cpp:1384-1458，解析后缓存且「不允许运行时改状态」）。
2. **「没加载」与「不相关」在 UE 协议层是两个独立信号，客户端可以区分，但引擎主要用「发不发」掩盖区别**：客户端的流送可见性由服务器通过 `NMT_DebugText` 之外的独立流（UpdateLevelVisibility / ServerUpdateLevelVisibility，PlayerController 复制）下发；复制侧的门有三处——BuildConsiderList 跳过「Level 可见性变更中」的对象（NetDriver.cpp:5362-5367）、PrioritizeActors 的 `IsLevelInitializedForActor`（:5582-5586）、销毁信息的 `ClientVisibleLevelNames` 检查（:5710-5714）。客户端收到 `MissingLevelPackage` 断连（NetConnection.cpp:1976）——**缺包是断连级错误**。
3. **UE 没有做到「缺失 ≠ 空」的铁律**：startup actor 的销毁在客户端未加载该 Level 时被扣住（`net.SendDormantDestructionOnRemoval`、`net.SkipDestroyNetStartupActorsOnChannelCloseDueToLevelUnloaded` 默认 true，DataChannel.cpp:175），但**非 startup actor 关通道即销毁**（NetDriver.cpp:5877-5889）——客户端「没收到」和「对象没了」在对象层不可区分，没有「未加载区域的状态仍存在、只是没发」的显式语义。流送边界两侧的实体状态由 GameMode 的 GetSeamlessTravelActorList/项目逻辑自行处理。

## 12.1 服务器端流送行为

- 开关与模式：`EWorldPartitionServerStreamingMode`（ProjectDefault/Enabled/EnabledInPIE，WorldPartition.cpp:1393-1425）+ 两级全局 CVar；逐世界配置（`.ini` 的 WorldPartition 段）。
- 卸载后对象：`ServerStreamingOutEnabled` 时 cell 可在服务器卸载——卸载中的 Level 上的 actor 不进 ConsiderList（HasVisibilityChangeRequestPending / bIsAssociatingLevel，NetDriver.cpp:5363）；**卸载期间的「状态保真」没有引擎级方案**（transform-only/序列化保留是项目侧做法）。
- LevelScriptActor 的 changelist 在 Level 卸载时从驱动移除（NetDriver.cpp:4784、7289）。

## 12.2 「未加载 vs 不相关」的协议证据

| 信号 | 坐标 | 语义 |
|---|---|---|
| IsLevelInitializedForActor | NetDriver.cpp:5582、5744 | 客户端**是否已加载**该 actor 所在 Level——与相关性无关的门 |
| ClientVisibleLevelNames | NetDriver.cpp:5710 | 销毁信息只发已加载 Level 的 |
| UpdateLevelVisibilityInternal | NetConnection.cpp:1886-2037 | 服务器把「该客户端可见的 Level 集」下发；服务器自己也流送时会校验（IsServerStreamingLevelVisible 警告，:1886） |
| MissingLevelPackage 断连 | NetConnection.cpp:1976 | 客户端缺包 → 断连（可 `net.SkipMissingLevelDisconnect` 抑制） |

客户端能区分吗？能到什么程度：客户端知道「服务器通知我加载过哪些 Level」（可见性流）与「本地的加载状态」；但**协议上没有「服务器侧存在但没发给你的对象集合」的任何表示**——不存在「这个区域有 3 个实体未同步」的元数据。对观战/迟加入者，信息真空就是真空。

## 12.3 对目标引擎铁律的对照（反面教材写透）

UE 的立场：**「未加载」是客户端本地资源状态，「不相关」是发送决策，两者正交但协议不向客户端披露第三态**。目标铁律「缺失的 chunk 永远不等于空世界」要求的恰是第三态显式化：每个未同步区域携带「存在性元数据」（版本号/摘要/实体计数）。UE 里最接近的原语是 dormancy（冻结而非销毁）与 startup actor 的通道保留（NetDriver.cpp:5884）——**两者都只对「曾经同步过」的对象成立**，从未覆盖「从未同步」的区域。体素世界必须在协议里加 UE 没有的东西：chunk 存在性清单（如每区域摘要头：chunk_hash、版本、实体计数），缺失时客户端显示「未加载」而非空气。

## 意外发现

1. `net.SkipDestroyNetStartupActorsOnChannelCloseDueToLevelUnloaded`（DataChannel.cpp:175，默认 true）——「卸载导致的关通道不销毁 startup actor」是个 CVar 兜底的妥协行为，说明此处语义从未定案。
2. 服务器流送解析结果缓存且禁止运行时切换（WorldPartition.cpp:1386 注释 "we don't allow changing the state at runtime"）——运行时动态开流送不在支持范围。
3. `wp.Runtime.DebugDedicatedServerStreaming`（WorldPartition.cpp:115）——DS 流送有自己的调试 CVar，Epic 知道这块难观察。

## 对目标环境的迁移含义

采纳 UE 的「两级门」结构（资源门+相关性门分开判定）但必须补第三态：目标引擎的 chunk 状态机应为 未加载/加载中/活跃/冻结 四态显式进线协议，未加载与冻结都携带存在性元数据（哈希+版本）；服务器侧 chunk 驻留预算（画像中的驱逐栅栏）对应 UE 的 ServerStreamingOut——但 UE 没解决「驱逐后状态保真」，目标的 WAL/快照体系天然给出答案（驱逐 = 回到 WAL 可重建态），这是 UE 做不到而目标架构免费的部分。
