# sources.md · 证据总表

> 编号规则：S-EN* 为引擎源码（本次实际打开读取），S-CFG 配置文件，S-PLG 插件描述符，「实际访问状态」= 本次会话亲读的深度（函数体/声明体/字段值）。所有路径相对 UE 源码根 `C:\Work\UE-Engine`。支撑章节指本报告 T 章。

| 编号 | 类型 | 标题或符号 | 定位 | 实际访问状态 | 支撑章节 |
|---|---|---|---|---|---|
| S-EN01 | 引擎源码 | UNetDriver / ServerReplicateActors 族 | Engine/Source/Runtime/Engine/Private/NetDriver.cpp（9467 行） | 函数体亲读：5148-6016、6274-6473、1186-1233、8125-8304 | T1/T2/T5/T7 |
| S-EN02 | 引擎源码 | UNetConnection（IsNetReady/Tick/FlushNet/Close/CleanUp/GracefulClose） | Engine/Source/Runtime/Engine/Private/NetConnection.cpp（6655 行） | 函数体亲读：2548-2756、5078-5152 等 | T2/T5/T14/T15 |
| S-EN03 | 引擎源码 | UChannel/UControlChannel/UActorChannel | Engine/Source/Runtime/Engine/Private/DataChannel.cpp（5932 行） | 关键段亲读（可靠/溢出/通道生命周期）；部分段经代理核对 | T2/T3/T7/T14 |
| S-EN04 | 引擎源码 | FRepLayout/CompareProperties/UpdateChangelistMgr | Engine/Source/Runtime/Engine/Private/RepLayout.cpp（8607 行） | 函数体亲读：1275-1404、1777-1881、6071-6125 | T4 |
| S-EN05 | 引擎源码 | FObjectReplicator | Engine/Source/Runtime/Engine/Public/Net/DataReplication.h + Private/DataReplication.cpp（2837 行） | 声明全读 + 函数体亲读 1940-2090、1430-1484、1592-1610 | T4/T7 |
| S-EN06 | 引擎源码 | FRepChangelistState/FSendingRepState/FReplicationChangelistMgr | Engine/Source/Runtime/Engine/Public/Net/RepLayout.h | 结构体与注释全读 326-630 | T4（4.1 裁决核心） |
| S-EN07 | 引擎源码 | AActor::IsNetRelevantFor/GetNetPriority/GatherCurrentMovement | Engine/Source/Runtime/Engine/Private/ActorReplication.cpp | 函数体亲读 48-92、383-424 | T5/T6 |
| S-EN08 | 引擎源码 | AActor::SetNetDormancy/FlushNetDormancy | Engine/Source/Runtime/Engine/Private/Actor.cpp | 函数体亲读 3051-3135 | T6 |
| S-EN09 | 引擎源码 | UCharacterMovementComponent 预测链 | Engine/Source/Runtime/Engine/Private/Components/CharacterMovementComponent.cpp | 函数体亲读：8606-8690、8907-9096、9967-10135、10188-10257、11223-11330 | T10 |
| S-EN10 | 引擎源码 | StatelessConnectHandlerComponent（含头注释协议文档） | Engine/Source/Runtime/Engine/Private/PacketHandlers/StatelessConnectHandlerComponent.cpp | 头注释协议图全读 + CVar 段 | T3 |
| S-EN11 | 引擎源码 | FNetworkVersion | Engine/Source/Runtime/Core/Private/Misc/NetworkVersion.cpp | 函数体亲读 88-152、223-314 | T3 |
| S-EN12 | 引擎源码 | UWorld::NotifyControlMessage / UPendingNetGame / 登录族 | Engine/Source/Runtime/Engine/Private/World.cpp、PendingNetGame.cpp | 经代理逐 case 核对（行号抽验） | T3 |
| S-EN13 | 引擎源码 | UWebSocketNetDriver / UWebSocketConnection | Engine/Plugins/Experimental/WebSocketNetworking/Source/WebSocketNetworking/ | 头文件全读 + 实现亲读（16-179、88-222） | T11 |
| S-EN14 | 引擎源码 | Iris：ReplicationSystem/ReplicationStateDescriptor/DataStream/EngineReplicationBridge/DataStreamChannel | Engine/Source/Runtime/Net/Iris/Public/** + Engine/Private/Net/Experimental/Iris/ | 头文件结构与关键段亲读；实现体未逐函数（见 Known gaps） | T8 |
| S-EN15 | 引擎源码 | ReplicationGraph 插件 | Engine/Plugins/Runtime/ReplicationGraph/Source/{Public,Private}/ReplicationGraph.{h,cpp} | 头注释与结构全读；cpp 关键函数定位+调用链 | T6 |
| S-EN16 | 引擎源码 | NetworkPrediction 插件 | Engine/Plugins/Runtime/NetworkPrediction/Source/NetworkPrediction/ | 配置/设置头全读；WorldManager 关键函数经代理核对 | T9/T10 |
| S-EN17 | 引擎源码 | DemoNetDriver/ReplayHelper/ReplaySubsystem/INetworkReplayStreamer | Engine/Source/Runtime/Engine/Private/{DemoNetDriver,ReplayHelper,ReplaySubsystem}.cpp + NetworkReplayStreaming/** | 经代理核对（关键坐标抽验通过） | T13 |
| S-EN18 | 引擎源码 | UWorldPartition 服务器流送 | Engine/Source/Runtime/Engine/Private/WorldPartition/WorldPartition.cpp | 函数体亲读 128-138、1384-1458 | T12 |
| S-EN19 | 引擎源码 | UGameEngine/UEngine tick 率族 + GameStateBase 时间同步 + PhysicsSettings | GameEngine.cpp / UnrealEngine.cpp / GameStateBase.cpp / PhysicsSettings.cpp | 经代理核对（关键坐标抽验） | T9 |
| S-EN20 | 引擎源码 | ENetCloseResult 族 | Engine/Source/Runtime/Net/Core/Public/Net/Core/Connection/NetCloseResult.h | 经代理全表核对 | T14 |
| S-EN21 | 引擎源码 | PushModel / FastArraySerializer | Engine/Source/Runtime/Net/Core/** | 注册点亲读 + 头注释全读 | T4 |
| S-EN22 | 引擎源码 | ELifetimeCondition / EChannelCloseReason / ENetDormancy / ENetRole / ENetMode | CoreNetTypes.h / EngineTypes.h / EngineBaseTypes.h | 枚举体全读 | T1/T4/T6 |
| S-EN23 | 引擎源码 | 观测设施（EngineStats/EngineLogs/NetTraceReporter） | Engine/Public/EngineStats.h、EngineLogs.h + Net/Core/.../NetTraceReporter.cpp | 经代理全表核对 | T16 |
| S-CFG01 | 配置 | BaseEngine.ini（网络相关段） | Engine/Config/BaseEngine.ini | 亲读 343-347、1838-1871 | T0/T5/T9 |
| S-CFG02 | 配置 | BaseGame.ini（GameNetworkManager 段） | Engine/Config/BaseGame.ini | 经代理核对 | T10/T16 |
| S-PLG01 | 插件描述符 | Iris / NetworkPrediction / ReplicationGraph / WebSocketNetworking / OnlineSubsystem(Null) / 各 NetDriver 插件 | Engine/Plugins/**.uplugin | 逐个打开读字段 | T0 |
| S-GIT01 | 版本记录 | git HEAD ff8421f2b（branch 5.8）+ Build.version | .git / Engine/Build/Build.version | 亲读 | 全部 |

外部文档：本次未使用任何官方文档/社区帖子作为论断依据（0 条 Verified-Doc）——全部结论来自上表源码亲读或明确标注的 Estimated/Reported。
