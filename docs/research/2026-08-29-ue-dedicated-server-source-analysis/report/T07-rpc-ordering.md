# T7 · RPC 路径与顺序保证清单

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。顺序保证全表见 appendix/ordering-guarantees.csv。

## 结论先行

1. **ProcessRemoteFunction 的完整路径**（NetDriver.cpp:8125-8277 亲读）：销毁/撕离守卫（:8133-8143）→ 项目拦截钩子 `SendRPCDel`（:8157，可整体阻断）→ Iris/ReplicationDriver 优先接管（:8184-8216）→ **multicast 逐连接过 IsNetRelevantFor**（:8243，可靠 RPC 在 `net.AllowReliableMulticastToNonRelevantChannels`=1 且通道还在时可豁免，:8244）→ 单播走 `Actor->GetNetConnection()`（:8263，无连接=丢弃+告警 :8274）。参数打包在通道的 actor bunch 里（ProcessRemoteFunctionForChannelPrivate，:3223-3360+），可靠/不可靠分别入 OutRec 或随当前包走。
2. **`_Validate` 失败的真实后果是断连**：UHT 生成的 `_Validate` 在接收端 `ProcessEvent` 前执行，失败时置 `RPC_ValidateFailed(reason)`（CoreNet.cpp:667-670）→ `FObjectReplicator::ReceivedRPC` 检测后返回 false（DataReplication.cpp:1465-1469）→ 上游 `UActorChannel::ProcessBunchInternal` 以 ObjectReplicatorReceivedBunchFail 关连接（DataChannel.cpp:3473，见 T14 表）。校验函数**只拿得到参数本身**（拿不到调用者上下文/历史），这是它的结构性局限。
3. **顺序保证是一张精确的表，不是一句「保证/不保证」**（下文全表）。核心事实：同一 actor 通道内「属性与 RPC 同通道」→ 发送顺序保持；跨通道（不同 actor）无任何顺序保证；可靠与不可靠混排无顺序保证；通道关闭 bunch 可靠且排在该通道最后。

## 7.1 发送路径细节

- multicast 的投递范围 = 「有 ViewTarget 的连接 × IsNetRelevantFor」——即 **multicast 与 relevancy 直接耦合**，且判定用的是调用瞬间的 relevancy（无 T5 的 1Hz 节流/滞回——RPC 时刻逐连接现算，:8233/8243）。不可靠 multicast 发给不相关连接会被静默丢弃；可靠 multicast 靠 CVar 豁免保通道存活期的投递。
- 「对不相关对象调用 RPC」的源码行为：服务器→客户端单播依赖 `GetNetConnection()`（属主连接），与 relevancy 无关；若 actor 无连接（不复制/无主）→ 告警丢弃（:8274）。客户端→服务器 RPC 必须有 actor 通道（bunch 挂通道），没有通道的 actor 客户端根本不持有。关闭中的通道收到 RPC → 丢弃记日志（"RPC bunch on closing channel"，:3444）。
- 不可靠 multicast 的每帧预算：`net.MaxRPCPerNetUpdate=2`（DataReplication.cpp:38-42，超出丢弃）；Iris 路径的队列上限族见 T8。
- RPC 延迟执行：可靠 RPC 引用的 GUID 未映射时挂 unmapped 队列（`net.DelayUnmappedRPCs`，DataReplication.cpp:45；SkipIfNotReady 语义，:1431-1434）。

## 7.2 顺序保证全表

| 关系 | 保证 | 依据 |
|---|---|---|
| 属性↔属性（同一对象同一连接） | **有**（通道内 changelist 序 + bunch 序） | RepLayout.h:356-363（changelist 线性历史）；FRepChangedHistory 按通道游标推进 |
| 属性↔RPC（同一对象） | **有**（同通道同序） | RPC bunch 与属性 bunch 共用 UActorChannel（NetDriver.cpp:3223+ 序列化进通道） |
| RPC↔RPC（同对象=同通道、同可靠） | **有**（OutRec/InRec 保序） | DataChannel.cpp:681+ 接收序检查 |
| RPC↔RPC（同通道、跨可靠性） | **无**（不可靠可被后续可靠超越/丢失） | 可靠走重传队列、不可靠随包即弃 |
| RPC↔RPC（跨对象=跨通道） | **无** | 通道独立序号空间（NetConnection.h:82 语义） |
| 属性↔RPC（跨对象） | **无** | 同上 |
| 通道关闭↔该通道属性 | **有**（close bunch 可靠且最后） | DataBunch.cpp:209-215（控制消息恒可靠）；UChannel::ReceivedSequencedBunch :579-596 |
| multicast 的跨连接相对时序 | **无**（各连接独立预算/相关性） | NetDriver.cpp:8227-8253 逐连接独立判断 |
| 客户端 Server RPC ↔ 服务器属性下发 | **无跨向顺序**（两个方向不同通道族） | 上行 actor 通道 vs 下行复制通道语义 |

语义边界（「最新值获胜」的不可表达清单）：计数器（增删、计分——中间值丢失）、一次性事件语义依赖可靠 RPC、跨对象因果（A 死亡触发 B 开门——B 的玩家可能还没收到 A 的死亡）、时序敏感状态机（依赖两个属性先后来到的顺序）。源码依据：属性 = 共享 changelog 的最新快照序列化（T4），无事件语义；事件必须用 RPC 或 FastArray 回调表达。

## 意外发现

1. `ForwardRemoteFunction`（DataReplication.cpp:1442）——接收端 RPC 在执行前先转发给驱动（回放驱动用它分流录制），「RPC 的接收侧也有钩子链」这一点常被忽略。
2. `Rejected unwanted function`（:1473）：收到的函数不在该类的复制函数表 → 静默拒绝（防伪 RPC 的第一道闸，仅记 Verbose）。
3. `net.MaxRPCPerNetUpdate` 默认仅 2——不可靠 multicast 的每帧配额比多数人直觉低一个数量级。

## 对目标环境的迁移含义

顺序表可直接当需求清单用：目标引擎「契约先行 + 封闭字段集」下，应把「同实体内全序（属性+RPC 统一为实体日志）」做成默认保证——UE 里最常咬人的「跨对象无序」在 ECS 单实体流里免费消失；跨实体因果则应显式交给命令序号（目标引擎的 WAL 顺序）。_Validate 的教训：校验钩子的输入应包含「会话上下文 + 速率历史」（UE 只给参数，逼项目在 GameMode 层重做）；「校验失败=断连」过于一刀切，目标引擎应允许「拒绝本条+计数+升级处罚」的梯度。
