# 执行摘要 · UE Dedicated Server 与网络栈源码解剖（2026-08-29）

> 版本：**UE 5.8.2**（BranchName UE5 · CompatibleChangelist 55116800 · git `ff8421f2b`，2026-08-25）。所有行号相对此版本。置信度四级：Verified-Src（亲读源码）/ Verified-Doc / Reported / Estimated——本文除标注外全部为 Verified-Src，坐标见各章与 appendix/evidence-index.csv（120 条）。

## 这是什么

对 UE 专用服务器与网络栈的源码级解剖，接续第一波（文档级预研）的欠账：把 Reported 变成 Verified-Src，把写薄的章（回放、安全、断线恢复）按源码补厚。交付物为 19 章（T0–T18）+ 9 个附录硬指标表，全部坐标化。

## 最重要的十个源码级发现（★=推翻/修正预研）

1. **★ 影子状态不是每连接一份，是每对象一份、全连接共享。** 影子缓冲挂在 `FRepChangelistState::StaticBuffer`，由 `FReplicationChangelistMgr` 持有并存放在 `UNetDriver::ReplicationChangeListMap`（每对象一个）；每连接只有 `FSendingRepState` 游标。逐属性比较**每帧每对象至多做一次**（`GShareShadowState` 复用门；角色属性每连接特查）。RepLayout.h:433-500 · NetDriver.cpp:7917-7926 · RepLayout.cpp:1275-1335。**「连接数×属性数」的 diff 成本项在 UE 5.8 不存在**——预研最重要的成本模型勘误（CW-01）。
2. **ServerReplicateActors 的排序与截断全部钉死。** 排序 = 每连接相关集按 `65536 × NetPriority × Time`（Time=自上次发送秒数或 SpawnPrioritySeconds）降序；距离乘数用编译期宏（2000/3162/8000 的平方）。截断 = 两道 `IsNetReady()` 预算门（连接级 return 0、对象级 return j）；**截断不丢弃**——未处理对象经 `MarkRelevantActors` 打 `bPendingNetUpdate` 回流下一帧。带宽预算的真实形状是每连接 token bucket（`QueuedBits`，按 `CurrentNetSpeed×dt` 回填、允许 2 帧突发）。NetDriver.cpp:5152-5934 · NetConnection.cpp:2731-2751、5112-5145。
3. **★ 可靠队列溢出的精确行号与后果。** `RELIABLE_BUFFER=512`（NetConnection.h:82）：发送侧 DataChannel.cpp:1414 判定 → :1445 `Close(ReliableBufferOverflow)`（先发 NMT_Failure）；接收侧 :681 → NetConnection.cpp:4196 CorruptData 汇总断连；RPC 变体 NetDriver.cpp:3314-3331。无降级路径——注释自评 "can't recover without increasing RELIABLE_BUFFER"。
4. **★ 浏览器传输的裁决证据：UE 自带的 WebSocket 驱动在可靠有序流上重演整套 UDP 协议。** UWebSocketConnection 继承 UNetConnection 全部机制（序号/ack/重传/无状态握手全套重跑），MaxPacket 默认 **512 并记账幻想的 IP+UDP 头开销**；驱动级发送按地址字符串线性扫描。结论：连接/通道/bunch/复制四层传输无关可继承；包层与拥塞映射必须重造；加密层在 TLS 下冗余。WebsocketConnection.cpp:16-17/63-101/162-179。
5. **★ 三条「名称待核」被推翻**：不存在 `net.MaxTickRate`（真身是 ini 键 `NetServerMaxTickRate=30`，被 `t.MaxFPS` 覆盖）；时间同步不在 PlayerState 而在 **AGameStateBase**（10Hz 下发、250 样本均值 + 0.5 阻尼，**无 RTT 补偿**）；另有 8 个常见符号在 5.8 源码不存在（含 `WITH_REPLICATION_GRAPH`、`ObjectReplicator.cpp`、`LogRepFastArray`、`IDemoNetworkStream` 等，全表见 CW-12）。
6. **断线重连的「没有」被生命周期证明，且找到两个例外。** 断链后 N+1 帧内联销毁 PlayerController/PlayerState（全链坐标在 T15）；token 族关键字全树 0 命中。例外：`AGameMode::InactivePlayerArray` 的限时快照、StatelessConnect 的 Restart Handshake（仅地址变化场景的连接恢复）。引擎唯一的排空原语是 `GracefulClose`（等可靠数据 ack，上限 2s）。
7. **AOI 没有进出事件：通道开关就是事件，且天然迟到。** 失相关关通道发生在 `RelevantTimeout=5s` 滞回之后，重查节流 ~1Hz；startup actor 关通道不销毁对象（僵尸态）。把 AOI 进出绑成实体生命周期钩子的设计会同时踩抖动与僵尸两个坑。
8. **CMC 预测链全坐标化，前提条件被结构化。** 客户端 SavedMove（含 move 合并的「回退+碰撞测试」）、发送节流 [1/120,1/5]；服务器**不信任客户端 dt**（按时间戳差重算）、纠错带率限与落地信任阈值；客户端按时间戳 ack 后重放未确认 move。成立前提=输入可重放+步长两端一致+状态可量化——通用 Actor 三缺三，这就是「只有移动能预测」的结构性答案。NetworkPrediction 的 Fixed 策略 + group rollback 是 UE 内最接近目标引擎「整帧确认/回滚」的原型（Beta、默认关）。
9. **服务器流送默认关，且协议层没有「未加载≠空」的第三态。** `wp.Runtime.EnableServerStreaming=0`——DS 默认全量常驻；开启后复制侧有三道 Level 门，但客户端缺包=断连、非 startup actor 失相关=销毁，「服务器存在但没发的状态」对客户端不可见。目标铁律「缺失 chunk≠空世界」对应的协议件 UE 完全没有。
10. **关闭原因已是枚举全表（约 40+ 触发点），其中三处反直觉**：控制通道队列超 32768 绕过 Close() 直接置 Closed；畸形 bunch **服务器必断、客户端（非 Iris）容忍**；客户端日志 >5 条/秒持续即踢。`_Validate` 失败的真实后果是断连而非忽略。

## 一句话给决策者

UE 网络栈真正值得原样搬走的是四件「问题域本质」：共享 changelog+每连接游标的复制内核、token bucket+回流截断的调度、消息层可靠性、滞回 AOI；真正必须抛弃的是它的 UDP 形状（包尺寸/可靠上限/幻想头开销）与「断连是唯一失败答案」的粗暴；浏览器传输用 WebTransport 多流映射通道即可继承大部分结构（详见 T11.3 逐层裁决表与 T18 十二条原则）。

## Known gaps（读了源码仍没答案）

1. **PacketHandler 组件的完整 ini 装配语法**（ChannelDefinitions 之外的组件段逐键清单）——读了框架与两个组件，未逐键核 DefaultEngine.ini 模板；下一轮从 `UPacketHandlerComponent` 子类反查注册宏即可。
2. **Iris 深水区**（FReplicationSystemImpl 内部的 preDispatch/delta compression 算法体、ChunkedDataStream 分块协议）——读了描述符/桥/流接口与集成点，未逐函数读实现；建议下一轮带「Iris 在 Fortnite 的公开分享」对照读 EngineReplicationBridge.cpp。
3. **cook/打包对 server-only 内容的剥离链**（Target/CookCommand 层）——判定为打包管线而非网络栈，按范围裁掉；若需要，从 `FServerPlatform`/cook filter 入手。
4. **`net.iris.*` 全 CVar 族的逐项语义**——抓了 8 个关键项，未穷举（IrisConfig.cpp 一带还有数十个）；下一轮 grep `TEXT("net.Iris` 全量落表。
5. **第一波正文比对**——第一波目录不在本机，T17 以提示词所引预研论断为基准；拿到第一波全文后按 corrections CSV 回填「预研位置」列即可。
