# T2 · NetDriver / NetConnection / Channel / Bunch 逐层

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **三层所有权**：UNetDriver 拥有连接列表与共享复制状态（RepLayoutMap/ReplicationChangeListMap，T4）；UNetConnection 拥有通道数组（Channels[]，静态索引 0=控制通道）、发送缓冲（SendBuffer）、可靠队列（OutRec/InRec）与带宽账本（QueuedBits）；UChannel 拥有 bunch 层（partial bunch 重组、OutgoingBunches）。生命周期：连接由驱动 CreateClientConnection/RemoveClientConnection 管理，销毁走 CleanUp（T15 全链）。
2. **可靠性建在 bunch 层而不是 packet 层，换来的确切能力**：①**部分可靠**（partial bunch 拆分时按 `net.PartialBunchReliableThreshold`=8 决定整段是否转可靠，DataChannel.cpp:1416-1429）——包级可靠性做不到「一个逻辑消息跨多包仍整体可靠」；②**按通道独立保序**（每通道自己的 OutRec/InRec 序号空间，一个通道堵塞不冻结其他通道的投递）；③**精确重发**（changelist 记录 OutPacketIdRange，NAK 时只重发受影响 bunch，RepLayout.h:356-363）。**坑**：可靠 bunch 深度上限与包无关（RELIABLE_BUFFER=512 条），发送速率与 ack 速率失配时在 TCP 型传输上会误伤（见 T11）；「部分可靠」的例外语义（分片>8 强转可靠）对调用者不可见。
3. **可靠队列溢出的精确坐标（预研缺口，本次钉死）**：发送侧 `UChannel::SendBunch`——`bOverflowsReliable = NumOutRec + OutgoingBunches.Num() >= RELIABLE_BUFFER(512) + bClose`（DataChannel.cpp:1414）→ reliable 时先发 `NMT_Failure("Outgoing reliable buffer overflow")` 再 `Close(ENetCloseResult::ReliableBufferOverflow)`（:1431-1447，注释 "Bail out, we can't recover from this (without increasing RELIABLE_BUFFER)"）。接收侧 `UChannel::ReceivedRawBunch`——`NumInRec >= RELIABLE_BUFFER` → SetError(MaxReliableExceeded)（:681-686）→ 链上抛至 `DispatchPacket` 的 CorruptData 汇总断连（NetConnection.cpp:4187-4196）。RPC 变体：`RPCReliableBufferOverflow`（NetDriver.cpp:3314-3331）。

## 2.1 Bunch 抽象

- FOutBunch/FInBunch（Engine/Source/Runtime/Engine/Public/Net/DataBunch.h）：携带 ChIndex/ChSequence/bReliable/bPartial/bOpen/bClose/bDormant/ChName/CloseReason。构造时即检查可靠缓冲余量（DataBunch.cpp:151-155，将满则 `SetOverflowed(-1)`——上层据此在发送前失败）。
- 单 bunch 尺寸上限：`GetMaxSingleBunchSizeBits() = MaxPacket×8 − MAX_BUNCH_HEADER_BITS(256) − 包头 − HandlerBits`（NetConnection.h:1361-1364）。分片循环：DataChannel.cpp:1296-1401（`MAX_SINGLE_BUNCH_SIZE_BITS/BYTES`、partial 首片/中片/末片标志）；重组在 `UChannel::ReceivedNextBunch`（:768+，partial 队列合并，非字节对齐等错误族 :803-1053 各自 SetError 上抛）。逻辑 bunch 上限 64KB（`net.MaxConstructedPartialBunchSizeBytes=65536`，DataChannel.cpp:105-110；发送侧超限仅丢弃+ensure :1259-1264，接收侧 PartialTooLarge→断连 :971-978）。
- 保序实现：可靠 bunch 带通道内序号，`ReceivedSequencedBunch`（:579-596）丢弃重复/旧序；不可靠 bunch 依赖包序 + 「只有最新通道状态有效」的语义。

## 2.2 为什么不是 packet 层（能力清单，全带坐标）

| 能力 | bunch 层实现 | packet 层做不到的原因 |
|---|---|---|
| 部分可靠 | 分片阈值转可靠（DataChannel.cpp:1416-1429） | 包丢失重发不能只重发「半个消息」的另一半以外部分 |
| 每通道独立保序 | 通道各自 InRec/OutRec（NetConnection.h:82 上限；:681 接收检查） | 包级全局序会被单通道反压拖死全部 |
| 精确 NAK 重发 | changelist→OutPacketIdRange→bunch 重发（RepLayout.h:356-363） | 包级重发重传整包（含无关通道数据） |
| 通道关闭语义 | bClose/bDormant bunch + CloseReason（EChannelCloseReason，CoreNetTypes.h:45-55） | 关闭是通道生命期事件，不是包事件 |

**坑**：512 条上限是「条」不是字节——大消息分片多时更快触顶；溢出即断连无降级路径（注释明言无解，除非加大 RELIABLE_BUFFER）；跨通道的总体顺序无保证（T7 表）。

## 2.3 PacketHandler 链

- 框架：Engine/Source/Runtime/PacketHandlers/PacketHandler（独立模块，PacketHandler.cpp 1358 行）。链从 .ini 读：BaseEngine.ini 的 ChannelDefinitions 与 PacketHandler 组件配置（`PacketHandlerComponents` 数组，配置于 DefaultEngine.ini 的 [/Script/OnlineSubsystemUtils.IpNetDriver] 段，组件如 StatelessConnectHandlerComponent/EncryptionComponent 按名实例化——引擎侧组件在 Engine/Public/PacketHandlers/）。
- 挂点：每个包 Outgoing/Incoming 全链变换（Handler->Outgoing 在 LowLevelSend 之前，WebsocketConnection.cpp:68-81 亲读）；连接前流量走 `ConnectionlessHandler`（WebSocketNetDriver.cpp:95、130-144）。握手=StatelessConnect（T3）；加密=`net.AllowEncryption`（NetDriver.cpp:529）；压缩不在默认链（项目自加组件）。
- 可插拔程度：换传输后端时 PacketHandler **原样保留**（WebSocket 驱动即证，T11）；换应用协议才需要动它。压缩/加密组件与传输解耦是这套设计的真实卖点。

## 2.4 位级序列化与量化

- 量化类型：`FVector_NetQuantize/10/100/Normal`、`FRotator_NetQuantize*`（EngineNetSerialization.h，CoreUObject）——定点量化（10=每分量 1/10 单位精度，100=1/100）；`FRepMovement` 打包（ActorReplication.cpp:426-482 的 GatherCurrentMovement——服务器侧把变换/物理状态压进 FRepMovement 再走属性复制）。误差：量化误差 = 分辨率的一半（Estimated，由定点定义推）；对一致性的影响：**同一逻辑值在两端可能量化出不同位**（浮点进定点），UE 用「两端同码量化」规避，跨实现（目标引擎 Rust/C# 差分）必须把量化器也规范化。
- NetSerialize/NetDeltaSerialize 钩子调用点：属性比较与序列化在 RepLayout 命令表执行时分流（NetSerializeLayouts，RepLayout.cpp:6098；CustomDelta 走 ReplicateCustomDeltaProperties，DataReplication.cpp:2033）。

## 2.5 每连接带宽/速率配置项（准确名）

- 基础：`UPlayer::ConfiguredInternetSpeed/ConfiguredLanSpeed`（Player.h:30-36，globalconfig；BaseEngine.ini:1838-1840 默认 100000/100000）；DS 下限钳 1800/2600（NetConnection.cpp:588-596）；`MaxClientRate/MaxInternetClientRate`（NetDriver.h:907-912，BaseEngine.ini:1866-1867 100000）；协商 `NMT_Netspeed` → Clamp(1800, MaxClientRate)（PlayerController.cpp:539-542、World.cpp:7471）。
- 执行：token bucket（T5.9）。动态带宽：AGameNetworkManager `TotalNetBandwidth=32000 / MinDynamicBandwidth=4000 / MaxDynamicBandwidth=7000`（GameNetworkManager.h:62-71，BaseGame.ini:19-47）。
- keepalive：`KeepAliveTime=0.2`（NetDriver.h:931-932）。

## 意外发现

1. `net.PartialBunchReliableThreshold` 的转可靠逻辑带着「除非会溢出可靠缓冲」的例外（:1416-1429）——两个预算互相挤兑时的取舍写在同一段 if 里。
2. FOutBunch 构造即自检可靠余量（DataBunch.cpp:151-155）——「构造失败」是合法状态，上层必须检查（RPC 路径检查了，NetDriver.cpp:3314；普通属性路径靠通道内检查）。
3. 控制通道 bunch 禁止序列化 FName/UObject*（DataBunch.h:205-216，直接 Fatal）——协议设计约束以断言形式执行。

## 对目标环境的迁移含义

「可靠性建在消息层（bunch）而非传输层」对目标引擎依然正确——WebTransport 的可靠流承担通道保序、不可靠数据报承担状态流，正好是 bunch 模型的传输化版本；但 UE 的三个具体决定应重新考虑：512 条上限+断连（应改为反压+降级）、部分可靠的隐式阈值（应显式进 Schema：每消息可靠性档位）、每通道独立队列无全局长度视图（应暴露每连接总积压给拥塞控制）。量化器必须与状态哈希同一套规范化定义（T4.8 第 5 条）。
