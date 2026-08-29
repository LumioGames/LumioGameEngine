# appendix/diagrams.md — 两张硬指标图（mermaid 源码）

## 图 1 · 连接建立时序（每框标函数与坐标，UE 5.8.2）

```mermaid
sequenceDiagram
    participant C as 客户端 (UPendingNetGame)
    participant S as 服务器 (UIpNetDriver + UWorld)

    Note over C: WebSocket/UdpNetDriver InitConnect<br/>PendingNetGame.cpp
    C->>S: 初始握手包 [MagicHeader/SessionID/ClientID/NetworkVersion/Features]<br/>StatelessConnectHandlerComponent.cpp:41-45 协议图
    Note over S: 无状态校验(不分配内存)<br/>cookie = HMAC(HandshakeSecret, Timestamp+IP+Port) :146
    S->>C: Stateless Handshake Challenge [Timestamp+Cookie] :47-51
    C->>C: CreateInitialClientChannels(假控制通道)<br/>NetDriver.cpp:8279-8291
    C->>S: Challenge Response [回传 Timestamp+Cookie] :53-57
    Note over S: 重算 cookie 比对+时间窗校验 :148-159<br/>通过→创建 UNetConnection
    S->>C: Stateless Handshake Ack :62-66
    C->>S: NMT_Hello [字节序/NetworkVersion/EncryptionToken/Features]<br/>DataChannel.h:173; World.cpp:7369
    alt 版本不匹配 (IsNetworkCompatible 精确相等 NetworkVersion.cpp:263-271)
        S->>C: NMT_Upgrade 或 NMT_Failure [错误串]<br/>World.cpp:7397 / NetDriver.cpp:3329
        Note over C: Close(OutdatedClient) NetConnection.cpp:1372
    else 版本匹配
        S->>C: NMT_Challenge [Challenge 串]<br/>NetConnection.cpp:6192-6193 (ExpectedClientLoginMsgType=NMT_Login)
        C->>S: NMT_Netspeed [Rate] (World.cpp:7465→Clamp 1800..MaxClientRate PlayerController.cpp:539)
        C->>S: NMT_Login [ClientResponse/URL/PlayerId/Platform]<br/>World.cpp:7485 (载荷上限临时+16MB :7492-7498)
        Note over S: AGameSession::ApproveLogin (容量拒绝 "Server full.")<br/>AGameModeBase::PreLogin GameModeBase.cpp:690/715
        S->>S: GameMode Login → SpawnPlayerController<br/>(AGameModeBase::Login 家族)
        S->>C: NMT_Welcome [Map/GameName/RedirectURL] World.cpp:7186
        C->C: 加载地图 (UPendingNetGame → 正式 UNetConnection)
        C->>S: NMT_Join [无载荷] World.cpp:7571
        S->>S: PostLogin; PlayerController 通道打开<br/>(UActorChannel::SetChannelActor)
        Note over C,S: 进入常规复制循环(图 2)
    end
```

## 图 2 · 服务器复制循环（经典路径，每框标函数与行号）

```mermaid
flowchart TD
    A[UNetDriver::TickFlush<br/>NetDriver.cpp:1168-1233] -->|IsServer && 有连接 && !bSkipServerReplicateActors :1186| B{ReplicationSystem?<br/>:1188}
    B -->|Iris| B1[InternalIrisUpdateTransactional :1212-1215]
    B -->|经典/RepGraph| C[UNetDriver::ServerReplicateActors<br/>NetDriver.cpp:6277-6473]
    C -->|ReplicationDriver 存在 :6295| C1[UReplicationGraph::ServerReplicateActors<br/>ReplicationGraph.cpp:1112]
    C --> D[ServerReplicateActors_PrepConnections :5198-5301<br/>连接节流/1.5s 无收视为不 ready/ViewTarget 设定]
    D -->|0 连接 ready| Z[return 0]
    D --> E[ServerReplicateActors_BuildConsiderList :5303-5455<br/>频率门/自适应 OptimalNetUpdateDelta/PreReplication 每对象一次]
    E --> F[每连接循环 :6384-6463]
    F -->|i>=NumClientsToTick| F1[打 bPendingNetUpdate 留待下帧 :6412-6428]
    F -->|ViewTarget 空| F2[跳过]
    F --> G[ServerReplicateActors_ForConnection :5938-6016<br/>构建 FNetViewer/SendClientAdjustment 每包一次 :5977-5990]
    G --> H[ServerReplicateActors_PrioritizeActors :5528-5679<br/>无通道者过 Level 门+IsNetRelevantFor :5582-5592<br/>休眠门 :5618-5634/FActorPriority 构造 :5152-5162/降序排序 :5669]
    H --> I[ServerReplicateActors_ProcessPrioritizedActorsRange :5687-5894]
    I -->|!Connection->IsNetReady 预算门1 :5695| I0[return 0 GNumSaturatedConnections++]
    I --> I1[relevancy 重查(~1Hz 节流 :5750)/滞回判定 :5773<br/>bIsRecentlyRelevant]
    I1 -->|相关| I2[通道不存在→创建 :5789-5798<br/>Channel->ReplicateActor :5830<br/>自适应频率回写 :5849-5856]
    I2 -->|Channel 饱和| I3[Actor->ForceNetUpdate :5865]
    I2 -->|!Connection->IsNetReady 预算门2 :5868| I4[return j ←截断点]
    I1 -->|失相关超滞回| I5[Channel->Close Relevancy/TearOff :5877-5889]
    I4 --> J[ServerReplicateActors_MarkRelevantActors :5896-5934<br/>未处理区打 bPendingNetUpdate → 回流 E]
    J --> F
    I2 --> K[FObjectReplicator::ReplicateActor 属性段<br/>DataReplication.cpp:1940-2090]
    K --> K1[UpdateChangelistMgr 共享比较复用<br/>RepLayout.cpp:1275-1335 → CompareProperties :1777+]
    K1 --> K2[ReplicateProperties 序列化增量 :2016<br/>+ CustomDelta/FastArray :2033]
    K2 --> L[UChannel::SendBunch<br/>DataChannel.cpp:1249-1447 分片/可靠判定/溢出断连 :1414-1445]
    L --> M[UNetConnection::FlushNet 组包<br/>NetConnection.cpp:2562-2598 QueuedBits 扣减]
    M --> N[LowLevelSend → socket/WebSocket]
```
