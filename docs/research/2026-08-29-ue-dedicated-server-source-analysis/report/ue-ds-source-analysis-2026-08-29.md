# UE Dedicated Server 与网络栈源码解剖 · 主报告（T0–T18）

> 本文件为主报告入口。因篇幅按章拆分（许可见 README），各章正文在同目录 `Txx-*.md`；硬指标表在 `../appendix/`。

## 版本三件套（所有行号相对此版本）

| 项 | 值 |
|---|---|
| Engine/Build/Build.version | UE **5.8.2**（Major 5 / Minor 8 / Patch 2）· BranchName "UE5" · Changelist 0 · **CompatibleChangelist 55116800** · IsLicenseeVersion 0 |
| git | HEAD `ff8421f2b8cb4feb76fff57965a1effc53a6eb7b`（branch 5.8）· 最近提交 2026-08-25 "Localization Automation using CL 57313377" |
| 关键插件成熟度 | Iris **Beta**（默认禁用，真身 IrisCore 模块）· ReplicationGraph **Beta 插件**（`WITH_REPLICATION_GRAPH` 已不存在）· NetworkPrediction **Beta** · WebSocketNetworking **Experimental 目录** · OnlineSubsystem/Null 成熟（默认启用）。逐字段证据：T0 |

## 读取范围声明

实际打开读取：Engine/Private 的 NetDriver.cpp（关键函数族全文）、NetConnection.cpp、DataChannel.cpp（关键段）、RepLayout.cpp（比较/布局段）、DataReplication.cpp（复制执行段）、ActorReplication.cpp、Actor.cpp（休眠段）、CharacterMovementComponent.cpp（预测链五段）、StatelessConnectHandlerComponent.cpp（头注释协议+实现）、NetworkVersion.cpp、WorldPartition.cpp（服务器流送段）；Net/Core（PushModel/FastArray/NetCloseResult/NetTrace）、Net/Iris（描述符/系统/流/桥）、PacketHandlers、NetworkReplayStreaming、WebSocketNetworking 插件（全量）、ReplicationGraph/NetworkPrediction 插件（结构+关键函数）；BaseEngine.ini/BaseGame.ini 网络段；全部 .uplugin。**该读未读**：Iris 实现深水区（ReplicationSystemImpl/delta 压缩算法体）、PacketHandler ini 装配逐键、cook 剥离链——见执行摘要 Known gaps。

## 置信度图例

| 级别 | 含义 | 本次占比 |
|---|---|---|
| Verified-Src | 亲读源码实现，必带 路径:行 · 符号 三件套 | 绝对主导（五个重章全部亲读；120 条证据索引） |
| Verified-Doc | 官方文档/源码注释明文 | 0（未引用外部文档） |
| Reported | 社区共识未核一手 | 仅个别历史背景，均标注 |
| Estimated | 推断，注明依据 | 少量（如 RepGraph 项目侧代码量级、量化误差公式） |

## 章节导航

T0 基线 → T1 进程与权威 → T2 驱动/连接/通道/Bunch → T3 握手与控制消息 → **T4 复制内核（重）** → **T5 复制主循环（重）** → **T6 Relevancy/Dormancy/RepGraph（重）** → T7 RPC 与顺序 → T8 Iris → T9 时间与 Tick → **T10 预测与和解（重）** → **T11 传输可替换性（重）** → T12 服务器流送 → T13 回放 → T14 失败与信任边界 → T15 断线重连 → T16 可观测性 → T17 勘误 → T18 结论。执行摘要见 `00-executive-summary.md`。

## 执行摘要（十个最重要源码级发现，★=推翻/修正预研）

1. ★影子状态每对象一份全连接共享，非每连接（CW-01，本次最重要勘误）。
2. ★ServerReplicateActors 排序/截断/回流全坐标化（token bucket+截断回流）。
3. ★可靠队列溢出精确到行：发送 DataChannel.cpp:1414→1445；接收 :681→NetConnection.cpp:4196。
4. ★WebSocket 驱动在可靠流上重演整套 UDP 协议（512 包+幻想头开销）——浏览器裁决的一手证据。
5. ★三条名称待核被推翻：无 net.MaxTickRate；时间同步在 GameStateBase 非 PlayerState 且无 RTT 补偿；8 个常见符号不存在。
6. 断线重连的「没有」被生命周期证明；GracefulClose(≤2s) 是唯一排空原语。
7. AOI 无进出事件：通道开关即事件且带 5s 滞回 + startup actor 僵尸态。
8. CMC 预测链全坐标化；「只有移动能预测」的三个结构性前提。
9. 服务器流送默认关；「未加载≠空世界」的协议第三态 UE 完全没有。
10. 关闭原因枚举全表（约 40+ 触发点）；三处反直觉（绕过 Close 的特例/两端畸形容忍不对称/日志限速踢人）。

（展开与坐标：`00-executive-summary.md`）
