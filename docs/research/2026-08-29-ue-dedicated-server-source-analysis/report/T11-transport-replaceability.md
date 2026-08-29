# T11 · 传输可替换性：剥掉原生 UDP 之后还剩什么（对目标环境最关键章）

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **UE 的网络栈在「连接/通道/bunch/复制」四层是传输无关的，在「包」层是 UDP 形状的**。证据是双面的：`UWebSocketNetDriver`/`UWebSocketConnection` 只替换了最底层（InitBase/LowLevelSend/TickDispatch/ReceivedRawPacket，WebSocketNetDriver.h:26-40、WebSocketConnection.h:18-28）就跑通了全部上层；但 WebSocket 连接**把 MaxPacket 默认设为 512 并记账 IP+UDP 头开销**（WebSocketNetworking/Private/WebsocketConnection.cpp:16-17、30-31：`UDP_HEADER_SIZE=(IP_HEADER_SIZE+8)`、`WINSOCK_MAX_PACKET=512`）——上层对「我有数据报语义」的假设深到字节预算。
2. **WebSocket 驱动是「在可靠有序流上重演 UDP 协议」的活体标本**：PacketHandler 链、无状态握手（`bChallengeHandshake` → `ConnectionlessHandler->IncomingConnectionless` → `StatelessConnect->HasPassedChallenge`，WebsocketConnection.cpp:162-179）、包序号/ack/NAK/重传整套原样运行（连接继承 UNetConnection 全部机制），尽管底层 TCP 已经保证交付。冗余但正确；代价是队头阻塞叠加双层重传超时。
3. **逐层裁决**（详见 11.3 表）：可原样继承 4 层（复制内核/通道/业务协议/控制消息），必须重造 2 层（包层可靠性、拥塞/速率控制），1 层（加密）在 TLS 之上变冗余。UE 的 `LowLevelSend` 是**按地址路由**的连接无关接口（WebSocketNetDriver.cpp:124-166），而浏览器 API 天然是「按连接的流」——这里存在一次必须的接口倒置。

## 11.1 UNetDriver 的抽象边界：哪些传输无关、哪些假设了数据报

传输无关的接口面（UNetDriver 层）：`InitBase/InitConnect/InitListen`、`TickDispatch/DropShip`、`LowLevelSend(Address, Data, CountBits, Traits)`、`LowLevelGetNetworkNumber/LowLevelDestroy`、`IsNetResourceValid`、`GetSocketSubsystem`（WebSocket 版为 stub，WebSocketNetDriver.h:37-38 注释 "stub implementation because for websockets we don't use any underlying socket sub system"）。子类清单见 T0（UIpNetDriver 之下还有 GDK/PlayFabParty/Steam/EOS/DisplayCluster/MultiServer 一整族——传输可替换性被大规模验证过）。

假设了数据报语义的地方（逐条挂坐标）：

| 假设 | 坐标 | 内容 |
|---|---|---|
| 包=自包含数据报 | NetConnection.cpp:3823（DispatchPacket 按位长读 bunch）；NetConnection.h:85-89（MAX_PACKET_HEADER_BITS 等） | 每个 SendBuffer flush 即一个「包」，有独立序号/ack |
| 包会丢、会乱序 | FPacketNotify 体系（NetConnection.cpp:3420-3429 AckSequenceMismatch 等）；`net.PacketOrderMaxMissingPackets=3`/`net.PacketOrderMaxCachedPackets=32`（NetConnection.cpp:91-97，乱序纠正缓存） | 可靠性/排序全部自建在这上面 |
| MTU/包尺寸上限 | MAX_PACKET_SIZE=1024（CoreNet.h:794）；MaxPacket 每连接协商后 clamp | 512~1024 字节的包是 bunch 分片、可靠队列、部分 bunch 全部设计的尺寸基础 |
| 每包 ACK/NAK | ReceivedAck/ReceivedNak（NetConnection.cpp:2761 起） | 逐包反馈驱动重传 |
| 地址=IP:Port | LowLevelSend 首参 FInternetAddr；WebSocket 版用字符串比对找连接（WebSocketNetDriver.cpp:151-158） | 连接定位靠地址 |

## 11.2 WebSocket 驱动逐层复用/绕过裁决（亲读）

- **复用**：UNetConnection 全部（通道表、OutRec/InRec 可靠队列、bunch 重组、PacketHandler 链、StatelessConnect 握手、控制消息、复制——Super::InitBase 之后一切照旧，WebsocketConnection.cpp:25-61）；驱动层仅替换传输与分发（WebSocketNetDriver.cpp:88-166）。
- **替换点**：`LowLevelSend` = `WebSocket->Send(bytes)`（一包一条二进制消息，WebsocketConnection.cpp:98-101）；收包回调进 `ReceivedRawPacket`（:149-179，先过无状态握手判定）；`Tick` 额外泵 `WebSocket->Tick()`（:130-134）；服务器接受连接在 `OnWebSocketClientConnected`（WebSocketNetDriver.cpp:211-222，仍走 `Notify->NotifyAcceptingConnection` 准入门）。
- **性能/健壮性短板（源码可见）**：驱动级 `LowLevelSend` 对 ClientConnections 做**线性扫描 + 地址字符串全串比较**（:151-158，注释自认 "connectionless websockets do not exist (yet)"）；MaxPacket=512 比默认 UDP 1024 更小；插件标记 Experimental（T0）。
- **byte 层模拟 UDP**：512 包上限 + 幻想出的 IP/UDP 头开销记账（WebsocketConnection.cpp:16-17）——上层的「带宽预算按包计」逻辑因此在 WebSocket 上依旧成立，代价是效率。
- 传输后端：libwebsockets / WinHttp（WebSockets 模块双后端，见 T0）；WebSocketNetworking 自带独立的服务器实现（WebSocketServer.cpp）。

## 11.3 换传输后的逐条裁决（WebSocket 有序可靠流 / WebTransport 多流）

| UE 机制 | WebSocket（有序可靠单流） | WebTransport（多流可靠+不可靠数据报） | 坐标 |
|---|---|---|---|
| 通道抽象（UChannel/ActorChannel/ControlChannel） | **变冗余但不塌**：通道保序可改由流保序承载；每通道独立流的映射自然 | **变简单**：每通道一条WT流即得独立保序+独立背压 | DataChannel.h 通道族 |
| bunch 分片（partial bunch，≤64KB 重组） | 变简单：消息边界免分片重组（但保留尺寸上限防队头垄断） | 同左 | DataChannel.cpp:1296-1401 |
| 包序号/ack/NAK/重传 | **纯冗余**（TCP 已交付）；但开箱可跑——WebSocket 驱动即证 | 不可靠数据报流上仍必要（保留）；可靠流上冗余 | NetConnection.cpp:2761+ |
| 可靠队列 OutRec/InRec（RELIABLE_BUFFER=512，溢出断连） | 冗余且有害：TCP 队头阻塞会让 512 深度假队列超时误断（见 T14 溢出路径在慢链路上的风险） | 可靠流上冗余 | DataChannel.cpp:1414-1445 |
| 带宽 token bucket（QueuedBits） | **仍然必要**（发送侧限速与 TCP 拥塞控制正交，否则上行突发在 TCP 缓冲里排队放大延迟） | 仍然必要 | NetConnection.cpp:5112-5145 |
| 拥塞控制（net.EnableCongestionControl 默认关） | **必须重造**：TCP 拥塞不告诉你应用层该降什么频；UE 的复制降频逻辑不知道 TCP RTT 膨胀 | WT 有传输层拥塞信号，但仍需映射到「降谁」 | NetConnection.cpp:105、2745-2748 |
| StatelessConnect 握手（HMAC cookie） | 冗余（WSS 已完成 TLS+源验证），但 UE 在 WebSocket 上仍跑了它——证明可保留 | 数据报路径上仍有价值 | WebsocketConnection.cpp:162-179 |
| 加密（PacketHandler EncryptionComponent，net.AllowEncryption） | **变冗余**：TLS 已覆盖机密性/完整性 | 同左 | NetDriver.cpp:529 |
| MTU 假设（MAX_PACKET_SIZE=1024） | 失效：消息可更大，但 UE 分片/预算按 1024 写死 → 需参数化 | 数据报路径仍有真 MTU | CoreNet.h:794 |
| 低延迟不可靠复制 | **塌掉**：TCP 上无「丢弃旧状态」选项——不可靠属性复制变可靠有序，恶化拥塞 | **正好匹配**：不可靠数据报=UE 原生 UDP 语义 | — |

**裁决题**：浏览器客户端 + WebSocket/WebTransport 的引擎，能从 UE **原样继承**：复制内核（changelist/条件掩码/量化思想，T4）、通道-业务协议分层（控制消息序列、T3）、调度思想（T5 的预算/优先级/回流，T6 的滞回）。**必须重造**：包层（可靠性是否自建取决于传输是否给不可靠数据报；WT 数据报下仍需 ack 序号）、拥塞→复制降频的映射（UE 缺失，浏览器下更缺）、按连接的发送接口（LowLevelSend 的地址路由 → 连接句柄路由）、以及鉴权前限流（浏览器下伪造 WSS 升级比伪造 UDP 源更容易被 CDN 挡，但应用层仍需 token 限连）。

## 11.4 包大小/MTU 假设常量全表

| 常量 | 值 | 坐标 |
|---|---|---|
| MAX_PACKET_SIZE | 1024 字节 | CoreUObject/Public/UObject/CoreNet.h:794 |
| MaxPacket（连接级，协商+clamp） | ≤1024；WebSocket 默认 512 | NetConnection.cpp:687、WebsocketConnection.cpp:17 |
| MAX_BUNCH_HEADER_BITS | 256 | NetConnection.h:85 |
| GetMaxSingleBunchSizeBits | MaxPacket×8 − 256 − 包头 − HandlerBits | NetConnection.h:1361-1364 |
| net.MaxConstructedPartialBunchSizeBytes | 65536 | DataChannel.cpp:105-110 |
| RELIABLE_BUFFER | 512 | NetConnection.h:82 |
| 回放例外 | MAX_REPLAY_PACKET=2048 | ReplayNetConnection.cpp:16 |

## 意外发现

1. WebSocket 驱动把 UDP 头开销记进带宽账本（WebsocketConnection.cpp:16-17）——**协议考古层级的兼容**：为了不动上层的统计与预算公式，宁可记账不存在的头。
2. 驱动级按地址线性扫描发路径（WebSocketNetDriver.cpp:151-158）没有索引表——Epic 从没打算让这个 Experimental 插件承载规模。
3. `EDataStreamWriteMode::DebugData`（DataStream.h:22-32，Iris）：「不计入带宽限额的调试数据」在流抽象里是一等公民——可观测性进协议的范例。
4. UE 的 PacketHandler 链对「连接前」流量有独立实例（`ConnectionlessHandler`，WebSocketNetDriver.cpp:95 `InitConnectionlessHandler`）——无连接包与连接包走同一组件栈，这是无状态握手能跨传输复用的结构原因。

## 对目标环境的迁移含义

目标引擎（Rust 宿主 + 浏览器客户端）应把 UE 的教训压缩成三条设计令：(1) **传输接口面向「带交付语义的流/报」而非「地址」**——UE 的 LowLevelSend(Address) 在浏览器世界必须倒置成 ConnectionHandle，且接口上显式声明每通道的交付语义（可靠有序/不可靠），让 WebTransport 的多流直接映射 UE 的「通道」概念，WebSocket 则退化为单通道+自建不可靠层；(2) **应用层速率控制与传输拥塞控制解耦并显式交换信号**（UE 两层互不感知，`net.EnableCongestionControl` 默认关是自白）；(3) **所有包尺寸常量参数化进 Schema/Profile**（UE 的 1024/512/64KB 编译期写死，浏览器路径的 MTU 语义完全不同）。加密层直接砍掉（TLS 强制），把省下的复杂度预算花在「不可靠数据报上的 ack 窗口」与「WSS 升级风暴限流」。
