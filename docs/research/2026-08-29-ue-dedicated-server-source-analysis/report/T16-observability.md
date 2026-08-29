# T16 · 可观测性设施的准确清单

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。完整 CVar/ini/命令表见 appendix/cvar-ini-and-commands.csv。

## 结论先行

1. **预研的「名称待核」全部钉死，其中四条是纠错**：不存在 `net.MaxTickRate`（真名是 ini 键 `NetServerMaxTickRate=30`，T9）；不存在 `MaxReliableBuffer` ini 键（是编译期 `RELIABLE_BUFFER=512`）；`LogRepFastArray` 真名是 `LogNetFastTArray`（NetCore 模块，FastArraySerializer.h:41）；`STATGROUP_NetTraffic` 不存在（只有 STATGROUP_Net 与 STATGROUP_Packet）。
2. **网络 trace 只有唯一一个 channel：`NetChannel`**（NetTraceReporter.cpp:16-17），描述明言服务对象是 Network Insights profiler；stat 族以 `stat net`/`stat packet` 可用（组名=STATGROUP_ 前缀剥离，UnrealEngine.cpp:6029-6057、:6039）。
3. **定位 DS 问题所需五类观测量，UE 提供四类半**：带宽构成（有：InRate/OutRate/PerConnection + CSV 分类）、每对象复制成本（有：profiler 宏下 TrackReplicatePropertiesMetadata，DataReplication.cpp:2018-2023；`ActorsStarvedByClassTimeMap` 饥饿统计，NetDriver.cpp:5832-5837）、丢包重传（有：OutLoss/InLoss、PacketOrderCorrection 族）、通道/队列深度（半：STAT_OutgoingReliableMessageQueueMaxSize/IncomingReliableMessageQueueMaxSize，EngineStats.h:259-260；通道队列无直接 stat，靠日志）、tick 超时（有：STAT_NetTickFlush/STAT_NetServerRepActorsTime 族 + `net.ReportGameTickFlushTime`）。

## 16.1 日志分类全表（网络相关）

| 分类 | 声明/定义 | 默认级别 |
|---|---|---|
| LogNet / LogNetLifecycle / LogNetSubObject / LogRep | EngineLogs.h:21-24；定义 DataChannel.cpp:51-58 | Log |
| LogNetPlayerMovement | EngineLogs.h:25；:54 | Warning（生产降噪） |
| LogNetTraffic / LogRepTraffic / LogNetDormancy | EngineLogs.h:26-28；:55-58 | Warning |
| LogNetPartialBunch | static，DataChannel.cpp:60 | Warning |
| LogNetPackageMap / LogNetSerialization | CoreGlobals.h:34-35 | Warning |
| LogNetFastTArray | FastArraySerializer.h:41（NetCore） | 编译期 All/运行期 Warning（:32-39 宏可调） |
| LogNetToken / LogNetCore | NetToken.h:10 / NetCoreLog.h:7（NetCore） | — |
| LogIris / LogIrisFiltering / LogIrisNetCull / LogIrisCreationFlow / LogIrisFilterConfig / LogIrisDirtyTracker / LogIrisChunkedDataStream / LogNetStats | IrisLog.h:7-9 等（IrisCore） | — |
| LogDemo | ReplayTypes.h:19；定义 DemoNetDriver.cpp:41 | Log |
| LogNetVersion | NetworkVersion.cpp（使用处 :234） | — |

不存在：`LogPacketHandler`（PacketHandler 相关日志走 LogNet/LogNetTraffic）。

## 16.2 CVar 家族的分区地图（全表见 CSV）

- 调度/预算：net.UseAdaptiveNetUpdateFrequency(0)、net.DisableRandomNetUpdateDelay(0)、net.MaxRPCPerNetUpdate(2)、net.MaxConnectionsToTickPerServerFrame(0)。
- 可靠性/通道：net.MaxChannelSize(0)、net.MaxConstructedPartialBunchSizeBytes(65536)、net.PartialBunchReliableThreshold(8)、net.PartialBunchReliableThreshold 族、net.QueuedBunchTimeoutSeconds(30)。
- 休眠：net.DormancyEnable(1)、net.DormancyValidate(0)、net.DormancyHysteresis(0)、Net.ReuseReplicatorsForDormantObjects(0)。
- 复制内核：net.ShareShadowState(1)、net.ShareSerializedData(1)、net.ShareInitialCompareState(0)、Net.IsPushModelEnabled(false)、Net.UsePackedShadowBuffers(1)、net.PushModelValidateProperties(false)。
- 连接/超时/关闭：net.GracefulCloseEnabled(true)、net.EnableCongestionControl(0)、netEmulation.*（丢包/延迟/缓冲膨胀仿真——**测试利器**，前缀 netEmulation.）、net.IpConnectionUseSendTasks?（未核实，不在本次清单）。
- 观测本身：net.EnableNetStats(false)、net.EnableActorCountInStats(false)、net.ReportGameTickFlushTime(false)、net.DebugDraw(0)。
- 调试命令：net.DumpActiveNetActors（NetDriver.cpp:433）、net.DumpRelevantActors(:9314)、net.PrintNetConnections(:9375)、net.ListNetGUIDs(DataChannel.cpp:5712)、net.ListActorChannels(:5769)、net.SimulateConnections(NetConnection.cpp:6404)、net.ForceOnePacketPerBunch(:6408)、Net.PushModelPrintHandles(PushModel.cpp:448)。

## 16.3 一个网络服务器最少需要什么 vs UE 提供什么

| 观测量 | UE 提供 | 坐标/缺口 |
|---|---|---|
| 带宽构成（按通道类型/按 actor 类） | stat net + CSV Profiler 分类 + profiler 宏 | EngineStats.h:208-260；CSV_RECORD_DETAILED_ACTOR_NET_STATS 宏族 |
| 每对象复制成本 | Network Profiler（编译期 NETWORK_PROFILER 宏；TrackReplicatePropertiesMetadata DataReplication.cpp:2018-2023） | 需非 shipping 构建 |
| 饥饿/降频可观测 | ActorsStarvedByClassTimeMap（USE_SERVER_PERF_COUNTERS，NetDriver.cpp:5832-5837）；STAT_NetSaturated | 默认关闭（编译期宏） |
| 丢包/乱序/重传 | STAT_OutLoss/InLoss、PacketOrderCorrection CVar 族、NetPing 族（net.NetPing*，NetPing.cpp:38-79，带 ICMP/UDP 双模式） | 服务器侧 RTT 观测完整 |
| 队列深度 | 可靠队列 MAXSize 两个 stat（EngineStats.h:259-260）；**通道级队列深度无 stat**——缺口 | 缺口 |
| tick/复制超时 | STAT_NetTickFlush、STAT_NetServerRepActorsTime（NetDriver.cpp:161-165）、CSV_SCOPED_TIMING_STAT_EXCLUSIVE(ServerReplicateActors)（NetDriver.cpp:1210）、GTickFlushGameDriverTimeSeconds | 有 |
| Iris | 独立 metrics（FNetMetrics）+ 专属日志族 + NetStats | IrisCore/Stats |

**缺口清单**（UE 没有的）：每连接的「实际收到状态滞后量」（状态新鲜度）；每对象「上次成功送达时间」的直接读数（要靠 profiler）；协议级 diff 采样（两个版本字段集差异——只有 net.DoPropertyChecksum=0 的调试位）；生产可用的按字段带宽归因（要 profiler 构建）。

## 对目标环境的迁移含义

把 UE 的三层结构抄走并补缺口：stat（轻量、常开、按组聚合）→ CSV/trace（服务级时序）→ profiler（重武器、采样级归因）。目标引擎多出的 WAL/状态哈希是 UE 没有的观测金矿：每帧哈希链天然给出「两端状态漂移」的最终裁决观测，应与带宽/队列观测并列为一等指标；浏览器端还应把「传输层信号（RTT 膨胀、流控窗口）」纳入统一指标面——UE 的传输与复制观测是割裂的（T11 结论 2）。
