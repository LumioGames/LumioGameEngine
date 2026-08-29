# T14 · 失败、关闭与信任边界

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。关闭原因全表见 appendix/close-reasons.csv；资源上限常量并入该 CSV 与 2.2 节。

## 结论先行

1. **UE 5.8 把关闭原因做成了枚举 `ENetCloseResult`**（Net/Core/Public/Net/Core/Connection/NetCloseResult.h:23），经 `NMT_CloseReason` 发给对端（NetConnection.cpp:1241-1284）——这张枚举表就是「UE 认为的所有连接级失败模式」穷举（约 40+ 触发点，全表见 CSV）。断连的正规路径之外还有一个**绕过 Close() 的特例**：控制通道排队超 32768 条直接置 USOCK_Closed（DataChannel.cpp:2170-2176）。
2. **畸形数据 = 断连是默认答案，且服务器比客户端严**：FBitReader 越界置错（BitReader.cpp:271-303）逐层上抛（包→bunch→通道），最终在 `DispatchPacket` 汇总——**服务器必断**（NetConnection.cpp:4196），客户端仅 Iris 路径断（:4207）、非 Iris 只记日志。无「跳过坏包继续」模式。
3. **RPC 参数校验的机制与局限**：`_Validate`（UHT 生成，只拿参数）失败 → 断连（T7.2）；未知函数静默拒（DataReplication.cpp:1473）；每帧不可靠 multicast 配额 `net.MaxRPCPerNetUpdate=2`；RPC 洪水的专门检测在 NetCore（RPCDoSDetection，NetConnection.cpp:642 的踢人回调）。**字段可见性**：`COND_OwnerOnly/SkipOwner` 属性级可控（T4.3 表）；打包客户端的服务器信息：`WITH_SERVER_CODE=0` 编译期剥离服务器分支（T1），但类布局/字段集不含条件字段的连接可见性差异——可见性是「不序列化」而非「不存在于客户端知识」。

## 14.1 关闭原因分类学（对着 CSV 的归纳）

| 类别 | 代表枚举 | 触发点例 |
|---|---|---|
| 超时 | ConnectionTimeout | NetConnection.cpp:4923-4961（判定）、:5205（关闭）；值 60s（BaseEngine.ini:1855-1856） |
| 资源上限 | ReliableBufferOverflow / MaxReliableExceeded / RPCReliableBufferOverflow / PartialTooLarge / ControlChannelBunchOverflowed | T2.2；CSV 全表 |
| 协议畸形 | Bunch* 族 / ControlChannelMessage* / CorruptData | NetConnection.cpp:3672-4216；DataChannel.cpp:1817-2277 |
| 版本 | OutdatedClient | NetConnection.cpp:1367-1372（T3） |
| 滥用防护 | RPCDoS / LogLimitInstant / LogLimitSustained | NetConnection.cpp:642、5902、5926（>60 logs/s 即踢——**日志洪泛也算攻击面**） |
| 应用层 | Disconnect / HostClosedConnection / EncryptionFailure / MissingLevelPackage | NetDriver.cpp:3942/2606、NetConnection.cpp:6216、:1976 |
| 优雅路径 | GracefulClose 机制（等可靠数据 ack，上限 2s） | NetConnection.cpp:1176-1224（T15） |

## 14.2 资源上限全表（常量名/值/坐标/可配置性）

| 常量 | 值 | 坐标 | 可配置性 |
|---|---|---|---|
| RELIABLE_BUFFER | 512 | NetConnection.h:82 | 编译期（注释明言改它=网络版本变更级） |
| MAX_CHSEQUENCE | 1024 | NetConnection.h:84 | 编译期 |
| MAX_PACKET_SIZE | 1024B | CoreNet.h:794 | 编译期（连接级 MaxPacket 经握手再 clamp） |
| MAX_BUNCH_HEADER_BITS | 256 | NetConnection.h:85 | 编译期 |
| net.MaxConstructedPartialBunchSizeBytes | 65536 | DataChannel.cpp:105-110 | CVar |
| net.PartialBunchReliableThreshold | 8 | DataChannel.cpp:92-96 | CVar |
| MAX_QUEUED_CONTROL_MESSAGES | 32768 | ControlChannel.h:61-64 | 编译期 |
| 通道数 | 运行时数组；`net.MaxChannelSize`（默认 0→DefaultMaxChannelSize=32767，NetConnection.cpp:405、80-81） | NetConnection.h:657-658 | CVar/ini |
| OLD_MAX_ACTOR_CHANNELS | 10240（旧版本兼容） | NetConnection.cpp:3664 | 编译期 |
| 超时族 | ConnectionTimeout/InitialConnectTimeout=60、KeepAliveTime=0.2、GracefulCloseConnectionTimeout=2.0 | NetDriver.h:931-951 + BaseEngine.ini:1855-1859 | ini + URL 参数（?ConnectionTimeout=，NetDriver.cpp:1837-1852） |

溢出传播链（BitReader→断连）逐层坐标见 CSV 头部注释与 T2；要点：**每层都可能被 `FNetConnectionFaultRecovery` 拦截一次**（NetConnection.cpp:1233-1239、:2192-2196——可注入的自愈钩子，引擎默认无人注册）。

## 14.3 信任边界的源码坐标

- 客户端默认被信任的大宗：移动（服务器复核+纠错而非拒绝，T10；`ClientAuthorativePosition` 默认 false 是唯一总开关，GameNetworkManager.h:188-189）；`bClientIgnoreMovementCorrections` 允许客户端拒收纠正（CharacterMovementComponent.cpp:11248）。
- 加固钩子：`_Validate`（参数级，断连）；`SendRPCDel`（发送侧项目拦截，NetDriver.cpp:8157）；`NotifyServerReceivedClientData`（移动输入准入门，T10）；`AGameSession::ApproveLogin/PreLogin`（准入）；RPCDoS/日志限速（引擎自动）；PacketHandler 加密（`net.AllowEncryption` 可强制=2，NetDriver.cpp:529、630）。
- 打包客户端的信息残留：服务器代码段编译期消失（WITH_SERVER_CODE），但**类默认对象（CDO）与复制字段集完整存在于客户端**（布局由类序列化决定）——「服务器字段」若未标 Replicated 则无数据泄漏；标了但用 COND_ 前缀控制的字段值在满足条件时仍会到达客户端。真正的保密边界在属性声明层，不在构建层。

## 意外发现

1. 客户端非 Iris 收到坏 bunch 只记日志不断连（NetConnection.cpp:4202）——**同一畸形数据在 DS 上必踢、在客户端被容忍**，两端对「继续对话」的风险偏好不同。
2. FaultRecovery 钩子存在且默认空（:1233）——Epic 预留了「断连前的最后自救」接口，没人用。
3. 日志限速踢人（5 logs/s 持续 10 次）——服务器把「客户端刷日志」视为资源攻击。
4. `net.SkipMissingLevelDisconnect`（默认 false）——缺 Level 包断连可以被关掉变成忽略，负面清单的一员。

## 对目标环境的迁移含义

把 `ENetCloseResult` 这张表直接当目标引擎「关闭原因码」的需求基线（它是一线数值游戏多年事故的结晶），再做三个升级：①断连前先走**可注入的恢复钩子**（UE 的 FaultRecovery 空置是浪费的设计位）；②资源上限全部运行期可配并带**降级档**（UE 的 512=断连没有降级）；③信任边界集中到**准入+校验+限速**三层显式声明（UE 散在 7 处钩子，目标应收敛成一张策略表）。浏览器环境下还要加一条 UE 没有的：**鉴权前的连接配额**（WSS 升级洪泛）。
