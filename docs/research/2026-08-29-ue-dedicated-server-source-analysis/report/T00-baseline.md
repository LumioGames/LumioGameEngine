# T0 · 基线、模块地图与读取纪律

## 版本三件套（R4，所有行号相对此版本）

| 项 | 值 |
|---|---|
| Engine/Build/Build.version | MajorVersion 5 / MinorVersion 8 / PatchVersion 2；BranchName "UE5"；Changelist 0；CompatibleChangelist 55116800；IsLicenseeVersion 0；IsPromotedBuild 0 |
| git HEAD | ff8421f2b8cb4feb76fff57965a1effc53a6eb7b（branch 5.8） |
| 最近提交 | ff8421f2b 2026-08-25 "Localization Automation using CL 57313377" |
| 本报告记法 | UE 5.8.2 / CL 55116800 / ff8421f2b |

## 关键插件成熟度（逐个打开 .uplugin 读字段，非凭印象）

| 插件 | 路径 | EnabledByDefault | 成熟度字段 | 备注 |
|---|---|---|---|---|
| Iris | Engine/Plugins/Experimental/Iris/Iris.uplugin:5-16 | false | **IsBetaVersion=true**（IsExperimentalVersion 无/false） | 插件本体只是 14 行启动桩（Engine/Plugins/Experimental/Iris/Source/Iris/Private/Iris/IrisModule.cpp）；真正代码是运行时模块 **IrisCore**（Engine/Source/Runtime/Net/Iris/IrisCore.Build.cs:5）。引擎侧集成在 Engine 模块：Engine/Source/Runtime/Engine/Public/Net/Iris/* |
| NetworkPrediction | Engine/Plugins/Runtime/NetworkPrediction/NetworkPrediction.uplugin:5-15 | false | IsBetaVersion=true | 伴生 NetworkPredictionExtras（Beta，"Not intended to be used directly in a shipping product"）与 NetworkPredictionInsights（EditorAndProgram） |
| ReplicationGraph | Engine/Plugins/Runtime/ReplicationGraph/ReplicationGraph.uplugin:5-15 | false | **IsBetaVersion=true** | 独立插件（Category "Performance"，LoadingPhase PreDefault）。**`WITH_REPLICATION_GRAPH` 宏在本版本不存在**（检索 Engine/Source/Runtime/Engine、Net、全部 *.Build.cs：0 命中）；引擎侧只留抽象基类 UReplicationDriver（Engine/Source/Runtime/Engine/Classes/Engine/ReplicationDriver.h:48） |
| WebSocketNetworking | Engine/Plugins/Experimental/WebSocketNetworking/WebSocketNetworking.uplugin:5-15 | false | 无 Beta/Experimental 字段，但目录在 **Experimental** | FriendlyName 自称 "Experimental WebSocket Networking Plugin"；PlatformAllowList Mac/Win64/Linux |
| WebSockets（检索线索说是插件） | — | — | — | **不是插件**。是运行时模块 Engine/Source/Runtime/Online/WebSockets/WebSockets.Build.cs:5（libWebSockets/WinHttp 双后端）；另有 WebSocketServer 模块（Engine/Source/Runtime/Online/WebSocketServer/） |
| ReplaySystemNext（检索线索） | — | — | — | **源码中不存在**（Engine/Plugins 下 *Replay* 插件仅 ReplayTracks、IOSReplayKit）。新一代回放是引擎内部的 UReplaySubsystem/UReplayNetConnection（见 T13） |
| OnlineSubsystem / OnlineSubsystemNull | Engine/Plugins/Online/OnlineSubsystem{,Null}/*.uplugin:4-11 | **true** | 无 Beta/Experimental 标记（成熟） | Null 子系统是 DS 无平台时的默认会话层 |
| OnlineSubsystemUtils（插件） | Engine/Plugins/Online/OnlineSubsystemUtils/ | true | 成熟 | **默认 NetDriver UIpNetDriver 在这里**（Source/OnlineSubsystemUtils/Classes/IpNetDriver.h:65），Beacon 全家桶也在此 |

## 网络相关模块地图（引擎提供了什么、在哪个模块）

| 模块/目录 | 职责 | 代表文件 |
|---|---|---|
| Engine/Source/Runtime/Sockets | 平台无关 socket 抽象（FSocket/ISocketSubsystem） | Public/Sockets.h、SocketSubsystem.h；平台实现 Private/{BSDSockets,Windows,...} |
| Engine/Source/Runtime/Networking | socket 之上的 TCP/UDP 辅助层（TcpListener/UdpSocketReceiver 等）。**Readme 自述 "internal R&D effort... Production use is NOT encouraged"** | Public/Common/TcpListener.h、UdpSocketReceiver.h |
| Engine/Source/Runtime/PacketHandlers/PacketHandler | PacketHandler 组件链框架（独立模块，不在 Engine 内） | Public/PacketHandler.h；Private/PacketHandler.cpp（1358 行） |
| Engine/Source/Runtime/PacketHandlers/ReliabilityHandlerComponent | 可靠性组件（Iris 路径使用） | 同目录 |
| Engine/Source/Runtime/Net/Core（NetCore） | NetBitArray、**PushModel**、NetToken/NetHandle、RPC DoS 检测、FastArraySerializer | Public/Net/Core/* |
| Engine/Source/Runtime/Net/Iris（IrisCore） | Iris 复制系统本体 | Public/Iris/ReplicationState、ReplicationSystem、DataStream、Serialization |
| Engine/Source/Runtime/Net/Common | 公共小类型 | Public/Net/Common |
| Engine/Source/Runtime/Engine（Engine 模块内网络部分） | NetDriver/NetConnection/Channel/RepLayout/DataReplication/StatelessConnect 等 | Private/NetDriver.cpp（9467 行）、RepLayout.cpp（8607）、NetConnection.cpp（6655）、DataChannel.cpp（5932）、DataReplication.cpp（2837） |
| Engine/Source/Runtime/NetworkReplayStreaming | 回放存储抽象 + LocalFile/Http/InMemory/Null/SaveGame 五实现 | Public/NetworkReplayStreaming.h:515 |

**纠正两条检索线索**：`ObjectReplicator.cpp` **不存在**——FObjectReplicator 声明于 Engine/Source/Runtime/Engine/Public/Net/DataReplication.h:73、实现全在 DataReplication.cpp；`ActorChannel.cpp` 不存在——UActorChannel 实现合并于 DataChannel.cpp（头文件 Classes/Engine/ActorChannel.h）。`PacketHandler.cpp` 不在 Engine/Private 下，在 PacketHandlers 模块。

## UNetDriver 子类清单（传输可替换性的地图，T11 展开）

- 直接继承 UNetDriver（基类：Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h:810）：UDemoNetDriver（Classes/Engine/DemoNetDriver.h:151）、UIpNetDriver（OnlineSubsystemUtils 插件 IpNetDriver.h:65，**默认驱动**，BaseEngine.ini:343-344）、UWebSocketNetDriver（WebSocketNetworking 插件 WebSocketNetDriver.h:18）、USteamSocketsNetDriver（SteamSocketsNetDriver.h:17）。
- 继承 UIpNetDriver：UGDKNetDriver、UPlayFabPartyNetDriver、USteamNetDriver、UNetDriverEOS、UDisplayClusterNetDriver、UMultiServerNetDriver（各坐标见 appendix/symbol-map.csv）。
- 默认绑定：BaseEngine.ini:343-347 · GameNetDriver/BeaconNetDriver = `/Script/OnlineSubsystemUtils.IpNetDriver`；DemoNetDriver = `/Script/Engine.DemoNetDriver`；`IrisNetDriverConfigs=(NetDriverName=GameNetDriver, bCanUseIris=true)` / `(NetDriverName=DemoNetDriver, bCanUseIris=false)`。

## 类型清单（坐标索引）

完整表见 appendix/symbol-map.csv（持续补充至各章完成）。核心条目摘要：

- 驱动/连接/通道：UNetDriver（NetDriver.h:810）、UNetConnection（NetConnection.h）、UChannel（Channel.h）、UActorChannel（ActorChannel.h，实现在 DataChannel.cpp）、UControlChannel（ControlChannel.h）、UIpConnection（OnlineSubsystemUtils/Classes/IpConnection.h）、UChildConnection、UDemoNetConnection（Classes/Engine/DemoNetConnection.h:19）。
- 复制：FRepLayout（Public/Net/RepLayout.h）、FObjectReplicator（Public/Net/DataReplication.h:73）、FRepState/FRepChangelistState（DataReplication.h）、FRepChangedPropertyTracker、FFastArraySerializer（Net/Core/Classes/Net/Serialization/FastArraySerializer.h）、UReplicationDriver（Classes/Engine/ReplicationDriver.h:48）、UReplicationGraphNode（ReplicationGraph 插件 ReplicationGraph.h:69）。
- 数据结构：FOutBunch/FInBunch（Public/Net/DataBunch.h）、FActorPriority（NetDriver.h）、FNetworkObjectInfo（Classes/Engine/NetworkObjectList.h:34）、FNetViewer、FNetworkGUID（Public/Net/NetGuid.h）。
- 枚举：ENetMode（Classes/Engine/EngineBaseTypes.h:978）、ENetRole（Classes/Engine/EngineTypes.h:3582）、ENetDormancy（EngineTypes.h:3597）、ELifetimeCondition（**CoreUObject**/Public/UObject/CoreNetTypes.h:16）、EChannelCloseReason（Public/ReplayTypes.h:22 前置声明）。

## 读取范围声明

实际打开并读取函数体/声明体的目录：Engine/Source/Runtime/Engine/{Private,Classes/Public}（网络部分）、Engine/Source/Runtime/PacketHandlers、Engine/Source/Runtime/Net/{Core,Iris}（选择性）、Engine/Source/Runtime/NetworkReplayStreaming、Engine/Plugins/{Experimental/Iris, Experimental/WebSocketNetworking, Runtime/ReplicationGraph, Runtime/NetworkPrediction, Online/*}（descriptor 与代表头文件）、Engine/Config/BaseEngine.ini。该读未读：Engine/Source/Runtime/Engine/Private/Net/Iris/* 全量（Iris 深水区只读了桥接层与关键头）；OnlineSubsystem 各平台实现；Chaos 物理仅读了 substep 相关。均在对应章标注。

## 检索日志

见 appendix/search-log.md（持续追加：关键字 → 命中/未命中 → 用在哪章）。
