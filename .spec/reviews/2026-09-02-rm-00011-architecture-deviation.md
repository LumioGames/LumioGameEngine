---
name: 2026-09-02-rm-00011-architecture-deviation
description: RM-00011 独立深审——证据退回结论与七步架构对照的偏离清单、重拆卡建议；复核 ADR-056 依据时查
metadata:
  type: doc
  status: 已交付
---

# RM-00011 独立深审：证据退回 + 架构偏离清单（2026-09-02）

Reviewer：独立深审（写的人 ≠ 审的人）。审查环境：六仓 `origin/main` 经 `git archive` 物化的只读快照；Workflow 只 GET。
SHA：Arch `3b681a6` / Game `a120f0d`（verifier `1169a66`）/ Server `0e10b04` / Runtime `c0e7f6d` / Client `967523c` / NativeCore `fdfa4dd`。
裁决落点：[`../decisions/ADR-056-rm00011-architecture-convergence.md`](../decisions/ADR-056-rm00011-architecture-convergence.md)（Draft）。

## 1. 证据面结论：退回（P0=0，P1=1，P2=5，P3=5）

| 级 | 发现 | 证据 |
|---|---|---|
| P1 | R-00354 C# live-11 包的 S6 / S11 / S7 关键字段是 harness 常量，非观测值 | Game `integration/entity-chat/scenarios.mjs:618-637`（eventOrder 由发送循环 push）、`:653`（`appliedTicks = eventOrder.map(() => 1)`）、`:672-673`（`restoredWindow: 0` 字面量）；Server `LiveElevenHost.cs:414-448` 丢弃 tick events，App 层无 chat.event 出站 |
| P2 | Playwright 只登录 Account Server，浏览器从未进 Room、未收聊天 | `scenarios.mjs:345`（`receivedFromNetwork = account.accepted`）、`web/index.html` 仅 `loginAccount`；Rust `browser.rs:244-271` |
| P2 | Bot 发言节奏两条路径都不是 Client Timer Manager | `scenarios.mjs:618` for 循环、`suite.rs:389-395` for 循环；`timerManagerInvoked` 由宿主 tick 反推（`scenarios.mjs:652`、`suite.rs:425`）；三仓对 lumio-timer 零依赖 |
| P2 | R-00345 done/100 但 3 条验收项 `not_started` | Workflow `acceptance-items` GET 原样 |
| P2 | R-00359 oracle 身份未钉死、路径硬编码派工机、Server CI 不跑 cargo | `entity_chat_acceptance.rs:10-13,41-58`；`discover.rs:7-8,29,48-55`；`browser.rs:11`；`run_playwright_browser.mjs:30`；`.github/workflows/repository-policy.yml` |
| P2 | Server PR #18/#22/#23 在必过检查 FAILURE 下 admin 合入；引用的审查存档 `ffb22c9` / `56d5afd` 不存在 | `gh pr view`；`git cat-file -t` fatal |
| P3 | Server 根目录 `.wf-report-*.md` 与 `verify_rust_evidence.mjs:3` 自称 SUCCESS oracle，与 features 文档矛盾 | 文件头注释 |
| P3 | Rust `restore_persist_snapshot` 为快照中不在线实体建 `Active` 幽灵 LiveEntity | `host.rs:880-907` |
| P3 | 验收测试先落盘 `conclusion: SUCCESS` 再跑 oracle | `entity_chat_acceptance.rs:320-336` |
| P3 | `sdk_loader.rs:116` 无条件链接 kernel32，非 Windows 不可编译 | 本机 `cargo test` 链接失败 |
| P3 | 陈旧远端分支 `feat/r-00346-admission` 未清理 | `git branch -r --no-merged` |

通过项（docs/adr 55×120000、`DEFINITION_SHA256` 零 diff、hello-wire 未扩展、三仓 spec-lint OK、契约镜像 sha 一致、Game verifier 谓词与 47 项自测、Rust S5/S6/S7/S8 代码路径、C# tick 经 timer、安全面 loopback + env 密钥）见会话审查报告，此处不复述。

## 2. 架构对照：七步偏离清单

对照真值：`knowledge/features/ecs-entity-chat.md` §1–§5、`knowledge/features/ecs.md` §M4、决策日志 Room Review Rulings 2026-09-01、C-1…C-4。

| 步 | 设计 | 实现事实 | 裁决（ADR-056） |
|---|---|---|---|
| 1 拓扑 | Room = Runtime ECS 世界；不第二套 ECS | Game `ChatRoomWorld.cs:11-19` 独立字典世界，csproj 零引用；Runtime R-00347/351/353 产物 Game/Client/Server 引用为 0 | Room 必须用 Runtime ECS；绑定/查询只归 Runtime |
| 2 身份 | NetEntityId 不透明永不复用，绑定五元组 | C# `RoomAdmissionRegistry.cs:26,506` 与 Rust `host.rs:24-26` 各自发号；Runtime `EntityBindingQuery.Bind` 只收字符串；Rust `ConnectionBinding` 混入 `session_id` / `u64`（`host.rs:59-69`） | Runtime 身份表发号；绑定记录只五元组 |
| 3 查询 | 按声明属性 + 三维标签 + 五结局 | Runtime 按契约实现；C# `LiveElevenHost.cs:146-200`、Rust `host.rs:784-841` 硬编码；标签手写三处；C# 未知属性→`unauthorized`（`:198`）、`EntityPresence.disconnected` 恒 false（`:197`） | 标签源 = 字段标注（ecs.md M4），其余生成；AccountId 不走查询 |
| 4 聊天 | 输入走 ECS 命令路径；事件可靠有序广播到全房 | `ChatRoomWorld.cs:18` 私有队列；C# `LiveElevenHost.cs:437-442` 丢事件；Rust `host.rs:708-715` 进程内窗口；无网线投递 | 必须真广播；输入收敛到 Runtime 命令路径 |
| 5 重连 | 完整权威快照重建 ReplicaWorld；顶号明确通知 | C# 发 ADR-045 五字段空体（`MvpEnvelopeWriter.cs:33-41`）；客户端等 C-1 `stateBlocks`（`GameplayCodec.cs:153`）；Runtime `BuildFullSnapshotJson` 无人调；`TakeoverNotice` 只存字典（`RoomAdmissionRegistry.cs:24,202`）；Rust `takeover` 静默删连接（`host.rs:575-582`） | C-1 完整快照为准；顶号通知上网线 |
| 6 定时 | 宿主闹钟管墙钟；Native 节拍器共用并有真消费者 | 闹钟三份（C# `ITimerService`、Rust `expire_due` 轮询 ×10、NativeCore `HostTimerService` 闲置）；lumio-timer 不进 ABI（NativeCore ADR 0007），C# 消费者无通道；消费者仅 crate 内测试 | 单一定时内核在 NativeCore 经 ABI 暴露，双模式；各层 Timer Manager 只是适配器 |
| 7 存档 | 走既有 ECS 快照路径，不建聊天旁路 | Runtime `EcsPersistSnapshotPipeline` internal 无人调；`ChatRoomWorld.CapturePersistState`、C# `/test-control/snapshot` JSON、Rust `persist`/`restore` op 三条内存旁路；无落盘 | 走 Runtime 管线，真落盘、重启读回；WAL 留主线 |

## 3. 重拆卡建议（供主 loop 派活，不写 Workflow）

按「契约先行、按 wave 并行、文件集不重叠」拆；每卡验收以 ADR-056「验证 Fixture」为准。

**Wave 0 · 契约与生成器（架构仓 + Runtime，可并行）**
- C-2′：修订 `entity-binding-and-query-v1.json`——发号归 Runtime、`attributeDeclarations` 为生成物。
- C-4′：修订 `native-timer-abi-v1.json` + ADR-055 修订——单内核双模式；定时可达面进 `native-abi.json`，重生成 `DEFINITION_SHA256`。
- C-1′：`FullSnapshot.stateBlocks` 为 Room 路径唯一快照载体；`dimensions` 为生成物；`connection_superseded` 通知形状落点。
- G-1：Runtime 字段标注类型 + 声明表生成器；`ChatComponent` 打标；生成物与契约零 diff 门。

**Wave 1 · 唯一实现与消费方改造（跨仓并行、同仓串行）**
- Runtime：R-00347′ `EntityBindingQuery` 增发号；R-00351′ ChatInput 走 IngressCapture → CommandBuffer；R-00353′ `EcsPersistSnapshotPipeline` 公开落盘/读回 API（ADR-032 记录头）。
- NativeCore：R-00352′ 单内核双模式 + ABI 导出；删 `HostTimerService` 门面。
- Game：R-00348′ `ChatComponent` 成为 Runtime ECS 组件，`ChatRoomWorld` 退役。
- Server（Rust 宿主，接力面）：R-00359′ 宿主改为消费 Runtime 绑定/查询/快照与 NativeCore 定时；删 `host.rs` 绑定表与 `expire_due` 轮询；补 Room 广播与顶号通知上网线。
- Client：R-00349′ ReplicaWorld 由 C-1 FullSnapshot 真实重建；浏览器页接 Room 网线。

**Wave 2 · 集成**
- R-00354′：11 场景以「客户端真收到、快照真落盘真读回、定时由内核回调触发」为验收；`verify-evidence.mjs` 不得从发送计数合成 `eventOrder` / `appliedTicks` / `restoredWindow`；oracle SHA 钉死；CI 跑 cargo acceptance。
- 撤回 Server ADR 0006 C# 冻结，直至 Rust 宿主按上述验收通过。

## 4. 未能复核的项

- 派工机证据包与 `R-0035x-gate-*.log` 不在任何仓；`node verify-evidence.mjs --dir <pack>` 未在本机跑。
- Rust identical suite 与 `lumio-server-process` 单测因 kernel32 链接无法在 macOS 复跑。
