# T13 · 回放：网络序列化 = 录像序列化

> 本章基于 UE 5.8.2（git ff8421f2b）源码直接读取；关键坐标经二次抽查确认。除标注外均为 Verified-Src。

## 结论先行

1. **「录制即复制」的字面实现是：DemoNetDriver 关掉通用复制主循环（`bSkipServerReplicateActors = true`），自己跑一套并行的 ServerReplicateActors 变体，把「发给假连接的包」从 socket 出口劫持进文件**——复制管线（ActorChannel/RepLayout/PackageMap）原样复用，被替换的只有最底层的 `LowLevelSend`（Engine/Source/Runtime/Engine/Private/DemoNetDriver.cpp:4872 · `UDemoNetConnection::LowLevelSend`）。
2. **检查点 = 「用既有通道把全部属性以 SinceOpen 状态重录一遍」**，不是引擎级世界快照（Engine/Source/Runtime/Engine/Private/ReplayHelper.cpp:574-794 · `FReplayHelper::SaveCheckpoint`/`TickCheckpoint`）；默认每 30s 触发（`demo.CheckpointUploadDelayInSeconds`），按帧预算摊销跨帧写入。
3. **5.8 存在两代回放并存**：旧 `UDemoNetDriver`（独立 NetDriver、禁用 Iris）与新 `UReplayNetConnection`（挂在真实 game NetDriver 上的一条假客户端连接，配合 ReplicationGraph/Iris，开关 `Replay.UseReplayConnection` **默认 false**）。不存在 "ReplaySystemNext" 插件（Engine/Plugins 全树检索 0 命中）——预研若提及该名，属源码中不存在。

## 机制正文

### 13.1 录制路径

- `UDemoNetDriver::InitDefaults` 置 `bSkipServerReplicateActors = true`（DemoNetDriver.cpp:732；标志声明 Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h:1245；门控在 NetDriver.cpp:1186 TickFlush）→ 游戏驱动的 ServerReplicateActors 不再运行。
- 驱动链：`TickFlush`（DemoNetDriver.cpp:1291）→ `TickFlushInternal`（:1330，先 Super 再 `TickDemoRecord` :1380）→ `TickDemoRecord`（:1747）→ `TickDemoRecordFrame`（:1826）→ `ReplicatePrioritizedActor(s)`（:2243/:2384）。
- 伪造连接：`InitListen`（:1122，注释 "demo stream acts 'as if' it's a client"）创建 `UDemoNetConnection`（:1142）并生成观战 PlayerController（:1156）；通道定义在 Engine/Config/BaseEngine.ini:2027-2033（`[/Script/Engine.DemoNetDriver]` 段）。
- 落盘：`UDemoNetConnection::LowLevelSend`（:4872）把包塞进 `ReplayHelper.QueuedDemoPackets`（检查点期间进 `QueuedCheckpointPackets`，:4896-4898），由 `WriteDemoFrameFromQueuedDemoPackets`（:2639）写入 `INetworkReplayStreamer::GetStreamingArchive()`。
- **不是所有 actor 都录**：只遍历复制注册表；过滤 `RemoteRole==ROLE_None && !TearOff`（:2006）、初始休眠（:2012）、`!bRelevantForNetworkReplays`（:2018，AActor 默认 true，Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h:492）；`demo.UseNetRelevancy`（默认 0）开启时才按真实客户端视点做距离剔除（:1951-1964），不相关对象按 `demo.RecordHzWhenNotRelevant`（默认 2Hz）降频（:2048-2053）。multicast RPC 默认录制（`ProcessRemoteFunction` :1511），unicast 默认不录（私有开关 `RecordUnicastRPCs` :139）。

### 13.2 检查点与恢复

- 触发：`demo.EnableCheckpoints`（默认 1）+ `DemoCurrentTime - LastCheckpointTime > 30s`（ReplayHelper.cpp:2111 · `ShouldSaveCheckpoint`）；跨帧摊销预算 `demo.CheckpointSaveMaxMSPerFrameOverride`（默认 -1 → 用 DemoNetDriver.h:294-299 的默认）。
- 内容：只收「有 open channel 或休眠中」的 actor（ReplayHelper.cpp:640-648）；对每个 actor 用 `EResendAllDataState::SinceOpen` 重录全量属性（:794 注释明说复用既有连接）；外加 NetGuidCache、已删除 startup actors、NetFieldExport。状态机 `ECheckpointSaveState`（ReplayHelper.h:259-268）在 `TickCheckpoint`（ReplayHelper.cpp:827）中按预算跨帧推进。Delta 检查点开关 `demo.WithDeltaCheckpoints`（DemoNetDriver.cpp:85）。
- 恢复：`GotoTimeInSeconds`（DemoNetDriver.cpp:2672）→ streamer `GotoTimeInMS` → `LoadCheckpoint`（:4013）：销毁非 startup actor（:4171-4223，`bReplayRewindable` 除外）→ 重建假连接与控制通道（:4281-4288）→ 重读 guid/deleted/field exports（:4386-4490）→ 一次性灌入检查点包（:4595）→ `SkipTimeInternal` 快进（:4548-4553）。快进期间 RPC 默认丢弃（`demo.FastForwardIgnoreRPCs` 默认 1，:4654-4657）。

### 13.3 存储抽象（引擎给的接口面）

`INetworkReplayStreamer`（Engine/Source/Runtime/NetworkReplayStreaming/NetworkReplayStreaming/Public/NetworkReplayStreaming.h:515）：`StartStreaming`/`GetHeaderArchive`/`GetStreamingArchive`/`GetCheckpointArchive`/`FlushCheckpoint`/`GotoTimeInMS`/`IsCheckpointTypeSupported(Full/Delta)`/事件检索（`AddEvent`/`EnumerateEvents`/`SearchEvents`）/`KeepReplay`/`EnumerateStreams` 等。实现模块：LocalFile / Http / InMemory / Null / SaveGame（Engine/Source/Runtime/NetworkReplayStreaming/ 下五个并列目录）。`IDemoNetworkStream`/`FDemoFileWriter` 等旧名在本版本 0 命中——检索线索作废。

### 13.4 限制的源码注释证据（每条挂坐标）

- **只能录复制出去的东西**：`bRelevantForNetworkReplays`（Actor.h:492）+ ShouldReplicateActor（DemoNetDriver.cpp:5526-5529）；服务器内部状态（没走复制管线的）天然不可录。
- **dormancy 抬高录制成本**：DemoNetDriver.cpp:5532-5546 `AdjustConsiderTime` 注释直说「休眠 actor 最坏情况下大量堆积、人为推高 consider/sort 时间」。
- **guid 复用假设可被打断**：DemoNetDriver.cpp:4175-4179 —— 检查点恢复假定 actor 复用同一 NetGUID，「录制期间销毁又以不同 guid 重建的 actor 会打破该假设」；Rewindable actor 出现在删除表时只能告警（:4523-4528 "Replay may show artifacts"）。
- **跨包/跨地图**：`ProcessSeamlessTravel`（:4096）；`RollbackNetStartupActors` 与 DeletedNetStartupActors 冲突时销毁优先（DemoNetDriver.h:196-199）。
- **与确定性无关**：整套回放建立在「重放收到的包」上，与引擎 tick 是否确定、状态可否哈希完全无关——这是与目标引擎「确定性重放」路线的根本差异。

### 13.5 两代系统并存（5.8 现状）

| | 旧：UDemoNetDriver | 新：UReplayNetConnection |
|---|---|---|
| 形态 | 独立 NetDriver + 假连接 | 真实 game NetDriver 上的一条假 client connection（ReplaySubsystem.cpp:148-159） |
| 复制执行 | 自己的 Prioritize/Replicate 循环 | 游戏 ReplicationDriver（RepGraph/Iris）照常跑 |
| 开关 | 默认路径 | `Replay.UseReplayConnection`，**默认 false**（ReplaySubsystem.cpp:16） |
| 回放侧 | 仍是 DemoNetDriver（PlayReplay 一律走它，ReplaySubsystem.cpp:224-264） | 仅录制侧替换 |
| Iris | 明确禁用（BaseEngine.ini:347 `bCanUseIris=false`） | 为 Iris/RepGraph 而生 |

## 意外发现

- `MAX_DEMO_READ_WRITE_BUFFER` 超限直接 `Fatal`（DemoNetDriver.cpp:4891）——单个 demo 包超缓冲即崩溃级断言。
- 检查点期间录制的包与正常流分队列（:4896-4898），避免快进语义污染。
- `SpecControlChannel` 检索线索在本版本 0 命中（已不存在）。

## 对目标环境的迁移含义

对一个已有「确定性 tick + 状态哈希 + WAL」的引擎，UE 这条「复用复制流做录像」的路**不建议照抄**：它的全部复杂度（检查点=全量属性重录、guid 复用假设、dormancy 交互、快进丢 RPC）都源于「只录线路流量、没有权威状态可快照」。目标引擎每帧已有可哈希的提交点状态——录像 = 记录输入流/命令流 + 定期全量状态快照（等价于 UE 检查点想模拟而不得的东西），恢复 = 从最近快照重放确定性逻辑。真正值得借鉴的只有两点：(1) **存储抽象做成 streamer 接口**（NetworkReplayStreaming.h:515 的接口面：流/检查点/事件检索三分）；(2) **按帧预算摊销检查点写入**（ReplayHelper 的跨帧状态机）——快照落盘永远不能阻塞模拟 tick。
