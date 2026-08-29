# T17 · 勘误：对预研报告的证实、证伪与修正

> 前提声明：第一波交付目录 `docs/research/2026-08-29-ue-dedicated-server/` 未随本环境提供（全盘检索未命中，见 search-log.md）。因此本章以委托方提示词第 4 节所引述的预研论断为勘误基准；逐条表格见 appendix/corrections-to-wave1.csv（CW-01..CW-14）。若第一波正文与本基准有出入，以第一波原文为准补一轮对表。

## 结论先行

1. **本次最重要的证伪是 CW-01（T4.1）**：「每连接影子状态」在 UE 5.8 不成立——影子与 changelist 每对象一份、全连接共享（EV-029/030/031/034）。这不是措辞问题：它把复制成本模型里「连接数×属性数」的 diff 项直接删除，容量规划的结论方向随之改变。
2. **三条「名称待核」被推翻而非确认**（CW-03/CW-04/CW-12）：不存在 `net.MaxTickRate`；时间同步不在 PlayerState 而在 GameStateBase 且无 RTT 补偿；八个检索线索符号（SpecControlChannel、ObjectReplicator.cpp、IDemoNetworkStream、FDemoFileWriter、MAX_PARTIAL_BUNCH_COUNT、LogRepFastArray、STATGROUP_NetTraffic、ini 键 MaxReliableBuffer）在 5.8 源码中不存在。
3. **预研的三个方向性判断被证实并升级为源码级**：可靠队列溢出确实踢连接且无降级（CW-05，坐标钉死）；断线重连确实几乎没有（CW-10，以完整生命周期链证明）；回放与安全两章「写薄」的部分已按源码补厚（CW-08/CW-09）。

## 17.1 裁决统计与明细

CW-01 证伪 ｜ CW-02 补齐为 Verified ｜ CW-03 不存在(命名纠正) ｜ CW-04 证伪 ｜ CW-05 补齐为 Verified ｜ CW-06 修正 ｜ CW-07 修正 ｜ CW-08 补厚 ｜ CW-09 补厚 ｜ CW-10 证实(含证明) ｜ CW-11 修正 ｜ CW-12 源码中不存在(8 项) ｜ CW-13 补齐为 Verified ｜ CW-14 修正。

统计：证伪 2、源码中不存在 2（含 8 个符号）、修正 5、证实/补齐 5。全部条目带证据编号，可在 evidence-index.csv 反查坐标。

## 17.2 预研写薄之处的补齐说明（委托方点名 H / K 两章）

- **H 回放（本次 T13，约 4.3k 字符）**：补了什么——录制的驱动级实现（bSkipServerReplicateActors + LowLevelSend 劫持）、检查点的真实构成（SinceOpen 全量属性重录 + 状态机跨帧摊销 + guid 复用假设的脆弱性注释）、存储抽象的接口面（INetworkReplayStreamer 全函数族 + 五实现模块）、两代系统并存的现状与开关（Replay.UseReplayConnection 默认 false；DemoNetDriver 禁 Iris）、限制的注释级证据（dormancy 抬高成本 / rewindable 冲突告警）。补到什么深度——足以裁决「确定性引擎是否照抄该路线」（结论：不抄，理由见 T13.5）。
- **K 安全与信任边界（本次 T14，约 3.8k 字符）**：补了什么——关闭原因全表（ENetCloseResult 枚举化 + 约 40 触发点 + 通道级关闭语义对照）、畸形数据拒绝链的层级传播（BitReader→bunch→DispatchPacket 汇总，服务器必断/客户端分层容忍）、资源上限全表（含编译期/CVar/ini 三档可改性标注）、RPC 校验的真实后果（_Validate 失败=断连）与输入局限、字段可见性粒度（COND_OwnerOnly 属性级，布局仍存在）、打包客户端的信息残留（编译期剥离边界在属性声明层不在构建层）。补到什么深度——可直接作为目标引擎关闭原因码与限流设计的需求基线。
- 其余补厚：T15 断线重连（证明「没有」的完整生命周期链）、T16 可观测性（四纠错 + 五类观测量对照）、T11 传输（逐层裁决表）。

## 17.3 对第一波建议的处理

按 R8：未触碰第一波目录；本目录的 corrections-to-wave1.csv 即全部修正的正式载体；涉及第一波 M 章（原则章）的影响已在该 CSV「影响面」列逐条标注（如 CW-01 影响容量规划原则、CW-11 影响 AOI 设计原则）。
