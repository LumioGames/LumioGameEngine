# search-log.md · 检索日志

工具：Grep 工具（ripgrep）+ 定位后 Read 按行读。引擎根 `C:\Work\UE-Engine`（UE 5.8.2）。仅记有裁决价值的检索；命中=进入正文/CSV 的条目。

## 1. 环境发现

| 操作 | 结果 |
|---|---|
| `Get-ChildItem C:\Work -Recurse -Depth 5 -Filter '2026-08-29-ue-dedicated-server'` | **0 命中**——第一波目录不在本机（R8 约束自动满足；T17 以提示词所引预研论断为基准） |
| `docs/research` 全盘搜索（LumioGames/LumioAgent/LumioAPI） | 0 命中 → ARCH_REPO 判定为 LumioGameEngineArchitecture（架构源仓，与画像吻合），已向用户口头确认前先以目录树建仓 |
| 读 `LumioGameEngineArchitecture/docs/plans/2026-08-28-kickoff-dispatch-prompts.md` | 七仓派单文档，确认架构仓地位 |

## 2. 亲读定位检索（关键命中）

| 关键字 | 范围 | 命中 → 用途 |
|---|---|---|
| `ServerReplicateActors` | NetDriver.cpp | 24 处 → T5 全链（5148-6016/6274-6473） |
| `QueuedBits -=/+= / =` | Engine/Private | 5 处 → T5.9 token bucket |
| `FActorPriority::FActorPriority` / `RelevantTimeout =` / `SpawnPrioritySeconds` | NetDriver.cpp+h | 优先级公式与滞回默认值 |
| `ConfiguredInternetSpeed` | Engine/Config | BaseEngine.ini:1839=100000 |
| `IsNetRelevantFor` / `NEARSIGHTTHRESHOLDSQUARED` | Engine/Private + Public | ActorReplication.cpp:388-419 + NetworkingDistanceConstants.h 全文 |
| `SetNetDormancy / FlushNetDormancy` | Engine/Private | Actor.cpp:3051-3135 |
| `struct FNetworkObjectInfo` | Engine/Classes | NetworkObjectList.h:34 |
| `FRepChangelistState / FReplicationChangelistMgr / FRepState` | RepLayout.h | 326-780 结构体全读（4.1 裁决） |
| `FReplicationChangelistMgr` | Engine/Private | DataReplication.cpp:126/724/728 + NetDriver.cpp:7917（归属钉死） |
| `UpdateChangelistMgr` / `CompareProperties` | RepLayout.cpp | 1275-1335 / 1777-1881 |
| `IsPushModelEnabled / MARK_PROPERTY_DIRTY` | Net/Core | PushModel.cpp:434-446 + PushModel.h |
| `enum ELifetimeCondition : int` | Engine/Source/Runtime | CoreNetTypes.h:16（真身在 CoreUObject，不在 Engine） |
| `ReplicateSubobjects` | DataChannel.cpp | 4007/4224 |
| `Net.MaxRPCPerNetUpdate`（命名）| DataReplication.cpp | 38-42（真名带 NetUpdate 非 Frame） |
| `RPC_ValidateFailed` | Engine/Private + CoreUObject | CoreNet.cpp:667 + DataReplication.cpp:1465（校验失败=断连链） |
| `ClientUpdatePositionAfterServerUpdate` | Engine/Runtime | CharacterMovementComponent.cpp:8606 |
| `WINSOCK_MAX_PACKET / UDP_HEADER_SIZE` | Sockets 无 → 插件内 | WebsocketConnection.cpp:16-17（伪 UDP 记账） |
| `GlobalEnableServerStreaming` | WorldPartition/ | WorldPartition.cpp:128-138（默认 0） |
| `IsServerStreamingEnabled` | WorldPartition.cpp | 1384-1458 |
| `net.MaxTickRate`（验证不存在） | Engine/Private 全 Net 文件 | 0 命中（真名 NetServerMaxTickRate，GameEngine.cpp:1745 消费） |
| `SpecControlChannel` | Engine/Source/Runtime | 0 命中（线索作废，T13） |
| `IDemoNetworkStream / FDemoFileWriter` | Engine/Runtime | 0 命中（真身 INetworkReplayStreamer，T13） |
| `MAX_PARTIAL_BUNCH_COUNT` | Engine 全树 | 0 命中（由 512+64KB 间接约束，T14） |

## 3. 代理检索（枚举类任务，行号已抽验）

| 任务 | 代理完成的关键检索 | 抽验 |
|---|---|---|
| NMT 全表 | `NMT_Hello` 定位 DataChannel.h:154-158 宏族；三处 NotifyControlMessage 分支 | 抽验 `bSkipServerReplicateActors`(NetDriver.h:1245)、`LowLevelSend`(DemoNetDriver.cpp:4872) 通过 |
| 关闭原因/上限 | `Close(`/`CleanUp(` 于 NetConnection/DataChannel/NetDriver；`RELIABLE_BUFFER/MAX_QUEUED_CONTROL_MESSAGES` 等 | 抽验 RELIABLE_BUFFER=512（NetConnection.h:82）通过 |
| CVar/ini | `TAutoConsoleVariable` net.* 族 + `UPROPERTY(Config)` 于 IpNetDriver/NetConnection/GameNetworkManager | 抽验 Net.IsPushModelEnabled 与 BaseEngine.ini:1867 通过 |
| 回放 | Demo/ReplayHelper 文件定位 + `checkpoint` 函数族 | 两次抽验通过（见上） |
| 断线生命周期 | `ReconnectToken/ResumeToken/SessionResume/ReconnectSessionId/bReconnecting` 全树 0 命中记录；`CopyProperties` 调用点 | 结构自洽，坐标进入 T15 |
| 时间系统 | `GetMaxTickRate` 链 / `ETickingGroup` / `bSubstepping` / `ServerWorldTime`（PlayerState 0 命中→GameStateBase） | 时间同步纠正（CW-04） |

## 4. 约束遵守记录

- 全文未粘贴超过 10 行源码（最长引用为 StatelessConnect 头注释要点转述，带坐标）。
- 未创建任何源码文件副本；未执行 git add/commit/push；未改动 docs/research/README.md（本仓库本无此文件）与第一波目录。
- 引擎源码 git 仓库未做任何写操作（全部读取；.claude/settings.local.json 为既有未跟踪文件，未触碰）。
