# 2026-08-29 - 八仓同步与 MS-00001 引擎架构进度评估

> **评估范围**：`LumioGameEngineArchitecture` 加七个实现仓库。
> **架构基线**：`LGE-V1.4-2026-08-27`。
> **Workflow 快照**：2026-08-29 15:04 UTC 读取；项目 `lumiogamesengine`。
> **结论性质**：这是一次只读盘点和计划输入，不是里程碑验收，也没有写入 Workflow。

## 1. 执行摘要

当前不能宣称 `MS-00001`「MVP · 多浏览器联机体素世界」完成。架构设计资产已经接近可消费状态，但发布门禁没有变绿，运行时主链和跨进程演示仍未形成。

- **架构设计/Governance**：约 **85% - 90%**。Baseline、ADR、Schema、ID、Fixture、生成器、Root ABI、Canonical/Binary、Trust/Loader/Evidence 和镜像规则都在树上。
- **架构发布可消费性**：**未通过**。`lumio_contract.py validate` 因已发布 Root ABI compiler digest 与锁定 compiler 不一致而失败；下游 pin 不能在此状态下收口。
- **Foundation**：NativeCore 最成熟；Voxel P0 领域代码大部分存在但 artifact hash 未绿；CoreEngine 有 composition/loader 骨架但有两个真实原子性失败；Server 的 platform/wire/transport/auth 基础可靠，WorldSlot/Session/App 缺失；Client 协议基础较完整，浏览器 host/presentation 缺失；GameRuntime 除 observability/generated contracts 外仍几乎没有生产模块；Game 仍为设计阶段。
- **可运行垂直切片**：约 **15% - 20%**（估算，不按卡数或代码行数加权）。还不能证明 `ECS -> Simulation -> Command -> CrossWorldTxn -> GAS -> Voxel -> Replication -> Prediction -> Replay/Save` 闭环。
- **Milestone 台账**：机械统计为 14/70 done（20%），但 70 张里把 Server 的长期 hardening 卡大面积纳入了 MVP。真正的 A1-alpha 直接路径是 17 张：9 done、2 acceptance、1 in_progress、5 backlog。

**最短结论**：先收口架构发布物和所有下游 pin，再并行补 Runtime 主串、Server A1 主串和 Game 内容，最后做 A0/A1 跨进程验收。现在投入 Release Pool、WAL recovery、Migration DAG 或 Unity/HybridCLR 会增加排队而不会缩短 MVP 关键路径。

## 2. 仓库同步结果

八个仓库均执行了：

```text
git fetch --prune origin
git pull --ff-only
```

八仓均输出 `Already up to date.`，并且 `HEAD == origin/main`。本次读取的提交如下：

| 仓库 | HEAD = origin/main | 最新提交主题 | 评估关注点 |
|---|---|---|---|
| `LumioGameEngineArchitecture` | `ce34c8d` | canonical P2 adjudication 文档与 lessons | 生成物/门禁仍需收口 |
| `LumioNativeCore` | `e2a801e` | capability/generated contract conversion | ABI mirror 与 generated hash |
| `LumioVoxelEngine` | `fe2b800` | canonical encoding 与测试增强 | 发布 artifact hash |
| `LumioCoreEngine` | `980c83f` | source-tree projection injectivity 测试增强 | freeze atomicity 两个失败 |
| `LumioGameRuntime` | `ef822a7` | lessons 更新 | 生产模块尚未落地 |
| `LumioServer` | `37d4af4` | MVP Auth Stub | WorldSlot/WS/session/App 缺口 |
| `LumioClient` | `45d804b` | 五文件包 hash 守护与验证 | SDK pin、浏览器 host |
| `LumioGame` | `4b6dd0e` | lessons 更新 | 无生产 C# 实现 |

保留了用户已有的工作区删除状态，没有执行 reset、checkout、push 或其他破坏性操作：

```text
LumioCoreEngine: D .agents/skills, D .claude/agents, D .claude/skills
LumioServer:     D .agents/skills, D .claude/agents, D .claude/skills
```

## 3. 实测验证矩阵

下表只记录实际运行的命令；`build` 不被当作 `test` 证据。

| 仓库/命令 | 结果 | 解释 |
|---|---|---|
| Architecture: `python -m py_compile tools/lumio_contract.py` | PASS | Python 语法可执行 |
| Architecture: `python tools/lumio_contract.py validate` | FAIL | published Root ABI compiler digest `0aaf61...`，锁定 compiler hash `6f51b9...`；需 generator，不可手改 `packages/` |
| Architecture: 临时目录 `generate --out <tmp>` | PASS | 生成 12 artifacts；`compilerHash=6f51b99e...`、`inputHash=d2ed2c9e...`、Root ABI digest `708ccb7e...`；重复生成 outputHash 稳定 |
| Architecture: `node .spec/tools/spec-lint.mjs` | BLOCKED | Windows checkout 无法创建 `.claude/.agents` symlink；不是规范内容失败 |
| Architecture: `node --test .spec/tools/spec-lint.test.mjs` | BLOCKED | 13 项均因 `EPERM: operation not permitted, symlink` |
| NativeCore: `cargo test --workspace` | FAIL | 3 项：`generated_data_matches_mirror_derivation`、`v14_mirror_digest_in_baseline_file_matches_hashed_bytes`、`v14_activity_refs_match_live_execution_truth`；表现为镜像/generator/`.baseline.sha256` 漂移 |
| VoxelEngine: `cargo test --workspace` | FAIL | 2 项：`tamper_fails_then_restore_passes`、`published_hashes_match_locked_packages`；根因 `HashMismatch { artifact_id: "contract-runtime-rust" }` |
| CoreEngine: `cargo test --workspace` | FAIL | `freeze_atomicity` 中 `a_file_occupying_the_output_path_is_also_refused` 与 `validation_failure_after_encoding_still_leaves_nothing_behind` 未按契约拒绝 |
| Server: `dotnet restore build.proj --locked-mode` | PASS | `mvp-host` restore 成功 |
| Server: `dotnet build build.proj -c Release --no-restore` | PASS | 0 warning / 0 error |
| Server: 非 Integration 测试 | PASS | 记录为 312/312；Auth 子集 71/71 |
| Server: `eng/verify-all.ps1` | BLOCKED | 脚本内部调用 `pwsh`，本机没有 `pwsh`；`bash` 也不存在 |
| GameRuntime: production build | PARTIAL | `net10.0` 可构建；`netstandard2.1` 因 `System.Threading.Channels/Channel<>` 失败 |
| GameRuntime: 已构建 DLL 直接测试 | PASS | Observability 39 + GeneratedContracts 4 = 43 项通过 |
| GameRuntime: 标准 `dotnet test` | BLOCKED | .NET 10 的 Microsoft Testing Platform 不再支持旧 VSTest 调用方式 |
| Client: `dotnet test .\LumioClient.slnx` | BLOCKED | 仓库要求 SDK `10.0.400`，本机只有 `10.0.111`，`rollForward=disable` |
| Game: 生产 C# inventory | NOT STARTED | 当前 tracked production C# 文件数为 0 |

当前环境工具：Python 3.12.10、Node 24.18.0、Rust/Cargo 1.98.0；没有 MSVC `link.exe`。环境缺口必须单列，不能改写成“测试通过”。

## 4. Workflow 现状

只读快照显示：

| 项 | 数量 |
|---|---:|
| Requirements | 294 |
| Work Items | 12 |
| Relations | 0 |
| done | 148 |
| in_review | 13 |
| acceptance | 3 |
| in_progress | 1 |
| backlog | 129 |

Room 分布：

| Room | 总数 | done | in_review | acceptance | in_progress | backlog |
|---|---:|---:|---:|---:|---:|---:|
| Architecture | 12 | 10 | 0 | 0 | 0 | 2 |
| NativeCore | 68 | 68 | 0 | 0 | 0 | 0 |
| VoxelEngine | 55 | 28 | 13 | 0 | 0 | 14 |
| CoreEngine | 40 | 13 | 0 | 0 | 0 | 27 |
| GameRuntime | 34 | 8 | 0 | 0 | 0 | 26 |
| Server | 67 | 11 | 0 | 2 | 1 | 53 |
| Client | 16 | 9 | 0 | 0 | 0 | 7 |
| Game | 2 | 1 | 0 | 1 | 0 | 0 |

`MS-00001`：`planned`，目标日 `2026-10-31`，关联 70 张需求；状态为 14 done、2 acceptance、1 in_progress、53 backlog。

### A1-alpha 直接路径

这 17 张才是近期“两个进程互见方块”的直接路径：

| 状态 | 卡片 |
|---|---|
| done (9) | `R-00257`, `R-00258`, `R-00259`, `R-00270`, `R-00271`, `R-00272`, `R-00273`, `R-00274`, `R-00275` |
| acceptance (2) | `R-00260`, `R-00276` |
| in_progress (1) | `R-00277` |
| backlog (5) | `R-00278`, `R-00279`, `R-00280`, `R-00281`, `R-00282` |

依赖顺序是 `R-00277 -> (R-00278 + R-00279) -> R-00280 -> R-00281`；`R-00282` 可并行准备，但必须在最终验收前补齐 Windows SDK/文档证据。`R-00281` 的 A1-beta 部分仍明确受 D-009（上行 dispatch）和状态 payload 公共契约约束，不能由 Server 自行发明字段。

## 5. 引擎架构层完成度

百分比是用于排程的区间估计，按“完成的 MVP 能力面 / 该仓承担的 MVP 能力面”判断；不把文档行数、卡数或生成文件数量当分母。

| 层/仓库 | 完成度估计 | 当前事实 | 还缺什么 |
|---|---:|---|---|
| Architecture 设计 | 85% - 90% | 公共语义和生成链较完整，P2 canonical 裁决已落库 | 重新生成并核对 Root ABI/镜像/baseline；`validate` 绿；可复核 symlink CI |
| Architecture release gate | 0% green | `validate` 明确失败 | 只能通过正式 generator 和全下游 pin 收口 |
| NativeCore | 85% - 90% | I0 crate、ABI、handle、error、capability、memory、job、context、spatial 基础均有代码和测试；Workflow 68/68 done | 消除 mirror/generated/hash 漂移；后续 NativeHeadless 不是 MVP 首要阻塞 |
| VoxelEngine | 70% - 80% P0 | world/chunk/revision/query/mutation/snapshot/restore/port 与大量测试存在 | 修 `contract-runtime-rust` pin；完成 ReferenceVoxelPort 与 Rust differential；可链接宿主证据 |
| CoreEngine | 35% - 45% MVP | composition、Root ABI、platform/manifest/trust/loader 骨架存在；R-00020 done | 修 freeze atomicity；Linux staging/evidence/loader smoke；不要以测试削弱拒绝语义 |
| Server | 30% - 40% A1 | mvp-host platform、generated mirror、wire、transport、auth 已有实现；312 非集成测试通过 | WorldSlotHost、真实 WS carrier、session/admission/reconnect、App/SmokeClient、跨进程验收 |
| Client | 50% - 60% foundation | connection/handshake/replica/prediction/input/session/bot/observability/persistence 有实现 | 安装匹配 SDK；接通远程进程；WASM host、WebGL/Canvas presentation、真实 A1 bot/resync |
| GameRuntime | 10% - 15% production | 主要是 observability 与 generated contracts；其余模块是 README/设计 | ECS、Simulation、Command、Coordination、GAS、Replication、Persistence、Reference Host |
| Game | <10% | 内容规格和模块脚手架设计完成 | `PlaceVoxelAbility`、`DigVoxelAbility`、mapping、config/content、scenario、测试 |
| A0/A1 vertical slice | 15% - 20% | 传输/契约/部分客户端和 voxel 基础可复用 | 单进程权威事务、跨进程 WS、复制、预测、断线 Full Resync、Replay/Save |
| Production hardening | <10% | 只有设计输入 | Release Pool、WAL recovery、Migration DAG、故障矩阵、soak、RemoteDS |

### 当前真正的架构缺口

1. **发布身份未闭合**：架构源临时生成结果与 `packages/`、NativeCore mirror、Voxel/Runtime/Client pin 尚未统一。任何下游继续开发都会产生可预期的 hash churn。
2. **Runtime 是最长的串行关键路径**：至少要完成 ECS、Simulation、Command、Coordination、GAS、Replication 的最小面，才能承载 Game 和 Server 的真实权威逻辑。
3. **Server 只有“宿主基础设施”，没有 World 运行时**：当前 `WorldSlotHost` 和 `Simulation.Reference` 在设计/测试断言中被预留，但生产工程尚未出现。
4. **Game 没有可执行内容**：没有 Ability 就没有可验证的 Place/Dig 命令，也无法证明 CrossWorldTxn 的业务语义。
5. **客户端缺真实远程闭环**：本地模块有状态机和 bot，但尚无“服务端两个进程 + 客户端跨进程复制 + 断线恢复”的实录。
6. **A1-beta 公共契约仍受阻**：D-009/D-011 以及状态 payload 规则未解冻前，禁止实现方私设 `InputCommand` 或状态字段；这属于架构源变更，不是 adapter 细节。

## 6. 推荐 Wave 与退出条件

### Wave 0 - 发布物收口（串行，先于下游重 pin）

1. 在隔离环境运行 `python tools/lumio_contract.py generate --out packages`，先与临时目录逐项比较。
2. 正式生成后一起核对 `compilerHash`、`inputHash`、Root ABI bundle digest、ABI mirror、`.baseline.sha256`、generated data 和七仓 pin。
3. 通过 `python tools/lumio_contract.py validate`；不要手工编辑 `packages/`、`generated_data.rs` 或任何 `.baseline.sha256`。
4. 在启用 symlink 的 Git/CI 环境重跑 `.spec` 两道门禁；Windows 本地失败只作为环境证据。
5. 以稳定的架构提交作为下游七仓重新 pin 的唯一基准，再重新运行各仓 contract/build/test gate。

**退出条件**：架构 `validate` 0；生成两次 outputHash 相同；NativeCore/Voxel/CoreEngine 的契约相关测试不再因上游身份漂移失败。

### Wave 1 - Foundation 核心（Wave 0 后）

可并行的短线：

- GameRuntime：`ecs -> simulation -> command -> coordination -> replication`；GAS 与 persistence 按各自卡面插入，不跨卡发明公共类型。
- Server：完成 `R-00277` WorldSlotHost，再做 `R-00278/R-00279`。
- Voxel：hash 收口、ReferenceVoxelPort、Rust differential。
- CoreEngine：freeze atomicity 修复、Linux staging/loader smoke。
- Client：Headless Bot 远程连接、reconnect/resync；SDK 环境先解锁。
- Game：脚手架后先落 Component/Mapping 与 Place/Dig 内容。

**退出条件**：Runtime 最小接口可被 Server/Game/Client 编译消费；A0 测试能在无 Native 的 `PureHeadless`/`LocalEmbedded` 上跑通。

### Wave 2 - A0 单进程闭环

必须证明：合法 Place/Dig 与拒绝路径、ECS/GAS/Voxel 同一 CrossWorldTxn、幂等重放、Revision 推进、Replica/Prediction 原子应用、Replay hash 一致。任何阶段失败都不能只靠日志声称成功，需保留命令和结果证据。

### Wave 3 - A1-alpha 跨进程

按 `R-00277 -> R-00278/R-00279 -> R-00280 -> R-00281` 推进：真实 `ws://127.0.0.1`、两个独立进程、两个客户端进入同一 WorldSlot、一个客户端挖/放方块后另一个在复制周期内可见、断线后 Full Resync。`R-00282` 补 Windows SDK 和文档证据。

### Wave 4 - A2/A3 与 hardening（A1 通过后）

再接 .NET WASM/browser host、WebGL/Canvas presentation、至少 5 个浏览器客户端、Snapshot/WAL Save/Load、Replay first-difference、慢客户端/丢连接/故障注入。Release Pool、Migration DAG、RemoteDS 仍是 MVP 之后的独立批次。

## 7. 风险与明确决策

- **不把 Auth 写成缺失**：`LumioServer@37d4af4` 已实现 exact-byte verifier、anti-replay、immutable grant 和 permission gate；当前只是 `R-00276` 仍待验收读回。
- **不发布 OperationId namespace**：D-009 仍挡住 dispatch，ADR-040 §7 的“无 OperationId namespace”终态继续有效；不要在 Wave 0 顺带发明它。
- **不绕过公共契约缺口**：需要新字段、错误码、状态或权限表时停在架构源，走 ADR -> Schema/ID -> 正反 Fixture -> generator -> mirrors。
- **不把环境问题变成代码豁免**：Client SDK、`pwsh`、symlink、MSVC linker、MTP runner 分别记录，使用 CI/合适宿主补证据。
- **本轮没有 Workflow 写入**：没有创建卡、修改卡面、评论或流转状态；后续若需新增卡或调整 MS-00001 归属，先取得授权并读回。

## 8. 下一步执行顺序

1. 先批准/安排 Wave 0 的架构 generator 与下游 pin 收口。
2. Wave 0 绿后，按附带的 [`2026-08-29-kickoff-dispatch-prompts.md`](../plans/2026-08-29-kickoff-dispatch-prompts.md) 领取既有卡；Runtime、Server、Voxel、CoreEngine、Client、Game 可按文件边界并行。
3. 每个仓以真实命令输出和 origin 提交号交回；总调度只在 reviewer 通过后把卡流转为 done。
4. 先验收 A0，再验收 A1-alpha；A1-beta、浏览器规模和 hardening 不提前抢跑。
