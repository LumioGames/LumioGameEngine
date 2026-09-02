# ADR-055：Native Timer ABI 与双层定时架构

状态：Accepted（2026-09-01，依 Room Review Rulings 2026-09-01 裁决 #6「双层定时、两者一等公民、补真实消费者」）
取代：无
Owner：`LumioGameEngineArchitecture`（契约真值）、`LumioNativeCore`（Timer Manager core 实现）、`LumioServer` / `LumioClient`（adapter 分布与消费）

## 背景

切片 RM-00011（ECS Formal Entity and Chat Vertical Slice）暴露出定时语义此前只存在于对话与散落文档：一层是宿主 Timer 服务（单调墙钟、类型化命令投递；C# MVP 宿主侧已由 R-00272 交付），另一层是 Native Tick/Frame Timer Manager（NativeCore core + Server/Client adapter 分发 + CallbackSlot），后者一直没有冻结的 ABI。Owner 裁决（2026-09-01）确定：两层都是一等公民；切片必须给 Timer Manager 补上真实消费者（Bot 发言节奏走 Client Timer Manager、服务器侧至少一个周期任务走 Server Timer Manager）；五分钟重连保留窗口归宿主单调时钟；终态向 native core scheduler 统一的方向必须落档，不能留在对话里。本 ADR 与契约定义即该裁决的执行。

同时本仓已转入预上线 Living Architecture（ADR-052 先例）：旧 Schema/ID/Fixture/Baseline/七仓镜像体系已移除，公共契约以 `engine/wire/` 下自包含契约 JSON 为唯一真值。本契约是**进程内 API 契约**，不是 wire 传输契约，也不进入 `engine/abi/native-abi.json`（那是托管/Native 二进制边界；Timer 不改变该边界，`eng/generate-abi.mjs` 对本契约保持零差异）。

## 决策

1. **契约唯一真值**：`engine/wire/native-timer-abi-v1.json`（contractId `lumio.native-timer-abi.v1`）。TimerHandle、TimerScope、TimerKind、TimerManager、CallbackSlot、四个操作（scheduleOneShot / scheduleRepeating / cancel / advance）、错误码、`errorTriggers`（每码触发与兄弟码判别）、上限与全部生命周期/失败用例（内嵌 `testCases` / `invalidCases`，逐例断言）以该文件为单一来源；NativeCore core 与 Server/Client adapter 不得另写一份语义真值。措辞宿主无关：C# MVP 宿主与切片级最小 Rust 宿主（R-00359）复跑同一语义。
2. **句柄与生命周期纪律**：TimerHandle 为 opaque `index:u32 + generation:u32 + context:u64` 编码（ADR-006 句柄模型）；slot 复用必增 generation；解析失败一律 `stale_handle`，绝不模糊命中同 index 新 generation 的另一定时器；generation 为 u32 单调递增、溢出即进程级 Fatal，不得回绕。scope teardown/reset 一次性失效名下全部 handle。one-shot 已交付或被终态拒绝后句柄失效。本契约无 `resolve` 操作。`advance` 开火窗口是 `(committedTick, toTick]`（含 `dueTick == toTick`）；repeating 在同一次 `advance` 内每次开火后 `due += intervalTicks` 并继续收取仍落在该窗口内的后续刻。
3. **CallbackSlot 投递模型**：投递载体是创建时预注册的类型化分发目的地（注册 id + adapter 分发表），不是原始函数指针；不接受任意函数指针注册，不以 C# delegate（或任何语言原生回调类型）作为 ABI 类型——语言绑定留在 adapter 侧。Native 侧绝不直接调用 managed/gameplay 热路径回调（ADR-006 纪律）：触发先进 `advance` 的确定性返回集（`(dueTick, scheduleSequence)` 稳定全序，回放确定性是契约属性），由 adapter 在声明的分发点排空并调用绑定回调；回调内不得同步 schedule/cancel；每 slot 投递队列有界，满时该触发以 `slot_queue_full` **稳定拒绝**、定时器随之终态（不重试、不重排队）、Manager 继续运行并记诊断事件——单个 slot 队列满不升级为进程失败，不静默丢弃。CallbackSlot 生命周期为 `unbound → armed → delivering → closed`：`schedule*` 对 unbound 返回 `slot_unbound`、对 closed 返回 `slot_closed`，两码互斥（unbound 是从未绑定的初始态，closed 只在 armed 之后进入）。
4. **双层职责表**（契约 `layers` 节为真值）：
   - 宿主 Timer 服务：单调墙钟 deadline（毫秒域、非确定性）、类型化命令投递（有界端口、非回调）。拥有 R-00350 五分钟断线保留窗口（进程本地单调时钟，不跨进程重启）与宿主进程级周期任务。
   - Native Tick/Frame Timer Manager：确定性 gameplay 调度（固定 Tick/Frame、one-shot/repeating/cancel、scope/generation 校验、CallbackSlot）。拥有 R-00352 的 Bot 发言节奏（Client 侧，每 N Tick）与服务器侧至少一个周期任务（Server 侧，Tick/Frame 域）。
   - 两层互不越界：gameplay 节奏不得用墙钟定时；墙钟 deadline 与进程生命周期绑定不迁入 Manager。
5. **终态统一方向与迁移边界**：终态向 native core scheduler 统一——P0（本切片）两层并存互为一等公民；P1 native scheduler 增加单调时间域后，宿主墙钟 deadline 改经 native scheduler 承载、宿主服务保留同签名门面；P2 单一 scheduler 收口。迁移期间消费方契约（TimerHandle 语义、投递保证、错误码）不变；任何一层不得直接调用另一层内部结构。
6. **失败语义显式化**：取消后、scope 失效后、slot 关闭后到达的触发一律 `late_completion` 终态拒绝——不写状态、不调用回调、不重排队（与 ADR-006「销毁后完成即终态」同纪律）。slot 类失败（`slot_closed` / `slot_unbound` / `slot_dispatch_mismatch` / `slot_queue_full`）显式拒绝并记录错误码：调度期 `slot_unbound` 与 `slot_closed` 互斥且无 handle；投递期 slot 失败终态化该定时器但不终止进程。`manager_shutdown` 的进入条件、优先级与四操作拒绝行为以契约 `errorTriggers.manager_shutdown` 为唯一真值（宿主/adapter 拆除 Manager 实例进入不可逆 shutdown——不是第五个公开 ABI 操作；此后四操作一律稳定拒绝；非进程 fatal）。

## 替代方案

- **单层宿主定时（全部走墙钟）**：拒绝——gameplay 节奏必须落在确定性 Tick/Frame 域（回放与权威 Tick 语义要求），墙钟定时不可回放。
- **单层 Native（墙钟也进 Manager）**：拒绝——五分钟重连窗口是进程本地单调时钟语义（不跨进程重启、绑定进程生命周期），Owner 裁决明确归宿主层；在 native 层复刻进程生命周期会造成第二份语义。
- **C# delegate 直接作 ABI 类型 / 任意函数指针注册**：拒绝——违反 ADR-006（Native 回调进 Gameplay 因重入与卸载风险被否决）；语言绑定留在 adapter，契约只认预注册分发目的地。
- **完整 GameTime/RealTime/Scaled/Unscaled 时间域矩阵**：拒绝——Room Review 显式延后，本切片只冻结两层最小面。

## 接口

契约文件：`engine/wire/native-timer-abi-v1.json`（唯一真值；含 operations、errorCodes、errorTriggers、limits、内嵌 testCases/invalidCases）。`engine/abi/native-abi.json` 不变。下游消费：R-00350（重连窗口，宿主层）、R-00352（Timer Manager 实现 + Bot 节奏与服务器周期任务消费者接线）。

## 失败语义

错误码词表以契约 `errorCodes` 为准：`stale_handle` / `scope_invalid` / `scope_generation_mismatch` / `invalid_due_tick` / `invalid_interval` / `schedule_budget_exceeded` / `slot_closed` / `slot_unbound` / `slot_dispatch_mismatch` / `slot_queue_full` / `late_completion` / `manager_shutdown`。每码的触发条件与兄弟码判别以契约 `errorTriggers` 为唯一真值（含 `scope_invalid` / `slot_dispatch_mismatch` / `manager_shutdown` 的进入语义；`invalid_due_tick` 覆盖 schedule 的 dueTick 与 `advance` 回退两面；`schedule_budget_exceeded` 覆盖 `maxActiveTimersPerScope` 与 `maxSchedulesPerTick` 两面）。三条总纪律：① 终态不可逆且零状态写入（late_completion 类）；② 预算与队列上限（含 `schedule_budget_exceeded`、`slot_queue_full`）显式**稳定拒绝**、无部分调度、不静默丢弃、不升级为进程失败；③ generation 溢出是词表外的进程级 fail-stop（非稳定错误码，是本契约唯一 ProcessFault）。基础设施级错误不做字段级 Undo 或 catch-and-continue。

## 兼容影响

开发态新契约，无既有消费方；预上线阶段允许破坏式变化（ADR-052 同口径）。`engine/abi/native-abi.json` 零变更，`eng/generate-abi.mjs` 零差异。措辞宿主无关，最小 Rust 宿主（R-00359）复跑同一语义不需契约改写。

## 迁移方案

无需迁移（新能力）。向 native core scheduler 的终态统一按契约 `layers.unification` 三阶段推进，迁移边界：确定性 gameplay 调度先行（本就 native 域）；墙钟 deadline 留宿主层直到 native scheduler 具备单调时间域与进程生命周期语义；迁移不改变消费方契约。

## 验证

- 契约内嵌用例：`testCases` 4 例（one-shot 恰一次、repeating 按周期、cancel 阻断投递、投递全序可回放）与 `invalidCases` 18 例（含 `slot_queue_full` 非 fatal、`scope_invalid`、`scope_generation_mismatch`、`slot_dispatch_mismatch`、`manager_shutdown`、调度期 `slot_unbound`/`slot_closed` 互斥、schedule 面 `invalid_due_tick`、`advance` 回退、`maxSchedulesPerTick`），覆盖卡 R-00358 验收要求的六类生命周期/失败路径（one-shot、repeating、cancel、stale handle、late completion、slot failure），逐例确定性断言。`errorTriggers` 覆盖全部 12 个稳定错误码。
- 本卡自检：JSON 结构自检（必含键、12 码各有 trigger、六类用例各 ≥1、无尾逗号）、`node .spec/tools/spec-lint.mjs`、`node eng/generate-abi.mjs` 零差异；统一校验器（`eng/verify-wire.mjs`）由并行卡 R-00355/C-1 建立，随本契约合并后纳入统一校验。
- 消费方验收：R-00352 以本契约为实现真值（Bot 每 N Tick 发言 + 服务器周期任务），R-00350 以本 ADR 分层记录为重连窗口归属依据。

## 修订记录（2026-09-02，ADR-056 §7）

本段为 Accepted 正文的附录，不改写上方决策原文。ADR-056 §7 将本 ADR 决策第 4–5 条「双层各自独立基础设施、P0 两层一等公民」修订为「单内核双模式」：

- 定时内核只有一个，位于 NativeCore（Rust），经 `engine/abi/native-abi.json` 的 `timer_*` 根表槽暴露给托管侧。
- 两种模式共用 TimerHandle / CallbackSlot / 错误码：`wallClock`（单调毫秒、非确定性，驱动函数 `timer_pump`，承载重连保留窗与宿主周期任务）与 `tickFrame`（确定性、固定 Tick/Frame，驱动函数 `timer_advance`）。
- `consumers.reconnectDeadline.layer` = `kernel:wallClock`。Game / Server / Client / Worker Timer Manager 只是适配层，不得自建定时器。
- 托管可达面以 C-4′ `abiSurface` 为最小函数集（创建/销毁 manager、register_scope/teardown_scope、register_dispatch、create/bind/close slot、scheduleOneShot/scheduleRepeating/cancel、advance/pump、drain）。参数只允许不透明句柄与整数，禁止函数指针（ADR-006）。
- 上方 Accepted 正文中「本契约不进 native-abi.json」「两层互为一等公民」「墙钟 deadline 不进 Manager」「没有第五个 shutdown() ABI」等句以本修订与 ADR-056 为准：`timer_destroy_manager` 是实例拆除的 ABI 投影。
- NativeCore 仓 ADR 0007「定时不进 C ABI」由 N-08 / R-00372 取代；本仓只冻结契约与 ABI 定义，不写内核实现。

契约真值：`engine/wire/native-timer-abi-v1.json`（C-4′）+ `engine/abi/native-abi.json`（`timer_*` 槽与 Timer* 状态码）。
