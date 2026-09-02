---
name: 2026-09-02-rm-00011-r3-convergence-blueprint
description: RM-00011 修订 r3 蓝图——按 ADR-056 收敛架构的 13 张重拆卡完整执行提示词与验收项;向需求室补卡或复核派活口径时查
metadata:
  type: doc
  status: 设计中
---

# RM-00011 修订 r3：架构收敛蓝图（workflow-plan: rm00011-r3）

- 蓝图 ID / 修订：`rm00011-r3`（r1 = 建室，r2 = 2026-09-01 室审重写，r3 = 本次按 ADR-056 收敛重拆）
- 规划模式：已落单（2026-09-02 Owner 授权；R-00345 评论 `01a0608e-9484-7bf6-b87b-3007830339e7` 为变更控制记录）
- 目标项目：Workflow `lumiogamesengine`（`proj_b6979c277715a6c6c490a541ac69709b`）；目标室 RM-00011（`01a05b5a-6fd3-797f-8608-580c55491802`）；里程碑 MS-00001（`01a04225-9740-769a-9a62-f309267c701d`）
- 来源：ADR-056（Draft）、`reviews/2026-09-02-rm-00011-architecture-deviation.md`、`knowledge/features/ecs-entity-chat.md` §1–§5、`knowledge/features/ecs.md` §M4、Room Review Rulings 2026-09-01、C-1…C-4
- 变更控制：R-00345 为室内 `[原始需求]`，本修订以一条评论登记「r3 取代 r2 的 Wave 1/2 交付判定；R-00346–R-00354、R-00359 的 done 不再代表设计落地」；旧卡不流转、不删除。

## 决策账本（已锁，来源 ADR-056）

| # | 决定 | 影响卡 |
|---|---|---|
| D1 | Room 世界 = Runtime `EcsWorld`；绑定/查询只在 Runtime 一处 | N-05 N-06 N-09 N-10 N-11 |
| D2 | NetEntityId 由 Runtime 身份表发号；绑定记录只五元组 | N-01 N-05 N-10 |
| D3 | 三维标签源 = 组件字段标注；契约/注册表/宿主全部生成；AccountId 不走属性查询 | N-01 N-02 N-04 N-09 |
| D4 | 聊天事件真广播到每个客户端；输入走 Runtime 命令路径 | N-06 N-10 N-11 N-12 |
| D5 | FullSnapshot 以 C-1 `stateBlocks` 为准；顶号通知 `connection_superseded` 上网线 | N-02 N-10 N-11 N-12 |
| D6 | 单一定时内核在 NativeCore，经 native ABI 暴露，墙钟 + Tick 双模式；各层 Timer Manager 只是适配器 | N-03 N-08 N-10 N-11 |
| D7 | 快照只走 `EcsPersistSnapshotPipeline`，真落盘、重启读回；WAL 留主线 | N-07 N-10 N-12 |
| D0 | 治理原则：第一性原理，如无必要勿增实体 | 全部 |

## 依赖 DAG 与 wave

```text
Wave 0a（并行，架构仓 / NativeCore 契约 / Runtime 生成器）
  N-03 C-4′ + ADR-055 修订 + native-abi 定时导出面        [LumioGameEngine: engine/wire/native-timer-abi-v1.json, engine/abi/native-abi.json, ADR-055, ADR-056§7]
  N-04 字段标注类型 + 声明表生成器                          [LumioGameRuntime: modules/ecs/**/Annotations, tools/gen-declarations]
Wave 0b（并行，消费 N-04 产物）
  N-01 ADR-056 定稿 + C-2′（发号归 Runtime、五元组、声明表为生成物）   [LumioGameEngine: ADR-056, engine/wire/entity-binding-and-query-v1.json]
  N-02 C-1′（stateBlocks 唯一快照载体、dimensions 生成物、connection_superseded） [LumioGameEngine: engine/wire/gameplay-command-envelope-v1.json, ADR-049 附录]
Wave 1a（并行，Runtime 三卡目录不重叠 + NativeCore）
  N-05 Runtime 绑定/查询唯一实现 + 发号             [LumioGameRuntime: modules/replication/**/Binding]
  N-06 Runtime 聊天走 ECS 命令路径 + 事件生成        [LumioGameRuntime: modules/replication/**/Chat, modules/ecs/**/Ingress]
  N-07 Runtime 快照落盘/读回 API                      [LumioGameRuntime: modules/ecs/**/Snapshot]
  N-08 NativeCore 定时内核双模式 + ABI 导出           [LumioNativeCore: crates/lumio-timer, crates/lumio-native-ffi]
Wave 1b（并行，消费 1a）
  N-09 Game ChatComponent 成为 Runtime 组件，ChatRoomWorld 退役   [LumioGame: modules/server-gameplay]
  N-10 Server Rust 宿主改为纯托管 + 广播/顶号通知上网线            [LumioServer: modules/process, modules/host-runtime, entity-chat-host]
  N-11 Client ReplicaWorld 真重建 + Bot 节奏走 Timer Manager      [LumioClient: modules/replica, modules/bot]
Wave 2（串行）
  N-12 集成：11 场景在 Rust 宿主重跑，证据全部来自观测           [LumioGame: integration/entity-chat]
  N-13 Review：ADR-056 Accepted、Server ADR 0006 撤回/取代、整体放行 [LumioGameEngine .spec, LumioServer .spec]
```

共享热点唯一所有者：`engine/abi/native-abi.json` 与 `abi_generated.rs` / `AbiConstants.g.cs` → N-03；Runtime 生成声明表文件 → N-04；`verify-evidence.mjs` → N-12；各仓锁文件 → 各卡自己的仓，同仓串行。

## 共同执行规范（写入每张卡正文「指令与真值优先级」之后，逐字内嵌）

```markdown
## 共同执行规范（RM-00011 r3，全卡相同）
### 治理原则
- 第一性原理，如无必要勿增实体。同一职责只允许一份实现。你在改动范围内发现第二份实现（绑定表、声明表、定时器、快照旁路、事件队列……）时，正确动作是删除或改为消费唯一实现，绝不新增第三份；拿不准是否「同一职责」就停下升级。
### 真值优先级（高→低）
1. 架构仓 `.spec/decisions/ADR-056-rm00011-architecture-convergence.md`（八条决策，编号 §1–§8）
2. `.spec/knowledge/features/ecs-entity-chat.md` §1–§5 与 `.spec/knowledge/features/ecs.md` §M4
3. `engine/wire/` 下修订后的 C-1 / C-2 / C-3 / C-4（以本卡「前置产物」列出的提交为准）
4. 本卡正文
- Workflow 上任何卡的 `done` 状态、任何 `.wf-report-*.md`、任何 closeout 报告都不是真值；它们与上述冲突时以上述为准并在交回物里指出冲突。
### 硬禁令（违反任一条即退回，不看其余）
- 不得假冒进程名或宿主身份；不得把未观测到的值写进证据（例如把发送计数当作接收顺序、把常量当作 applied tick / restoredWindow）。
- 不得新建或保留第二份绑定表、声明表、定时器实现、快照旁路、事件队列。
- 不得恢复已删除的 Schema/ID/Fixture/Baseline/七仓镜像体系；不得扩展 `hello-wire-v1`；不得触碰 RM-00010 / LumioConfig。
- 不得在源码、测试或脚本里硬编码开发机绝对路径（`C:\Work\…`、`C:\Users\…`）；一切外部路径经环境变量或相对仓根发现，缺失时以 BLOCKED 明确报出。
- 密钥/凭据不入库、不进日志、不进证据；`123456` 只允许作为切片测试档案口令常量出现在测试与 harness。
- 不得在 CI 必过检查失败时以 admin 合入；不得 push 受保护分支；开 PR 等主 loop 审查。
- 不得替 Owner 补产品决定：契约缺口一律升级为契约缺陷（在交回物「阻塞与升级」里写明缺哪条、建议措辞），不本地打补丁绕过。
### 工作方式
- 在独立 git worktree 实现；先读 `.spec/AGENTS.md`、目标仓 `.spec/knowledge/README.md` 导航到的规范与被改源文件。
- 测试先行：每条可自动化的验收先有失败测试再有生产代码；不能自动化的写明经批准的替代证据。
- 文件集只在本卡「拥有范围」内改动；越界即停并报告。
### 交回格式（五段，缺段即退回）
一、交付物与实际变更范围（分支/提交号、文件清单）；二、逐条验收证据（对应 Workflow 验收项编号）；三、实际运行的命令与关键输出（不得只写「已通过」）；四、偏离、风险与未完成项（没有写「无」）；五、下游集成入口与知识沉淀落点（无需沉淀须声明）。
```

## 验收项写入口径

- 类型 `需求验收`（`atype_2c92d7e5acc361f7ad82b1733ab4c223`），初始状态 `未提交`（`astat_20e2c7f5c6d891ad0966208b55da0372`，`systemSemantic=not_started`）；`sourceKind=ai`、`sourceRef=workflow-plan: rm00011-r3/N-xx`。
- PM 字段沿用室内惯例：`priority` P0、`risk` high、`module` `formal ECS entity-chat slice`、`category` 按卡；`ownerId` 采用 API 创建人默认；不传 `status`。
- 归属：POST 时 `roomId` = RM-00011；读回后 `PUT /schedule/requirements/{id}/milestone` → MS-00001（`reason`: `workflow-plan: rm00011-r3`）。

---

## N-01 [程序·协议/公共][Wave 0b] ADR-056 定稿与 C-2′ 修订：NetEntityId 归 Runtime 发号、绑定记录只五元组、声明表为生成物

- category：`architecture / contract`
- 拥有范围：`LumioGameEngine` 的 `.spec/decisions/ADR-056-*.md`、`.spec/decisions/README.md`、`engine/wire/entity-binding-and-query-v1.json`、`.spec/decisions/ADR-053-*.md`（仅追加「修订记录」段）、`eng/verify-wire.mjs` 中 binding 用例。
- 前置：N-04 已交付生成器与 `ChatComponent` 声明表产物（提交号写入本卡）；N-03 已合入（ADR-055 修订不与本卡冲突）。

### 执行提示词

# 执行提示词：[程序·协议/公共] ADR-056 定稿与 C-2′ 修订
## 任务元数据
- 蓝图：rm00011-r3 / N-01
- 目标项目/仓库：Workflow lumiogamesengine；仓库 `LumioGameEngine`（架构仓，契约真值）
- 责任角色：架构契约维护者
- 前置状态：N-04 声明表生成器产物已在 `LumioGameRuntime origin/main`；N-03 已合入
- 拥有范围：见上
## 你的身份
你是架构仓的契约维护者，只改契约与决策文档，不写实现。你接到的是一张可独立执行和验收的正式需求。先验证前置和来源，再在授权范围内交付；不得替产品负责人补未授权的产品决定。
## 指令与真值优先级
（共同执行规范内嵌于此）
## 来源真值
- 原始需求：R-00345 `[Original Requirement] Formal ECS Entity and Chat Vertical Slice`（r3 评论）
- 前置产物：ADR-056 Draft（架构仓 `docs/2026-09-02-rm00011-architecture-rulings` 分支）；N-04 交付的生成声明表（路径 + sha256 由派活时填入）；ADR-053 / C-2 `lumio.entity-binding-query.v1` 现行版本
- 项目规范：`.spec/AGENTS.md`、`.spec/knowledge/standards/repository-architecture.md`、`.spec/decisions/README.md` 写作契约
- 交付基线：`LumioGameEngine origin/main`（派活时钉 SHA）
## 产品背景与已锁决策
- ADR-056 §2 §3 §4：绑定/查询只在 Runtime；`NetEntityId` 由 Runtime 身份表发号；绑定记录只保留五元组；三维标签由字段标注生成；`AccountId` 不走属性查询。
- 现状偏离：C# `RoomAdmissionRegistry.cs:26,506` 与 Rust `host.rs:24-26` 各自发号；C-2 `attributeDeclarations.example` 为手写示例；Rust 绑定记录混入 `session_id` / `u64`。
## 本需求目标
ADR-056 由 Draft 变为可被下游消费的定稿候选（状态仍 Draft，待 N-13 转 Accepted），C-2 契约文字与 ADR-056 一致，下游 N-05 / N-10 能只凭 C-2′ 实现而不需要读对话。
## 执行前置
- 确认 N-04 产物存在且 `node eng/verify-wire.mjs` 在当前 main 通过；否则 BLOCKED。
## 决策权限与升级条件
- 可以自行决定：措辞、用例命名、段落组织。
- 必须升级确认：任何改变五元组字段集、结局码词表、错误码词表的动作；任何新增第五个契约文件的想法。
## 范围与协作边界
- 本卡拥有：上列文件。只读：N-04 产物。共享热点：无。
## 详细要求
1. C-2′ `identityModel.netEntityId` 增加「由 Runtime 身份表发号；宿主准入时调用 Runtime 取号；宿主不得自铸」措辞，并在 `invalidCases` 增加 `host_minted_net_entity_id` 拒绝用例（`expectedRejection: invalid_binding_shape`）。
2. C-2′ `binding.record` 之外新增说明：会话号与宿主内部句柄不得出现在绑定记录（`invalidCases` 增 `binding_record_carries_session_id`）。
3. C-2′ `attributeDeclarations` 改为：`source: "generated-from-field-annotations"`，内嵌表 = N-04 产物逐字节拷贝，并记录产物 sha256；删除手写 `example`。
4. C-2′ `readRules` 增加「`EntityIdentity.accountId` 不声明为可查属性；对其查询返回 `undeclared_attribute`」。
5. ADR-053 追加「修订记录（2026-09-02，ADR-056）」段，指向本次变更；不改写其 Accepted 正文。
6. ADR-056：把「接口 / Schema」段落里的待定项（顶号通知落点、标注类型命名）按 N-02 / N-04 的实际交付填实；保持状态 Draft。
7. `eng/verify-wire.mjs` 补对应正反用例；`node .spec/tools/spec-lint.mjs` 通过。
## 验证计划与证据
- `node eng/verify-wire.mjs` 全绿输出；`node .spec/tools/spec-lint.mjs` → OK；`git diff --stat`；生成表与 N-04 产物 `shasum -a 256` 相等的输出。
## 必须交付
- 上列文件的 PR；交回物含 sha 比对输出。
## 验收标准
1. C-2′ 明文规定 NetEntityId 由 Runtime 发号，且有宿主自铸的拒绝用例。
2. C-2′ 明文规定绑定记录只五元组，且有会话号混入的拒绝用例。
3. C-2′ 内嵌声明表与 N-04 生成产物 sha256 一致，无手写示例表。
4. `verify-wire` 与 `spec-lint` 通过，输出附在交回物。
## 明确不做与禁止事项
- 不改 C-1 / C-3 / C-4（各归 N-02 / N-03）；不新增契约文件；不改任何实现仓。
## 阻塞与升级
- N-04 产物缺失或与 ecs.md §M4 三维定义冲突 → 停止第 3 条，其余照做，交回物写明。
## 交回格式
按共同执行规范五段。

### 验收项（Workflow）
1. C-2′ 规定 NetEntityId 由 Runtime 发号并含宿主自铸拒绝用例
2. C-2′ 规定绑定记录只五元组并含会话号混入拒绝用例
3. C-2′ 内嵌声明表与 N-04 产物 sha256 一致，无手写示例
4. verify-wire 与 spec-lint 通过，输出在交回物

---

## N-02 [程序·协议/公共][Wave 0b] C-1′ 修订：FullSnapshot.stateBlocks 为 Room 路径唯一快照载体、dimensions 为生成物、connection_superseded 通知形状

- category：`architecture / contract`
- 拥有范围：`engine/wire/gameplay-command-envelope-v1.json`、`.spec/decisions/ADR-049-*.md`（追加修订记录段）、`.spec/decisions/ADR-045-*.md`（追加「Room 路径不适用」注记）、`eng/verify-wire.mjs` 中 envelope 用例。
- 前置：N-04 产物；不依赖 N-01。

### 执行提示词（与 N-01 同结构，差异如下）
## 产品背景与已锁决策
- ADR-056 §5 §6：事件必须真广播；Room 路径 FullSnapshot 以 C-1 `stateBlocks` 为准；顶号通知必须上网线。
- 现状：C# 宿主发 ADR-045 五字段体（`MvpEnvelopeWriter.cs:33-41`），客户端按 C-1 解析 `stateBlocks`（`GameplayCodec.cs:153`）；`TakeoverNotice` 只在字典。
## 详细要求
1. `messages.FullSnapshot.notes` 加固：「Room 路径唯一快照载体；ADR-045 五字段体不是本契约的 FullSnapshot；`stateBlocks` 必须含 Room 内每个活体实体的已复制状态块（entityType 等 replicated 属性）」，并加 `invalidCases`：`full_snapshot_without_state_blocks`、`full_snapshot_adr045_shape`。
2. 新增 s2c 消息 `ConnectionSuperseded`：`required { messageType: const:ConnectionSuperseded, reasonCode: const:connection_superseded, netEntityId: u64, newConnectionGeneration: u64 }`；语义：旧连接收到后服务器再关闭连接；加正用例与「先关后发」的反用例。
3. `mappings.*.dimensions` 改为 `source: "generated-from-field-annotations"` + N-04 产物拷贝 + sha256；`chat.component` 的三维必须与 `ChatComponent` 标注生成结果一致。
4. `mappings.chat.event.notes` 增加验收口径：「验收以客户端实际收到的 `Delta.changedBlocks` 为准；harness 不得由发送计数合成 eventOrder / appliedTicks」。
5. ADR-049 追加修订记录段；ADR-045 追加注记「自 ADR-056 起不约束 Room 路径 FullSnapshot」。
## 验收标准
1. C-1′ 明文规定 `stateBlocks` 为 Room 路径唯一快照载体，并含 ADR-045 形状的拒绝用例
2. C-1′ 定义 `ConnectionSuperseded` 下行消息及先关后发的反用例
3. C-1′ `dimensions` 为生成物且与 N-04 产物 sha256 一致
4. verify-wire 与 spec-lint 通过，输出在交回物

### 验收项（Workflow）：同上 1–4

---

## N-03 [程序·协议/公共][Wave 0a] C-4′ 与 ADR-055 修订：单一定时内核双模式，定时可达面进 native ABI

- category：`architecture / contract`
- 拥有范围：`engine/wire/native-timer-abi-v1.json`、`.spec/decisions/ADR-055-*.md`（追加修订记录段）、`engine/abi/native-abi.json`、`engine/native/modules/sdk-native/src/abi_generated.rs`、`engine/managed/Lumio.Engine.NativeLoader/AbiConstants.g.cs`（后两者只能经 `node eng/generate-abi.mjs` 更新）、`eng/verify-wire.mjs` timer 用例。
- 前置：无（Wave 0a）。

### 执行提示词（差异）
## 产品背景与已锁决策
- ADR-056 §7：定时内核只有一个，在 NativeCore，经 `native-abi.json` 暴露；墙钟到期 + Tick/帧双模式；Game / Server / Client / Worker Timer Manager 只是适配器；C# `ITimerService`、Rust `HostTimer` / `expire_due` 轮询、NativeCore `HostTimerService` 门面收敛。
- Owner 原话（2026-09-02）：「Native 有一个计时器；上层 Game / Server / Worker 各有一个 Timer Manager；核心底层在 Rust Native 库。」
- 现状：C-4 `layers` 写双层各自独立基础设施；NativeCore ADR 0007 写「不进 C ABI」。
## 详细要求
1. C-4′ `layers` 改为「单内核双模式」：`wallClock`（单调毫秒、非确定性、承载重连保留窗与宿主周期任务）与 `tickFrame`（确定性、固定 Tick/Frame）两个模式共用 TimerHandle / CallbackSlot / 错误码；`consumers.reconnectDeadline.layer` 改为 `kernel:wallClock`。
2. C-4′ 新增 `abiSurface` 节：托管侧可达的最小函数集（创建/销毁 manager、注册 dispatch、create/bind slot、scheduleOneShot / scheduleRepeating / cancel、advance(tick) / pump(wallClock)、drain），参数一律不透明句柄与整数，无函数指针（ADR-006 纪律）。
3. `engine/abi/native-abi.json` 按 `abiSurface` 增加对应 root 函数字段（doc 写清参数与错误码），`node eng/generate-abi.mjs` 重生成，新 `DEFINITION_SHA256` 写入交回物；`eng/dev-run` 两次 exit 0。
4. ADR-055 追加修订记录段；ADR-056 §7 「接口 / Schema」填实。
5. 通知 NativeCore：ADR 0007「不进 C ABI」需由 N-08 取代（写入交回物「下游集成入口」）。
## 验收标准
1. C-4′ 以单内核双模式描述分层，重连窗归 `kernel:wallClock`
2. C-4′ 定义 `abiSurface`，无函数指针类型
3. `native-abi.json` 含定时函数集，`generate-abi` 零 diff、新 `DEFINITION_SHA256` 与 `dev-run` 两次 exit 0 输出在交回物
4. verify-wire 与 spec-lint 通过

### 验收项（Workflow）：同上 1–4

---

## N-04 [程序·工具/构建][Wave 0a] Runtime 字段标注类型与声明表生成器（ChatComponent 首个打标组件）

- category：`runtime / tooling`
- 拥有范围：`LumioGameRuntime` 新增 `modules/ecs/src/Lumio.GameRuntime.Ecs/Annotations/`（标注类型）、`tools/gen-declarations/`（生成器）、生成产物 `modules/ecs/generated/attribute-declarations.json`（唯一所有者 = 本卡）、对应测试；不改 `EntityBindingQuery`（归 N-05）。
- 前置：无。

### 执行提示词（差异）
## 你的身份
工具/管线工程师：交付可重复、确定性的生成器与自动校验。
## 产品背景与已锁决策
- ADR-056 §4；ecs.md §M4「字段标记三个互不相干维度、各带默认值、90% 字段零声明、未打标记绝不上网」。
- Owner 示例（2026-09-02）：`[Sync(Scope.Everyone)] [Persist(Mode.Store)] int Hp;`、`[ServerOnly] int Hatred;`、聊天 `LastMessageText` 只 `[Persist]` 不同步。
- 现状：Runtime 零标注类型；标签手写于 C-1 `dimensions`、Runtime `DeclareDefaults()`、Game `ChatComponentSchema`。
## 详细要求
1. 标注类型（命名可自定，但必须覆盖 C-2 三维全部枚举）：持久化 `ephemeral | persistent`、复制 `not-replicated | replicated`、可见性 `server-only | room-public | aoi-scoped | claim-scoped`；缺省 = `ephemeral / not-replicated / server-only`（未打标绝不上网、不入档）。
2. 生成器：扫描指定程序集/源码中的组件类型，输出确定性 JSON（键排序、LF、末尾换行），格式与 C-2 `attributeDeclarations.structure` 一致；同输入两次运行字节相同。
3. 生成器对非法组合报错并退出非零：例如 `replicated` 且 `server-only`；`aoi-scoped` 非 replicated。
4. 在 Runtime 内定义 `ChatComponent` 的标注版本（`lastMessageText` / `lastMessageTick`：`persistent / not-replicated / server-only`；`EntityIdentity.entityType`：`ephemeral / replicated / room-public`），生成产物提交入库；`DeclareDefaults()` 改为从生成产物加载（删除硬编码表）。
5. 测试：生成器确定性、非法组合拒绝、产物与 `DeclareDefaults` 加载一致。
## 验收标准
1. 标注类型覆盖 C-2 三维全部枚举，缺省为不上网不入档
2. 生成器确定性（两次运行 sha256 相同）且拒绝非法组合，测试通过
3. `ChatComponent` 打标并生成产物入库，Runtime 声明注册表改为加载产物、硬编码表删除
4. 产物路径与 sha256 写入交回物供 N-01 / N-02 消费

### 验收项（Workflow）：同上 1–4

---

## N-05 [程序·服务端][Wave 1a] Runtime：绑定/查询唯一实现——发号、身份表登记、宿主消费 API

- category：`runtime / replication`
- 拥有范围：`LumioGameRuntime` `modules/replication/**/Binding/`（`EntityBindingQuery` 与契约类型）、对应测试；公开程序集面 `Lumio.GameRuntime.Replication`。
- 前置：N-01 合入（C-2′）；N-04 合入。

### 执行提示词（差异）
## 产品背景与已锁决策
- ADR-056 §2 §3；C-2′。现状：`EntityBindingQuery.Bind` 只收调用方给的字符串（`BindingContracts.cs:12-18`），身份表只在字符串可解析时登记（`EntityBindingQuery.cs:325-330`）；两宿主各自发号。
## 详细要求
1. 新增发号 API：`Admit(connection, accountId, roomId, entityType) → binding`，由 Runtime 身份表分配 `NetEntityId`（ADR-004 语义：永不复用、墓碑登记），宿主不得传入 NetEntityId；保留 `Bind(request)` 仅供恢复路径且拒绝未登记号。
2. 绑定记录类型只含五元组；任何携带会话号 / 宿主句柄的请求按 C-2′ 用例拒绝 `invalid_binding_shape`。
3. 宿主可消费的接口面（`public`，netstandard2.1 + net10.0）：`Admit / Disconnect / Rebind(takeover|reconnect) / Expire → tombstone / SelfLookup / ResolveByConnection / ResolveByNetEntityId / QueryAttribute / ListBindings(roomId)`；语义严格按 C-2′ 五结局与错误码，`undeclared_attribute` 不得映射为 `unauthorized`。
4. `EntityIdentity.accountId` 不在声明表，查询返回 `undeclared_attribute`。
5. 测试：C-2′ 全部 `testCases` / `invalidCases` 逐例断言；发号唯一性（含跨进程重启读回墓碑后不复用）。
## 验收标准
1. Runtime 发号 API 存在，宿主传入 NetEntityId 被拒绝
2. 绑定记录只五元组，混入会话号被拒绝
3. C-2′ 全部正反用例逐例通过，输出在交回物
4. 对外接口面文档化（README 或 features 文档），供 N-10 消费

### 验收项（Workflow）：同上 1–4

---

## N-06 [程序·服务端][Wave 1a] Runtime：ChatInput 走 IngressCapture → CommandBuffer → 固定 Tick；ChatMessageEvent 从 changedBlocks 生成

- category：`runtime / replication`
- 拥有范围：`LumioGameRuntime` `modules/replication/**/Chat/`、`modules/ecs/**/Ingress/`（如需新建）、对应测试。不改 `Binding/`（N-05）与 `Snapshot/`（N-07）。
- 前置：N-02 合入（C-1′）；N-04 合入。

### 执行提示词（差异）
## 产品背景与已锁决策
- ADR-056 §1 §5；ecs-entity-chat.md §3；C-1′。现状：`ChatRoomWorld` 私有队列（Game `ChatRoomWorld.cs:18`）；Runtime `ChatTypedMapping` 无人消费。
## 详细要求
1. `InputCommand(chat.input)` 解码后进入 Runtime 有界 Ingress（每连接容量 = C-1 `boundedInput`），在 `EcsCommandBufferCommit` 相由 Simulation Owner Thread 执行 `ChatComponent.SetMessage`，同一 Tick 产出 `chat.event` 块进入 `Delta.changedBlocks`。
2. 每发送者每 Tick 至多一条，超出 `chat_rate_exceeded`；文本超 512 字节 `chat_text_too_long`；非 owner 线程写入 fail-stop 零写入。
3. 提供宿主可调用的 `BuildFullSnapshot(roomId, tickId, revision)`（含 `stateBlocks` 全体活体实体）与 `BuildDelta(...)`，宿主直接把字节送上网线，不再自行拼装。
4. 删除 Runtime 内与 Game `ChatRoomWorld` 重复的私有世界/队列（如有）。
5. 测试：C-1′ 全部用例；两轮相同输入 → 相同 `(messageId, roomSequence, appliedTick)` 序列。
## 验收标准
1. 聊天输入经 Runtime Ingress 与命令提交路径，无私有队列
2. `chat.event` 与组件状态同 Tick 提交，`Delta.changedBlocks` 可靠有序
3. `BuildFullSnapshot` 输出含全部活体实体的 `stateBlocks`，C-1′ 用例通过
4. 两轮确定性测试通过，输出在交回物

### 验收项（Workflow）：同上 1–4

---

## N-07 [程序·服务端][Wave 1a] Runtime：EcsPersistSnapshotPipeline 公开落盘/读回 API（ADR-032 记录头），重启读回测试

- category：`runtime / ecs`
- 拥有范围：`LumioGameRuntime` `modules/ecs/**/Snapshot/`、对应测试。
- 前置：N-04 合入（`persistent` 标注决定入档字段）。

### 执行提示词（差异）
## 产品背景与已锁决策
- ADR-056 §8；ecs-entity-chat.md §5；ADR-010 / ADR-032。现状：`EcsPersistSnapshotPipeline` internal、无落盘、无人消费；三条内存旁路。WAL 本切片不做，但快照记录头按 ADR-032 预留兼容。
## 详细要求
1. 公开 API：`CapturePersist(world) → bytes`、`RestorePersist(world, bytes)`；字节格式 = ADR-032 记录头（recordVersion / recordSeq / schemaEpoch / payloadHash / checksum）+ LumioBinV1 规范字节；只包含标注为 `persistent` 的字段。
2. 落盘/读回由调用方指定路径；文件写入原子（临时文件 + rename）；读回校验 checksum 与 schemaEpoch，不匹配拒绝。
3. 测试：进程内往返；**跨进程**：测试 A 落盘 → 独立进程 B 读回 → `ChatComponent.lastMessageText/Tick` 逐实体相等；`historyCount` 恒 0；两轮字节一致。
4. 删除 Runtime 内任何聊天专用快照旁路。
## 验收标准
1. 公开落盘/读回 API，格式含 ADR-032 记录头与 checksum，校验失败拒绝
2. 只有 `persistent` 标注字段入档，聊天历史不入档
3. 跨进程读回测试通过（两个 OS 进程），输出在交回物
4. 两轮快照字节一致

### 验收项（Workflow）：同上 1–4

---

## N-08 [程序·服务端][Wave 1a] NativeCore：定时内核单核双模式与 native ABI 导出，删除 HostTimerService 门面

- category：`nativecore / timer`
- 拥有范围：`LumioNativeCore` `crates/lumio-timer/`、`crates/lumio-native-ffi/`（定时导出）、`.spec/decisions/0007-*.md`（标记被取代 + 新 ADR）、镜像 `docs/architecture/wire/native-timer-abi-v1.json`。
- 前置：N-03 合入（C-4′ + native-abi.json）。

### 执行提示词（差异）
## 产品背景与已锁决策
- ADR-056 §7；C-4′ `abiSurface`。现状：`lumio-timer` 不进 ABI（ADR 0007），`HostTimerService` 门面无人用，消费者只在 crate 内测试。
## 详细要求
1. `TimerManager` 增加 `wallClock` 模式：单调毫秒 deadline、one-shot / repeating、cancel、`pump(now_ms)`；与 `tickFrame` 共用 handle / slot / 错误码；两模式的 handle 空间不互指。
2. `lumio-native-ffi` 按 C-4′ `abiSurface` 导出函数，签名与 `native-abi.json` 逐字段一致；`abi_hash` 校验通过。
3. 删除 `crates/lumio-timer/src/host.rs` 的 `HostTimerService`；ADR 0007 状态改「被取代」，新 ADR 记录「进 ABI」。
4. 测试：现有 30 项保留；新增 wallClock 模式用例（到期回调、cancel、stale handle）；FFI 层用例（托管侧模拟调用）；Fixture 逐例断言。
## 验收标准
1. 内核支持 wallClock 与 tickFrame 双模式，共用 handle / slot / 错误码
2. FFI 导出与 `native-abi.json` 逐字段一致，`abi_hash` 校验通过
3. `HostTimerService` 门面删除，ADR 0007 被取代
4. `cargo test -p lumio-timer -p lumio-native-ffi` 全绿，输出在交回物

### 验收项（Workflow）：同上 1–4

---

## N-09 [程序·服务端][Wave 1b] Game：ChatComponent 成为 Runtime ECS 组件，ChatRoomWorld 退役

- category：`game / server-gameplay`
- 拥有范围：`LumioGame` `modules/server-gameplay/`（含删除 `Chat/ChatRoomWorld.cs`、`ChatComponentSchema.cs`、`EntityChat/GameRoomHost.cs`）、`modules/server-gameplay` 的 csproj 引用。不改 `integration/`（N-12）。
- 前置：N-04、N-06 合入。

### 执行提示词（差异）
## 产品背景与已锁决策
- ADR-056 §1 §4；ecs-entity-chat.md §3。现状：`ChatRoomWorld.cs:11-19` 独立字典世界、私有队列；csproj 零 ProjectReference。
## 详细要求
1. `ChatComponent` 改为 Runtime ECS 组件类型，用 N-04 标注声明三维；`SetMessage` 作为 Runtime 命令提交相内的系统，不再有自己的 `RunTick`。
2. 删除 `ChatRoomWorld`、`ChatComponentSchema`、`GameRoomHost`、`EntityChatSuite` 中重复的世界/队列；csproj 引用 `Lumio.GameRuntime.Ecs` 与 `Lumio.GameRuntime.Replication`。
3. `entity-chat-host` HostEntry 的 10 个 op 若仍需要，改为对 Runtime 世界的薄转发（无自有状态）；否则删除并在交回物说明由 N-10 直接托管。
4. 测试：`ChatComponentSetMessageTests` 迁移到 Runtime 世界上运行；owner 线程 fail-stop 用例保留。
## 验收标准
1. `ChatRoomWorld` / `ChatComponentSchema` / `GameRoomHost` 删除，Game 对 Runtime ECS 引用不为零
2. `ChatComponent` 以标注声明三维，生成产物与 N-04 一致
3. SetMessage 在 Runtime 命令提交相执行，测试迁移通过
4. 无私有队列 / 世界残留（grep 证据在交回物）

### 验收项（Workflow）：同上 1–4

---

## N-10 [程序·服务端][Wave 1b] Server Rust 宿主：纯托管——消费 Runtime 绑定/查询/快照与 NativeCore 定时；Room 广播与顶号通知上网线

- category：`server / rust-host`
- 拥有范围：`LumioServer` `modules/process/src/entity_chat/`、`modules/host-runtime/`、`entity-chat-host/`、`modules/process/tests/`、`.github/workflows/`（新增 cargo acceptance job）、`.spec/`（features 文档同步）。不改 `mvp-host/`（冻结中，归 N-13 裁定）。
- 前置：N-05 N-06 N-07 N-08 N-09 合入。

### 执行提示词（差异）
## 你的身份
服务端工程师；你交付的是接力面，Rust 宿主按 ADR-056 通过后 C# 宿主才允许再冻结。
## 产品背景与已锁决策
- ADR-056 全部八条。现状偏离（文件行号见 `reviews/2026-09-02-rm-00011-architecture-deviation.md` §2）：`host.rs:249-262` 自有绑定表与发号；`host.rs:784-841` 硬编码查询；`host.rs:708-715` 进程内窗口无网线；`host.rs:575-582` 顶号静默；`expire_due` 轮询十处；`discover.rs:7-8,29,48-55`、`browser.rs:11`、`entity_chat_acceptance.rs:12,23` 硬编码路径；`host.rs:880-907` 幽灵实体；`entity_chat_acceptance.rs:320-336` 先写 SUCCESS。
## 详细要求
1. 删除 `host.rs` 内 `rooms / by_account / by_connection / tombstones / next_net_entity_id / instance_key` 与 `read_attribute`；绑定、发号、查询、快照全部经 CoreCLR 调 Runtime（N-05 / N-06 / N-07 公开面），宿主只保存「连接 ↔ Runtime 绑定句柄」的会话表（会话号在此，不在绑定记录）。
2. 五分钟到期改为 NativeCore 定时内核 `wallClock` one-shot 回调触发（经 N-08 ABI），删除 `expire_due` 轮询与 `advance_monotonic`；Tick 节奏改为内核 `tickFrame`。
3. Room 网线：宿主建立 WebSocket 服务面（loopback），每 Tick 把 Runtime `BuildDelta` 字节可靠有序广播给 Room 全部连接；准入/重连时发送 Runtime `BuildFullSnapshot`（含 `stateBlocks`）；顶号时先发 `ConnectionSuperseded` 再关闭旧连接。
4. 幽灵实体：恢复路径不得凭快照新建 `Active` 绑定；未在线实体只恢复组件字段。
5. 硬编码路径全部改为环境变量 + 仓根相对发现，缺失即 BLOCKED；`manifest.conclusion` 只能在 oracle 通过后写。
6. CI：`repository-policy.yml` 增加 `cargo test -p lumio-server-process --locked`（含 acceptance，依赖缺失时以 BLOCKED 明确失败而非跳过）。
7. 测试：TDD；两轮 acceptance 由客户端实际接收构成证据（与 N-12 口径一致）。
## 验收标准
1. `host.rs` 无自有绑定表 / 发号 / 查询 switch，绑定与查询全经 Runtime（grep + 架构测试证据）
2. 到期与 Tick 均由 NativeCore 内核回调触发，`expire_due` 轮询删除
3. Room 广播、含 `stateBlocks` 的 FullSnapshot、`ConnectionSuperseded` 均在网线上可抓包/可由客户端断言
4. 无硬编码开发机路径；CI 含 cargo acceptance；`cargo test` 全绿输出在交回物

### 验收项（Workflow）：同上 1–4

---

## N-11 [程序·客户端][Wave 1b] Client：ReplicaWorld 由 C-1′ FullSnapshot 真实重建；Bot 发言节奏走 Client Timer Manager

- category：`client / replica`
- 拥有范围：`LumioClient` `modules/replica/`、`modules/bot/`（或 Bot 启动器所在模块）、Timer Manager 适配层。不改 Game `integration/entity-chat/web`（N-12）。
- 前置：N-02、N-08 合入；N-10 提供可连接的 Room 网线（联调时）。

### 执行提示词（差异）
## 产品背景与已锁决策
- ADR-056 §5 §6 §7；ecs-entity-chat.md §4。现状：`ReplicaWorld` 已能解析 `stateBlocks` 但从未收到；Bot 节奏是 harness for 循环。
## 详细要求
1. 收到 `FullSnapshot` 时丢弃旧 ReplicaWorld、按 `stateBlocks` 重建全部实体、清空聊天窗、再启用输入；旧代次实体不得残留。
2. 收到 `ConnectionSuperseded` 时停止输入并记录原因，不自动重连。
3. Bot 客户端：发言节奏由 Client Timer Manager（经 N-08 ABI 的 `tickFrame` 模式，N = 5）触发，每次触发提交一条 `chat.input`；trace 记录触发 Tick，供 N-12 断言。
4. Client 侧不得实现第二份定时器或绑定表；只消费。
5. 测试：两客户端 `(MessageId, RoomSequence)` 逐项相同（既有）；重连后旧代次实体为零；Bot 触发 Tick 序列为 5,10,15,…。
## 验收标准
1. FullSnapshot 重建后实体集 = `stateBlocks`，旧代次为零，聊天窗清空
2. `ConnectionSuperseded` 处理并有测试
3. Bot 发言由 Client Timer Manager 触发，trace 含触发 Tick 序列
4. Client 无自建定时器 / 绑定表（grep 证据）

### 验收项（Workflow）：同上 1–4

---

## N-12 [集成][Wave 2] R-00354′：11 场景在 Rust 宿主重跑，证据全部来自客户端与落盘观测

- category：`integration / acceptance`
- 拥有范围：`LumioGame` `integration/entity-chat/`（含 `verify-evidence.mjs`、`web/`、launcher/scenarios）。
- 前置：N-09 N-10 N-11 合入；Server CI 绿。

### 执行提示词（差异）
## 你的身份
集成负责人；你唯一的产出是「别人能复核的证据包」。
## 产品背景与已锁决策
- ADR-056 §5 §6 §8 与「验证 Fixture」段；深审 P1-1：`scenarios.mjs:618-637,653,672-673` 由发送计数合成 eventOrder / appliedTicks / restoredWindow；P2-1：Playwright 只登录账号服；P2-4：oracle SHA 未钉死。
## 详细要求
1. 宿主只能是 `lumio-entity-chat-replay`（N-10）；`GameRoomHost` 与 C# `lumio-mvp-host` 不再是任何 SUCCESS 路径。
2. `eventOrder` / `appliedTicks` 必须来自客户端（Node 客户端 + Playwright 页面）实际收到的 `chat.event`；`restoredWindow` 必须实测客户端窗口；`windowBeforeSnapshot` 必须是窗口长度；删除一切合成字段。
3. 浏览器页面接 Room 网线：`__lumioChat.window.lines` 由收到的事件填充，S6 断言其长度 101 且顺序与 `roomSequence` 一致；`receivedFromNetwork` 语义改为「收到至少一条 chat.event」。
4. S7：宿主进程 A 落盘 → 进程 B 读回 → 查询 `lastMessageText` 逐实体相等；`historyCount` 0；聊天窗不回填。
5. S6 Bot 节奏：断言 Client trace 的触发 Tick 序列；`timerManagerInvoked` 语义改为「Client Timer Manager 触发」，`tickSource` 必须为 `native-kernel/tickFrame`。
6. S8：`ConnectionSuperseded` 在旧连接上被断言收到；重绑同 `NetEntityId`（Runtime 发号形制）。
7. oracle：`verify-evidence.mjs` 自身版本以 sha256 写入 evidence.json 并由 Rust acceptance 校验；两轮独立证据包。
8. 全部路径经环境变量或仓根相对发现；不硬编码。
## 验收标准
1. 11 场景在 `lumio-entity-chat-replay` 上两轮 `ok:true`，证据字段无合成值（源码 grep + 审查）
2. 浏览器页面真实收到 101 条事件并按 `roomSequence` 有序
3. S7 跨进程落盘读回通过；S8 顶号通知被旧连接收到
4. oracle sha256 钉死并被 acceptance 校验；证据包与命令输出在交回物

### 验收项（Workflow）：同上 1–4

---

## N-13 [Review][Wave 2] 整体放行：ADR-056 转 Accepted，Server ADR 0006 取代，独立深审复核

- category：`review / closeout`
- 拥有范围：`LumioGameEngine` `.spec/decisions/ADR-056`、`.spec/reviews/`（新报告）；`LumioServer` `.spec/decisions/0006`（标记被取代 + 新 ADR：C# 冻结以 Rust 按 ADR-056 通过为条件）。
- 前置：N-12 交付。

### 执行提示词（差异）
## 你的身份
独立 reviewer（写的人 ≠ 审的人）：对 N-01…N-12 相对基线的完整 diff 与 N-12 证据包做对抗审查，产出放行/退回。
## 详细要求
1. 逐条核 ADR-056「验证 Fixture」六项，命令输出进报告；缺任何一项不得放行。
2. 逐仓 grep 证明无第二份绑定表 / 声明表 / 定时器 / 快照旁路 / 事件队列。
3. 通过后：ADR-056 状态 Draft → Accepted（新增 PR，索引同步）；Server ADR 0006 标记 Superseded，新 ADR 记录冻结条件已满足；R-00345 评论登记 r3 收口。
4. 不通过：按 P0/P1 退回对应 N-xx，报告落 `.spec/reviews/`。
## 验收标准
1. 审查报告含六项 Fixture 的实际命令输出
2. 「无第二份实现」grep 证据覆盖五仓
3. ADR-056 Accepted 且 Server ADR 0006 处置完成（或明确退回清单）
4. spec-lint 全仓通过

### 验收项（Workflow）：同上 1–4

---

## 落单结果（displayKey 映射）

| 临时编号 | Workflow |
|---|---|
| N-03 | R-00365 |
| N-04 | R-00366 |
| N-01 | R-00367 |
| N-02 | R-00368 |
| N-05 | R-00369 |
| N-06 | R-00370 |
| N-07 | R-00371 |
| N-08 | R-00372 |
| N-09 | R-00373 |
| N-10 | R-00374 |
| N-11 | R-00375 |
| N-12 | R-00376 |
| N-13 | R-00377 |

## 线上对象清单（写入授权范围）

| 对象 | 数量 | 说明 |
|---|---|---|
| Requirement（RM-00011 内） | 13 | N-01 … N-13，`roomId` 直接归属 |
| 验收项（需求验收 / 未提交） | 52 | 每卡 4 条 |
| 里程碑归属 MS-00001 | 13 | `PUT /schedule/requirements/{id}/milestone` |
| R-00345 评论 | 1 | 登记 r3 变更控制 |
| 不做 | — | 不流转旧卡、不删除、不建 Room、不建 WorkItem、不传附件 |
