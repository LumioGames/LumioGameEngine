# 2026-08-29 - MS-00001 下一阶段 Kickoff Dispatch Prompts

> **目标**：为 `MS-00001`「MVP · 多浏览器联机体素世界」准备可直接复制的跨仓派发提示词。
> **状态**：ready after W0/W0.5 gate；本文不执行任务、不创建 Workflow 卡、不流转状态。
> **架构基线**：`LGE-V1.4-2026-08-27`。
> **配套评估**：[`../reviews/2026-08-29-seven-repo-progress-assessment.md`](../reviews/2026-08-29-seven-repo-progress-assessment.md)。
> **最新边界附录**：[`../reviews/2026-08-29-ds-mvp-boundary-reconciliation.md`](../reviews/2026-08-29-ds-mvp-boundary-reconciliation.md)。

## 0. 统一派发规则

每份提示词都继承以下规则：

1. 开工前从 Workflow 读回卡面、前置和当前状态；不要自行创建、重开或修改 Requirement。需要新卡时先向总调度和用户申请授权。
2. 实现会话使用独立 `git worktree`；只修改提示词列出的仓库和文件边界。
3. 证据只引用已经推送到目标仓库 `origin/main` 的提交号；测试证据必须是链接执行的真实输出。
4. 交付物固定为：改动清单、命令和关键输出、known gaps、知识沉淀落点。完成后停在 `acceptance`，`done` 由总调度在 reviewer 通过后流转。
5. 公共字段、状态、错误码、ID、权限表或依赖方向缺失时立即 `BLOCKED` 上报架构源；不得在实现仓库发明第二套契约。
6. `packages/`、`Generated/`、`generated_data.*`、镜像和 `.baseline.sha256` 都按各仓库规则生成，禁止手工改生成物。架构基线变更必须走 ADR -> Schema/ID -> 正反 Fixture -> generator -> mirrors。
7. 不把 `build`、静态 grep 或“测试没有运行”写成测试通过；任何环境阻塞都单独记录。

## 1. 依赖图与波次

```text
W0  Architecture generator/validate
    -> W0.5 DS/MVP profile reconciliation
    -> all downstream contract pins
    -> W1 foundation streams

W1a GameRuntime: config -> ECS -> command -> coordination -> simulation
    \-> GAS + replication (after ECS/identity ports)
W1b Server: WorldSlotHost -> WebSocket carrier + Session
    -> executable App/SmokeClient
W1c Voxel/CoreEngine/NativeCore: hash closure, ReferencePort/differential,
    freeze atomicity, Linux staging/loader evidence
W1d Game/Client: Place-Dig content and headless client wiring

W2 A0: PureHeadless/LocalEmbedded single-process authority transaction
W3 A1-alpha: two independent processes, ws://127.0.0.1, resync
W4 A2/A3: browser host/WebGL, >=5 browsers, Snapshot/WAL/Replay/faults
```

| 波次 | 关键卡片/范围 | 并行边界 | 退出条件 |
|---|---|---|---|
| W0 | Architecture published artifacts and pins | 单串行；索引和 `packages/` 是共享热点 | `validate`=0；两次 generate outputHash 相同；下游契约测试不再因身份漂移失败 |
| W0.5 | DS 定稿与 `MS-00001` bootstrap/DS V1 profile 对齐 | 必须先有 Owner 决策；未决时不派 Server A1 卡 | 明确验收名称、Rust/C# 边界、替换条件、分母和目标日风险 |
| W1-Runtime | `R-00139`, `R-00140`, `R-00149`, `R-00150`, `R-00152`, `R-00154`, `R-00157`, `R-00162`, `R-00164`, `R-00167`, `R-00174`, `R-00176`, `R-00178`, `R-00184`, `R-00187`, `R-00189`, `R-00191`, `R-00192`, `R-00159`, `R-00172` | 同一仓按卡面依赖串行；不同域文件集可并行 | Runtime 最小 ECS/Txn/Tick/Replication 面可被 Host 和 Game 编译消费 |
| W1-Server | `R-00277` -> `R-00278` + `R-00279` -> `R-00280` | WorldSlot 先行；carrier/session 可在其接口稳定后并行 | App 与 SmokeClient 能在本机启动并交换合法 Envelope |
| W1-support | Voxel `R-00142`/`R-00203` follow-up、CoreEngine `R-00021`/`R-00022`、NativeCore pin verification | 文件集互不重叠；均依赖 W0 pin | hash/atomicity/Linux evidence 有真实输出 |
| W1-content | Game `C1-C4`（依据 `R-00259` 设计）、Client `R-00055` 及现有 bot/session 面 | 需要新实现卡时先授权；不重开已完成设计卡 | Place/Dig command 可由 A0 Scenario 驱动 |
| W2 | `R-00181`, `R-00195`, `R-00197`, `R-00199` 加 Server/Client A0 harness | 只能在各域接口稳定后合入 | A0 六类成功/失败断言与 Replay 首差异通过 |
| W3 | `R-00278`..`R-00282` 集成收口，Client remote bot | 跨进程验收单独环境，禁止与普通 build 并发共享 `obj/` | A1-alpha 两客户端互见、断线 Full Resync |
| W4 | browser/WASM/WebGL、Snapshot/WAL、soak/fault matrix | A1-alpha 通过后再开 | MVP DoD 全部有证据 |

**范围裁剪原则**：Server Room 的 67 张卡包含大量 Production Hardening，不作为 W1 的默认分母。Release Pool、WAL recovery 的完整形态、Migration DAG、RemoteDS、Unity/HybridCLR 和 WebTransport 继续留在 W4 以后，除非具体 MVP 验收项证明它们是硬前置。另：最新 DS 定稿要求 Rust DS 核心为 V1 必须；在 W0.5 明确采用 bootstrap profile 前，不得把 C# `mvp-host` 的 A1 结果写成 DS V1 完成。

## 2. Architecture source prompt

**卡片/依据**：`R-00009`（TargetProfile/LoadBackend/PackagingProfile，当前 backlog）、`R-00269`（V1.5 批规划，已完成，仅作边界依据）；正式生成若没有可执行卡，先申请授权。

```text
你负责 LumioGameEngineArchitecture 的发布物收口。先读 R-00009、R-00269、现行 LGE-V1.4 基线和本仓 rules，再在独立 worktree 中执行：
1. 用 `python tools/lumio_contract.py generate --out <tmp>` 取得候选输出，逐项比较 compilerHash、inputHash、Root ABI digest、artifact outputHash、ABI mirror、generated data 和 baseline。
2. 只有在下游 pin 影响清单被核对后，才按授权运行 `python tools/lumio_contract.py generate --out packages`；不得手改 packages、ABI bundle、fixture 或 baseline。
3. 运行 `python tools/lumio_contract.py validate`，并记录两次生成稳定性。Linux/启用 symlink 的环境运行 `.spec` lint；Windows symlink 失败要如实保留为环境证据。
4. 输出新的架构提交号和给七仓的 pin 清单，等待总调度确认后再让下游重 pin。

文件边界：tools/lumio_contract.py 相关生成入口、packages/**、docs/architecture/.baseline.sha256、必要的发布记录；不要修改实现仓文件。
禁止：新增未经 ADR 的公共字段/OperationId namespace；把 D-009/D-011 解冻写进本批；绕过 validate；直接编辑生成物。
验证：python -m py_compile tools/lumio_contract.py；python tools/lumio_contract.py validate；重复 generate 的 outputHash 对比；node .spec/tools/spec-lint.mjs（在可创建 symlink 的宿主）。
交付：候选/正式 hash 表、命令输出、known gaps、origin 提交号；停 acceptance，不自行流转 done。
```

## 3. NativeCore prompt

**状态**：Workflow `RM-00002` 当前 68/68 done，没有新的实现卡。只做 W0 后的消费方重 pin 和验证；若需要改变代码，先申请新卡。

```text
你负责 LumioNativeCore 的 W0 consumer verification。不要重新打开已完成卡，也不要在本仓定义新的公共 ABI。
1. 读取架构仓正式发布提交和 pin 清单，更新本仓只读 ABI mirror/生成记录；生成文件只能由仓内既有 generator 产生。
2. 运行 `cargo xtask gen-contracts`（若卡面要求）并确认 generated_data、ABI mirror、`.baseline.sha256` 和 packages index 的身份一致；发现上游身份仍漂移就 BLOCKED，不用本地常量遮盖。
3. 运行 `cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo build --workspace --benches`。
4. 维护 ADR-040 §7 的结论：`ArchitectureOperationId`/`operation_ids()` 保持空 seam，除非 D-009 已由新 ADR 明确取代；不要顺带发布 OperationId。

文件边界：docs/architecture/** mirror、crates/** 中由 generator 管理的文件、xtask contract wiring；不改 Voxel/Runtime/Server。
验证失败要给出具体 artifact、期望/实际 digest 和恢复后的第二段输出。证据只引用已推送 origin 提交。
```

## 4. VoxelEngine prompt

**卡片**：`R-00142`（generated `IVoxelWorldPort`，当前 in_review）、`R-00203`（P0 MVP review，当前 done，仅作复审基准）；`R-00057`..`R-00064` 为已完成的决策门。

```text
你负责 LumioVoxelEngine 的 P0 consumer closeout，不扩大到 Runtime 或 Server。
1. 等架构 W0 发布后，按 pin 清单更新只读 contract mirror；先复现当前 `contract-runtime-rust` HashMismatch，再更新并恢复，保留失败/恢复两段证据。
2. 保持 world/chunk/revision/query/mutation/snapshot/restore 的现有所有权边界；完成或复核 generated `IVoxelWorldPort` total adapter，并证明错误映射、World+Revision cache key 和 publication 规则没有漂移。
3. 建立 ReferenceVoxelPort 与 Rust world 的 differential 序列：同一 mutation 序列、同一 revision、同一 snapshot hash；不得把 Rust 实现直接塞进浏览器或 Runtime 公共契约。
4. 在可链接宿主运行 P0 decision-gate/benchmark；没有真实 host triple 和数值就保持 BLOCKED，不把设计值写成测量结果。

文件边界：crates/lumio-voxel-{contracts,domain,ops,world,project,migration,test-support}/**、benchmarks/**、docs/evidence/**；不改架构 schemas/ids、不改 LumioGameRuntime。
验证：`cargo test --workspace`；`cargo test --workspace --all-features`（若宿主支持）；生成 adapter/differential 相关测试；记录 hash、host triple 和失败注入输出。
```

## 5. CoreEngine prompt

**卡片**：`R-00020` 已完成；下一串 `R-00021 -> R-00022`（Linux staging 与 Evidence，当前 backlog），`R-00266` 为已完成的失效守护基准。

```text
你负责 LumioCoreEngine 的 loader/staging 最小面。先读取 W0 架构 pin 和 R-00021/R-00022 卡面。
1. 先修复并保留 `freeze_atomicity` 的真实拒绝语义：输出路径被文件占用必须失败；后置 validation 失败必须零残留。不要删弱测试或把异常改成成功结果。
2. 接入 Root ABI/manifest/trust/loader 的只读发布物，完成 Linux staging、EvidenceSet 输入和最小 NativeHeadless smoke；信任根仍是带外登记的 build-plan digest。
3. 运行 `cargo test --workspace`，至少重跑 `cargo test -p lumio-core-composition --test freeze_atomicity`；按卡面在 Linux runner 重跑 staging/evidence 命令和符号/依赖 gate。

文件边界：modules/composition/**、modules/platform/**、modules/loader/**、staging/evidence 相关测试和脚本；不改架构源、不改 Server/Runtime。
禁止：绕过路径冲突、把 sidecar 重编码当防篡改边界、为了通过测试放宽拒绝顺序。
交付需附实际失败修复前后输出、Linux host triple、origin 提交号和未执行项目。
```

## 6. GameRuntime prompt

**卡片串**：

- Config: `R-00139 -> R-00140`。
- ECS: `R-00149 -> R-00150 -> R-00152`。
- Command: `R-00154 -> R-00157 -> R-00162`。
- Coordination: `R-00164 -> R-00167 -> R-00174 -> R-00176`。
- GAS/Replication: `R-00159`、`R-00172`（依赖 ECS identity/port）。
- Simulation: `R-00178 -> R-00184 -> R-00187 -> R-00189 -> R-00191 -> R-00192`。
- Test/integration: `R-00181`、`R-00195 -> R-00197`、`R-00199`。
- `R-00141`（persistence canonical codec）在 Binary profile pin 后执行；它是 A3/Save 的硬前置，不得用 JSON canonical 代替 `LumioBinV1`。

```text
你负责 LumioGameRuntime 的 MVP 最小生产面。先读本仓卡面、架构 v1.4、ADR-002/003/021/027/028/031/047/048 和 W0 pin。
1. 按卡面依赖先完成 config snapshot/activation，再完成 ECS World + LocalEntityId/Generation + owner-thread fail-stop；测试与 production 单向依赖。
2. 继续实现每-processor CommandBuffer/deferred token、CrossWorldTxn prepare/commit/receipt、RevisionVector/SnapshotCut、SimulationSession 的唯一 RunTick 和 13-phase graph。
3. GAS 只提供通用 Type/Handle/Context；Gameplay 状态的单一真相落 ECS。Replication 提供 Mapping Registry、Net/Local identity 和单一权威更新事务。不得在 Runtime 私造 Server/Game DTO。
4. Persistence/Replay 使用已发布 `LumioBinV1` 和架构 canonical profiles；不得用 MessagePack 或 JSON 代替未裁决的公共编码。
5. 每张卡同时添加最小正向和失败测试，证明 owner thread、无副作用 Prepare、幂等 duplicate、fail-stop、determinism 和 first-difference。完成后让 Reference Host 能驱动 64-tick Scenario。

文件边界：modules/{config,ecs,command,coordination,gas,replication,simulation,persistence,testing}/**、src/Lumio.GameRuntime.GeneratedContracts/**、对应 tests/**；不改架构 packages，不改 Server/Client/Game。
验证：仓库既有 `eng/verify-*.ps1|sh`；`dotnet build` 双 TFM；各卡指定 `dotnet test <project> -c Release` 或 Microsoft Testing Platform 直接 runner。若 netstandard2.1 的 Channels/API 不兼容，修复到共同 API 面或按卡面停下，不删除 netstandard target。
交付：卡级测试输出、A0 可调用 port 清单、生成契约 pin、known gaps；不要把只编译 observability 的结果写成 Runtime 完成。
```

## 7. Server prompt

**A1-alpha 卡串**：`R-00277`（当前 in_progress） -> `R-00278` + `R-00279` -> `R-00280` -> `R-00281`；`R-00282` 可并行补证据。`R-00260`/`R-00276` 当前 acceptance，先由总调度验收读回。

```text
你负责 LumioServer 的 A1-alpha，但开工第一步是读取 W0.5 profile 决策。先读 R-00277..R-00282、docs/specs/2026-08-28-mvp-csharp-host-design.md、docs/specs/2026-08-29-ds-server-architecture.md 和架构生成物 pin；若没有明确选择 bootstrap 或 DS V1，立即 BLOCKED，不实现、不流转状态。
若选择 bootstrap profile：本次只实现/验证 C# 语义验收 harness，并在交付物和测试名称中明确“非 DS V1”；不得把它写成 Rust DS 核心完成。
若选择 DS V1 profile：按定稿 §4 将 Rust 连接/准入/会话/预算/WorldSlot 作为生产边界，并先补 Rust↔C# 接缝；现有 C# harness 只能作为对照，不得替代 Rust 验收。
若选择 bootstrap profile，执行以下 C# harness 步骤：
1. 在 `mvp-host/src/Lumio.Server.MvpHost.WorldSlot/**` 落地 WorldSlotHost 聚合根、epoch、13 态、Gate ownership、Quiesce/Stop、owner thread/tick permit、FaultAdjudicator；在 `Simulation.Reference/**` 提供最小 IWorldSimulationPort/reference mutation sink。
2. 在 WorldSlot 接口稳定后实现 `Transport.WebSocket/**` 的 IByteCarrier adapter 和 `Session/**` admission/reconnect/replication orchestration。A1-alpha 使用真实独立进程的 `ws://127.0.0.1`；TLS/WSS 扩展不改变 MVP 公共协议，也不引入未批准依赖。
3. 组装唯一 App root 与 SmokeClient，使用同一 Envelope/serializer/permission gate；Integration.Tests 必须以子进程启动，不得用进程内 fake 代替跨进程证据。
4. A1-alpha 必须证明两个客户端进入同一 WorldSlot、一个 Place/Dig 后另一个在复制周期内可见、丢连接后 Full Resync；A1-beta 的 InputCommand/state payload 若尚未由架构源发布就明确 BLOCKED。
若选择 DS V1 profile，不执行上面 C# 文件步骤；改按卡面在 `modules/process/**`、`modules/auth/**`、`modules/transport/**`、`modules/session/**`、`modules/world-slot/**`、`modules/pacing/**` 及 `coreclr-host` 接缝落地 Rust DS 核心，并用同一跨进程验收场景证明 Rust↔C# 边界。

文件边界（bootstrap）：mvp-host/src/Lumio.Server.MvpHost.{WorldSlot,Simulation.Reference,Transport.WebSocket,Session,App,SmokeClient}/**、对应 tests/**、仅必要的 build.proj glob。文件边界（DS V1）：上述 Rust 模块与 `coreclr-host` 接缝及对应 tests/**。两条路径都不改 generated mirror 的内容、不改 Client/Runtime。
验证：`dotnet restore build.proj --locked-mode`；`dotnet build build.proj -c Release --no-restore`；普通测试逐工程运行并排除 Integration；最后单独运行 Integration.Tests，附 stdout/exit code/trace。`eng/verify-all.ps1` 若缺 pwsh 要记录并在 CI/可用宿主补跑。
禁止：Auth 绕过、把 `ClientReplicaSession` 当 Server 类型、在 Transport 直接引用 Auth/Session/WorldSlot、发明角色权限表或新错误码。
```

## 8. Client prompt

**卡片/依据**：`R-00055`（Vertical Slice 计划，当前 backlog）、`R-00291`（mirror，已完成）、`R-00292`/`R-00294`（当前 backlog）；`R-00253`/`R-00255` 属 Unity/AOT 后续，不是 A1-alpha 前置。

```text
你负责 LumioClient 的 A1-alpha headless consumer 和随后浏览器接入准备。
1. 先解决 SDK 可执行环境：仓库锁定 10.0.400，本机只有 10.0.111 时不要改 global.json 伪造通过；在安装匹配 SDK 的 CI/宿主运行完整 restore/build/test，并记录版本。
2. 使用现有 connection/handshake/replica/prediction/input/session/bot 生产入口连接 Server App；同一生成 Envelope、BaselineAck、DeltaAck、ResyncRequest 和 generation 规则，不在 Client 手写第二套 body。
3. 先做 Headless Bot 的远程跨进程场景：FullSnapshot -> Active -> Place/Dig -> Delta -> gap -> same-connection Resync；断线重连必须新 generation、重新 auth/handshake、无 Resume Token。
4. A1-alpha 通过后再做 .NET WASM host、rAF tick 和 WebGL/Canvas presentation；不要把 Unity/HybridCLR SPIKE 当浏览器完成度。

文件边界：modules/{connection,handshake,replica,prediction,input,session,bot}/**、远程 transport adapter、对应 tests/**；R-00294 只处理既定 Internal/边界，不改公共协议。
验证：匹配 SDK 下 `dotnet build`；各模块 `dotnet test`/MTP runner；Headless Bot 进程命令和 trace；Client 架构测试必须在版本库路径下运行，不能以 worktree 假红为证。
禁止：把服务端 authority 复制进 Client、绕过 Session 状态机直接发包、私自添加 InputCommand/stateBlocks、用本地 LocalEmbedded 结果冒充远程 A1。
```

## 9. Game prompt

**依据**：`R-00259` 已完成（脚手架与 MVP 内容规格）；规格中的 C1-C4 是实现拆分蓝图。实现卡尚需用户授权或使用已有对应卡，不能把设计卡直接重开。

```text
你负责 LumioGame 的最小 MVP 内容实现；开始前先取得 C1-C4 对应的已授权 Workflow 卡。
1. 依照 docs/specs/engineering/module-scaffolding-design.md 和 mvp-placevoxel-content-spec.md 建立 solution/双 TFM 生产工程与测试工程，保持 ServerGameplay 与 ClientGameplay 程序集隔离。
2. C1 先落 BuildResourceAuthority/BuildPermissionAuthority、Client ghost 和三条 Mapping 声明；C2 再落 PlaceVoxelAbility/DigVoxelAbility 的注册、Cost/Targeting/permission callback。Prepare 阶段不得有可见副作用。
3. C3 落 `build.basics` 与 material palette；整数约束、canonical/hash 和 Place(Air) 拒绝必须有正反测试。C4 落 `Scenario.PlaceVoxel.BasicV1` + Dig 扩展、RequiredCapabilities 和 Replay 首差异断言。
4. 只消费 Runtime 的 GAS/ECS/Coordination/IVoxelWorldPort/Mapping API；接口不一致就停并上报，不在 Game 发明替代 port。

文件边界：modules/{server-gameplay,client-gameplay,mapping,gas-content,config,content,scenario}/**、根 solution/工程文件及对应 tests/**；不改架构 schemas、Server Host 或 Client 核心模块。
验证：匹配 SDK 下 `dotnet restore --locked-mode`、`dotnet build -c Release`、各模块 `dotnet test`；运行 Scenario 的成功、资源不足、权限、Revision 冲突、重复命令、Gap/Resync 和 Dig 空块失败路径。
```

## 10. A0/A1 验收编排 prompt

此提示词只在 W1 相关仓库都交回后使用，验收会话与实现会话必须隔离。

```text
你是独立验收会话，不修改实现代码、不替实现者补缺口。
先核对架构提交、七仓 pin、Workflow 卡状态和每仓 origin ancestor。然后按顺序运行：
1. A0 PureHeadless/LocalEmbedded：创建 World/Entity，执行合法 Place 与 Dig；断言 ECS/GAS/Voxel 同一 CrossWorldTxn、单一 Commit Point、Revision 单调、重复命令幂等、失败零副作用。
2. A0 Replica/Prediction：FullSnapshot -> BaselineAck -> Active，注入 Delta gap、权限拒绝、过期 revision，断言同一权威更新事务和 RolledBack/Resync 语义。
3. A0 Replay：相同 seed/input 序列产生相同 Snapshot Hash；故意扰动后输出 Tick/World/Entity/Chunk 的首差异。
4. A1-alpha：独立启动 mvp-host 与至少两个 Smoke/Headless client，使用真实 ws://127.0.0.1；验证互见、断线、新 generation、重新握手和 Full Resync。不得把进程内 loopback 当跨进程证据。
5. 每条验收项保存命令、exit code、关键 stdout/trace、commit SHA 和环境版本；失败则保持 acceptance/BLOCKED 并给出最小复现。
```

## 11. 完成定义与节奏

### 每张卡的 DoD

- 卡面所有验收项逐条有结果，至少一条正向和一条失败/对照组证据。
- 生产代码、测试、镜像和文档在同一 origin 提交链上；生成物由 generator 产生。
- 架构公共语义没有新增未裁决分支；跨仓依赖方向和程序集边界有机器断言。
- reviewer 在独立环境复跑关键命令并放行；总调度读回后才流转 done。

### 建议节奏（相对周次）

| 周次 | 目标 |
|---|---|
| 第 1 周 | W0 generator/validate、pin 清单、SDK/CI/Linux/symlink 环境解阻 |
| 第 2-3 周 | Runtime ECS/Command/Coordination/Simulation；Server WorldSlot/Session；Voxel/CoreEngine 修复；Game C1-C3 |
| 第 4 周 | A0 单进程闭环、Game C4、Reference Host/Replay |
| 第 5-6 周 | A1-alpha App/SmokeClient/远程 Bot/Full Resync |
| 第 7 周以后 | WASM/WebGL、>=5 浏览器、Snapshot/WAL、故障注入和 hardening；只保留能服务 MVP DoD 的项目 |

目标日 `2026-10-31` 仍可作为规划锚点，但前提是 W0/W0.5 在第一周完成且 Runtime/Server/Game 各有稳定 owner。选择 DS V1 profile 后必须重新估算日期；若第 3 周结束仍没有 A0 可运行产物，应立即重估里程碑范围，而不是把长期 Server 卡继续计入“完成度”。
