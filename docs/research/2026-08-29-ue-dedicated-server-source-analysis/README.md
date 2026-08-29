# UE Dedicated Server 源码解剖（2026-08-29 · 第二波）

## 这是什么

对 Unreal Engine **5.8.2**（git `ff8421f2b`，CL 55116800）专用服务器与网络栈的**源码级**解剖，是第一波（文档级预研，外部交回，未随本环境提供）的验证与补厚：把 Reported 升级为 Verified-Src、把回放/安全/断线恢复等写薄的章按源码补厚、把全部「名称待核」钉死。每条源码级论断带三件套坐标（相对引擎根的路径 : 起止行 · 符号名）。

## 与第一波的关系

- 第一波目录 `docs/research/2026-08-29-ue-dedicated-server/` 未改动（亦不在本机，见 search-log）。
- 对第一波结论的全部修正见 `report/T17-errata.md` 与 `appendix/corrections-to-wave1.csv`（14 条：证伪 2、不存在 2、修正 5、证实/补齐 5）。
- 本报告不重述第一波的架构/历史/社区内容，只写源码给出新证据的部分。

## 章节索引

| 章 | 文件 | 内容 |
|---|---|---|
| T0 | report/T00-baseline.md | 版本三件套、插件成熟度表、模块地图、UNetDriver 子类清单、类型索引 |
| T1 | report/T01-process-authority-trimming.md | ENetMode/Role/编译期开关、多世界支持度 |
| T2 | report/T02-driver-connection-channel-bunch.md | 三层所有权、bunch/可靠队列、溢出坐标、PacketHandler、量化 |
| T3 | report/T03-handshake-control-messages.md | 无状态握手（HMAC cookie）、登录状态机、版本校验、连接时序 |
| T4 | report/T04-replication-kernel.md | **重章**：影子状态裁决（4.1）、changelist、push model、条件全表、FastArray、迁移裁决 |
| T5 | report/T05-replication-loop.md | **重章**：ServerReplicateActors 全链、token bucket、优先级/饥饿、成本公式 |
| T6 | report/T06-relevancy-dormancy-repgraph.md | **重章**：relevancy 分支、滞回、dormancy、ReplicationGraph、体素启示 |
| T7 | report/T07-rpc-ordering.md | RPC 路径、_Validate、multicast 相关性、顺序保证表 |
| T8 | report/T08-iris.md | Iris 架构四件、Epic 自述、已接/未接 |
| T9 | report/T09-time-tick.md | tick 率真名、TickGroup、固定步长补丁清单、时间同步纠正 |
| T10 | report/T10-prediction-cmc-np.md | **重章**：CMC 全链、结构性前提、NP group rollback、信任边界 |
| T11 | report/T11-transport-replaceability.md | **重章**：UDP 假设清单、WebSocket 驱动逐层、浏览器裁决 |
| T12 | report/T12-streaming-server.md | 服务器流送、未加载≠不相关的协议证据 |
| T13 | report/T13-replay.md | 录制机制、检查点、两代系统、限制注释 |
| T14 | report/T14-failure-trust-boundary.md | 关闭原因分类学、资源上限、畸形数据策略、信任边界 |
| T15 | report/T15-reconnect-lifecycle.md | 断链销毁链、重连痕迹全零、seamless 过继、排空 |
| T16 | report/T16-observability.md | CVar/日志/stat/trace 准确清单、五类观测量对照 |
| T17 | report/T17-errata.md | 勘误（14 条）+ 写薄处补齐说明 |
| T18 | report/T18-conclusions.md | 12 条可迁移原则（五段式）、本质vs包袱、浏览器裁决、五条坑 |

## 执行摘要

全文见 `report/00-executive-summary.md`（可独立阅读）。一句话：UE 值得原样搬走的是「共享 changelog+游标、token bucket+回流截断、消息层可靠性、滞回 AOI」四件问题域本质；必须抛弃它的 UDP 字节形状与「断连是唯一失败答案」；浏览器传输经 WebTransport 多流可继承大部分结构（T11.3 逐层裁决表）。

## 附录硬指标

| 文件 | 内容 |
|---|---|
| appendix/evidence-index.csv | 120 条源码证据（章节/论断/路径/行号/符号/置信度/是否改变预研） |
| appendix/corrections-to-wave1.csv | 对第一波的 14 条勘误 |
| appendix/control-messages.csv | NMT_* 全表（33 条 + 序号空洞） |
| appendix/ordering-guarantees.csv | 顺序保证清单（13 项） |
| appendix/close-reasons.csv | 关闭原因码全表（含通道级与资源上限） |
| appendix/cvar-ini-and-commands.csv | CVar/ini 键/命令/日志/stat/trace 准确名清单（含 4 条命名纠错） |
| appendix/symbol-map.csv | T0 类型清单 |
| appendix/diagrams.md | 两张硬指标图（连接时序 / 复制循环，mermaid） |
| appendix/search-log.md | 检索日志 |

## 置信度图例

Verified-Src（亲读源码，必带坐标）｜Verified-Doc（官方文档/注释明文）｜Reported（社区共识未核一手）｜Estimated（推断，注明依据）。本次全文 Verified-Src 占绝对主导（五个重章全部为亲读函数体）；Verified-Doc 为 0（未引用外部文档）。

## Known gaps

见执行摘要末节：PacketHandler ini 装配逐键、Iris 实现深水区、cook 剥离链、net.iris.* 全族、第一波正文比对。
