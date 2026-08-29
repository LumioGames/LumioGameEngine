# 2026-08-30 - Dedicated Server 与 MS-00001 目标剖面对齐审计附录

> **性质**：只读增量审计；不改 BaselineId，不改生成物，不写入 Workflow。
> **Workflow 快照**：`2026-08-29T16:05:28.039Z` UTC（本地日期 2026-08-30）；项目 `LumioGamesEngine`，profile `lumiogamesengine`。
> **实时只读复核**：`2026-08-29T18:33:15.2618284Z` UTC 重新读取 `/requirements`、`/schedule/snapshot`、八个 Room overview 和 WorkItem 列表（需求列表读取始于 `2026-08-29T18:33:00.1355751Z` UTC）；总数、`MS-00001` 归属及关键卡状态与上述快照一致，没有写入。
> **架构锚点**：`origin/main` `d59afa9`；本地审计分支 `HEAD` `e282eb9`（较 `origin/main` 超前 4 个提交）。快照时本地工作区干净；本文件的本轮修改不计入快照证据。
> **架构基线**：`LGE-V1.4-2026-08-27`，本轮零 BaselineId 变更。
> **关联文档**：[`2026-08-29-ds-server-architecture.md`](../specs/2026-08-29-ds-server-architecture.md)、[`mvp-browser-voxel-multiplayer.md`](../plans/mvp-browser-voxel-multiplayer.md)、[`2026-08-29-seven-repo-progress-assessment.md`](2026-08-29-seven-repo-progress-assessment.md)、[`DECISIONS_PENDING.md`](../architecture/DECISIONS_PENDING.md)。

## 1. 执行摘要

当前仍不能宣称 `MS-00001`「MVP · 多浏览器联机体素世界」完成。Dedicated Server 定稿已经把目标架构的所有权画清，但它没有把实现仓的完成度向前推进，也没有使架构发布门变绿。

- DS 定稿的生产边界是 Rust `LumioServer`（准入、连接代次、会话、WS、预算、NativeCore spatial、WorldSlot、进程边界），C# `LumioGameRuntime`（13 相 Tick、ECS/GAS 真值、复制变更集、视野表、语义发送调度），VoxelEngine（自治体素同步，接入同一连接预算、提交点和确认/回滚单元）。
- Adopted MVP 计划仍把 C# `mvp-host` 放在 A0/A1 首发、Rust Host 后置。两份文档对“V1/MVP”的剖面不同，必须先做 W0.5 profile 决策。
- W0 候选生成可以重复得到稳定 `outputHash`，但 `packages/` 仍是旧 compiler identity；`validate` 实测退出码为 1，故架构发布可消费性仍为 **0% green**。
- `MS-00001` 关联 70 张需求，其中 14 done、2 acceptance、1 in_progress、53 backlog。真正的 A1-alpha 直接路径只有 17 张，当前仍卡在 `R-00277` 及其后置卡。
- 推荐采用 **MVP bootstrap profile**：C# 宿主只作为语义/验收 harness，不称为 DS V1；Rust DS V1 单列为后续里程碑。该建议需 Owner 明确确认，未确认前不改卡面、不派新卡。

## 2. DS 定稿与实现仓事实

### 2.1 所有权收口

| 面 | 定稿事实 | 当前可证明范围 |
|---|---|---|
| DS 底层核心 | Rust `LumioServer` 负责 TLS/WSS、未验证限额、连接代次、五步准入、会话、每连接 token bucket、NativeCore spatial、WorldSlot 与进程边界 | Rust 生产核心尚未形成可消费的完整实现 |
| 语义层 | C# `LumioGameRuntime` 负责固定 13 相 Tick、唯一 `GasAndEventFinalize` 提交点、ECS/GAS、视野表、变更集与发送调度 | `origin/main` 主要仍是 observability/generated contracts；生产 Runtime 主串尚未落地 |
| 体素同步 | VoxelEngine 独立自治；共享连接/带宽、提交点和确认/回滚单元；“没收到不等于空气” | 既有 Voxel 状态机和 P0 基础存在，客户端可见三态没有单独的需求卡 |
| 持久化与观测 | 沿用既有 ADR；耐久档位和丢失边界仍属于 D-005 Confirmation Record，不由 `DurabilityAck` 单卡替代 | 机制卡分散在 Runtime/Server/Voxel，政策尚未被单独冻结 |

### 2.2 八仓锚点

所有实现仓在复核时均为 `HEAD == origin/main`；本地差异只作为环境信息，不计入完成证据。

| 仓库 | `origin/main` | 本地差异/审计口径 |
|---|---|---|
| `LumioGameEngineArchitecture` | `d59afa9` | 本地审计分支为 `e282eb9`，领先 4 个提交；复核时干净 |
| `LumioNativeCore` | `e2a801e` | 无实现差异 |
| `LumioVoxelEngine` | `fe2b800` | 无实现差异 |
| `LumioCoreEngine` | `980c83f` | 本地 `.agents/skills`、`.claude/agents`、`.claude/skills` 有用户侧删除/未跟踪差异，排除 |
| `LumioGameRuntime` | `ef822a7` | 本地未跟踪 `modules/ecs/src/`，排除 |
| `LumioServer` | `37d4af4` | 本地 `.agents/skills`、`.claude/agents`、`.claude/skills` 有用户侧删除/未跟踪差异，排除 |
| `LumioClient` | `45d804b` | 无实现差异 |
| `LumioGame` | `4b6dd0e` | 无实现差异 |

实现仓的既有实测证据仍应按上一份七仓评估解释：NativeCore 3 个镜像/生成 hash 测试失败，Voxel 2 个发布 hash 测试失败，CoreEngine 2 个 `freeze_atomicity` 测试失败；Server 的 C# restore/build 与 312 项非 Integration 测试通过，但 `verify-all.ps1` 受本机缺少 `pwsh` 阻塞；Runtime、Client 和 Game 仍分别受生产模块、SDK/测试宿主和内容实现缺口限制。上述本轮未重新执行，不把它们改写成新的通过证据。

## 3. Workflow 快照对账

### 3.1 全局状态

| 项 | 数量 |
|---|---:|
| Requirements | 294 |
| Work Items | 12 |
| Rooms | 8 |
| done | 148 |
| in_review | 13 |
| acceptance | 3 |
| in_progress | 1 |
| backlog | 129 |

### 3.2 Room × 状态与验收缺口

| Room | 总数 | done | in_review | acceptance | in_progress | backlog | 缺验收标准 | 未通过验收项 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Architecture | 12 | 10 | 0 | 0 | 0 | 2 | 6 | 10 |
| NativeCore | 68 | 68 | 0 | 0 | 0 | 0 | 0 | 9 |
| VoxelEngine | 55 | 28 | 13 | 0 | 0 | 14 | 2 | 157 |
| CoreEngine | 40 | 13 | 0 | 0 | 0 | 27 | 2 | 134 |
| GameRuntime | 34 | 8 | 0 | 0 | 0 | 26 | 3 | 246 |
| Server | 67 | 11 | 0 | 2 | 1 | 53 | 1 | 329 |
| Client | 16 | 9 | 0 | 0 | 0 | 7 | 6 | 48 |
| Game | 2 | 1 | 0 | 1 | 0 | 0 | 2 | 0 |

### 3.3 `MS-00001` 与 A1-alpha 路径

`MS-00001` 当前为 `planned`，目标日 `2026-10-31`，关联 70 张需求：

- done：14
- acceptance：2
- in_progress：1
- backlog：53

A1-alpha 直接路径（17 张）如下：

| 状态 | 卡片 |
|---|---|
| done（9） | `R-00257`、`R-00258`、`R-00259`、`R-00270`–`R-00275` |
| acceptance（2） | `R-00260`、`R-00276` |
| in_progress（1） | `R-00277` |
| backlog（5） | `R-00278`、`R-00279`、`R-00280`、`R-00281`、`R-00282` |

依赖顺序是 `R-00277 -> (R-00278 + R-00279) -> R-00280 -> R-00281`；`R-00282` 可以并行准备。`R-00281` 只证明协议/生命周期闭环和其明确的 A1-beta 阻塞声明，**不证明另一客户端已经看见方块内容**。A1-beta 仍受 D-009（上行 dispatch）以及 ADR-028/ADR-049 的状态 payload 公共契约前置约束，Server 不得自行增加字段。

## 4. W0 发布门实测

在临时输出目录执行正式 generator，未触碰 `packages/`；两次生成的各 artifact `outputHash` 稳定，共 12 个 artifact。候选身份为：

```text
compilerHash       6f51b99ebd1b64f3045aff9a3bbd8047bd707ff2d5ec0c9b80e476b83d89e745
inputHash          d2ed2c9e4046fe7bd5ed81e2dd74ef02db6a5671cb971e9163835f763f87bb2f
Root ABI digest    708ccb7e1bd25cb3c66caa3a13bdadfa5446ff4403a0d043333f59e737eae583
Root ABI inputHash 50743b7785279a04976dc414623ccfa41068ba552831f6d2f2768544374a2959
```

已发布 `packages/` 仍记录旧身份：`compilerHash=0aaf61...`、`inputHash=bb95d870...`、Root ABI digest `02dce705...`。对 70 个受跟踪发布文件做原始字节比较：13 个相同、57 个不同，无新增或删除；行尾统一只解释其中一部分，身份元数据差异是真实差异。正式覆盖前必须核对所有下游 pin，不能手挑文件或手改生成物。

本轮命令结果：

| 命令 | 结果 | 解释 |
|---|---|---|
| `python -m py_compile tools/lumio_contract.py` | PASS，exit 0 | Python 语法可执行 |
| `python tools/lumio_contract.py validate` | FAIL，exit 1 | 已发布 Root ABI compiler digest 与锁定 compiler 不一致 |
| `node .spec/tools/spec-lint.mjs` | FAIL，exit 1 | Windows checkout 的 3 个 symlink 校验不匹配 |
| `node --test .spec/tools/spec-lint.test.mjs` | FAIL，exit 1 | 13 项均因创建 symlink 的 `EPERM` 被阻塞 |

`packages/`、下游镜像和 `.baseline.sha256` 本轮均未修改。

## 5. Host profile 冲突与建议

| Profile | 承诺的首发路径 | 可以宣称 | 不能宣称 | 必要动作 |
|---|---|---|---|---|
| **MVP bootstrap** | C# `mvp-host` 承载 A0/A1 语义与验收；Rust DS 后置 | 固定 Tick、事务、复制、预测、断线恢复等语义闭环 | Rust Dedicated Server V1 的底层边界、性能和宿主契约已落地 | 将 `mvp-host` 明确命名为 semantic/acceptance harness；把 Rust DS V1 单列后续里程碑；保留现有 A1 路径 |
| **DS V1** | Rust `LumioServer` 承载准入、连接、会话、WS、预算、WorldSlot；C# Runtime 作为调用方 | 定稿 §4 的真实生产宿主路径 | 当前 C# A1 进度等同于 DS V1 完成 | 把 Rust DS 核心加入关键路径，重排 `R-00277` 以后卡，补 Rust↔C# 接缝、真实 WS、WorldSlot 和重新估算目标日 |

这不是语言偏好，而是验收名称、进程边界和完成度分母不同。若不先选 profile，同一份跨进程测试会同时拥有“完成 MVP”和“未实现 DS V1”两种互相冲突的判定。

**建议**：若首要目标仍是 `2026-10-31` 前证明“两客户端互见方块”，采用 bootstrap profile；对外文案和验收评论必须写明“不等同 DS V1”。若 Owner 要求 `MS-00001` 本身就是 DS V1，则先改计划、依赖图和日期，再继续派 Server 卡。

## 6. 完成度修正

这些是排程用的能力面区间，不是卡数或代码行数：

| 能力面 | 估计 | 依据 |
|---|---:|---|
| 架构语义 / Governance | 约 90% | DS 分层、复制调度、慢客户端、准入顺序和回图条件已有文档落点 |
| 架构发布可消费性 | 0% green | `validate` 仍被 compiler identity 漂移阻塞 |
| Server C# bootstrap | 30%–40% | platform/wire/transport/auth 基础已在，WorldSlot/Session/App/真实跨进程仍缺 |
| Server DS V1（Rust） | <10% | Rust 必须的连接、准入、会话、预算、WorldSlot 生产面尚未形成 |
| `MS-00001` 有效垂直切片 | 15%–20% | profile 未裁决，且 Runtime 主串、跨进程复制和客户端闭环未证实 |

设计定稿本身不增加实现完成度；本机未跟踪的 Runtime 文件也不进入分子。

## 7. W0.5 守门与后续 Wave

顺序固定为：

`W0 generator/validate -> W0.5 profile 决策 -> 下游 pin -> Runtime/Server/Voxel/CoreEngine/Client/Game foundation -> A0 -> A1-alpha`。

W0.5 完成前：

1. 不把 `R-00277` 及其后置 Server 卡标为 DS V1 完成。
2. 不新增或重开 Workflow 卡，不流转现有状态。
3. 不在 Server 中私造 Rust/C# 双套公共协议，不绕过 D-009 或状态 payload 前置。
4. 不把本机未跟踪 Runtime 文件计入完成度。

W0 绿后，bootstrap 路径的短线重点是 Runtime 最小 ECS/Txn/Tick/Replication、`R-00277` 后的 C# A1 宿主、Voxel hash/ReferencePort/differential、CoreEngine freeze atomicity、Client remote bot/resync 和 Game Place/Dig 内容；A0 通过后才进入 A1-alpha。

## 8. 保留、修订与否决清单

下表是审计建议，不是已经执行的 Workflow 状态操作。

| 处置 | 对象 | 建议 |
|---|---|---|
| 保留 | `R-00257`、`R-00258`、`R-00270`–`R-00275` | 已完成的 D-014/WS 登记和 C# host 基础不回退；仍需按 W0 pin 重新核对消费证据 |
| 保留 | D-009、D-011、D-012，以及 `ADR-049` 的公共契约边界 | 继续维持有意封锁或既定重登语义；不以 A1 便利为由私加 dispatch、Auth wire 或 resume token |
| 保留 | Voxel `R-00151`、`R-00153`、`R-00155` | 这些卡覆盖服务器侧 Demand/Fetch/Apply；不把它们重写成客户端三态卡 |
| 修订 | `R-00260`、`R-00277`–`R-00281` | 增加 bootstrap/DS V1 profile 名称、可宣称的验收边界和替换退出条件；不改变公共 Schema |
| 修订 | `mvp-browser-voxel-multiplayer.md`、`2026-08-29-kickoff-dispatch-prompts.md` | 让 C# `mvp-host` 的临时 harness 身份与 DS 定稿的 Rust V1 身份并列，避免继续使用含混的“V1” |
| 修订 | `R-00172`、`R-00214`、`R-00222`、`R-00229`、`R-00279` | 明确 GameRuntime 语义发送调度与 Server token bucket/队列原语的边界，不重复实现预算语义 |
| 修订 | `R-00141`、`R-00174`、`R-00176`、`R-00215`、`R-00221`、`R-00228`、`R-00231`、`R-00236`、`R-00245` | 把 D-005 三档政策与具体文件 adapter、审计 ack、CommitIntent/SnapshotCut/Durable Stream、维护 ack 实现分开，避免以实现卡冒充决策确认 |
| 修订 | `R-00235`、`R-00237`、`R-00240`、`R-00241` | 分开单进程单槽 V1、Rust DS 准入（后置）、登出后完整重登录和 graceful quiesce/drain；live migration 与保留窗口只在独立触发和验收后启用 |
| 否决（不执行） | 新建 Rust/C# 第二套公共协议、OperationId namespace、手改 `packages/` 或以本机文件关闭卡 | 这些路径违反 D-009、生成物和证据纪律；本轮不创建对应卡、不关闭现有卡 |

上面四组 Requirement“修订”对象去重后为 23 张：其中 19 张属于 `MS-00001` 的逐卡表，另外四张支撑性卡（`R-00141`、`R-00172`、`R-00174`、`R-00176`）不属于该里程碑的 70 个 `requirementIds`，但需要同步补充边界说明。`R-00237` 是本轮补入的 DS V1 后置修订对象，并保留其现有 5 个原生验收项供后续复用。仅评论的 Voxel 支撑卡 `R-00155` 也不在这 70 个 ID 中；它不进入更新集，只接收客户端三态缺口评论。`R-00279` 同时出现在发送调度组和 C# 组，按对象只计一次。

`R-00215` 与 `R-00221` 虽然对应 Rust/持久化后置实现，仍保留在“更新”集：本提案只收紧未决 D-005 的 ack/flush 语义，消除隐含的默认耐久档，不启动这两张卡的实现，也不把它们移入 bootstrap 关键路径。

### 8.1 逐卡处置表（推荐 bootstrap profile）

本表按 `MS-00001` 的实时 `requirementIds` 逐行展开，每行恰有一张 Requirement；四种处置互斥：**保留**表示目标和边界可继续使用（执行仍须满足门禁），**修订**表示必须先改卡面/验收再派发，**条件/后置**表示卡仍有价值但不进入推荐 bootstrap 关键路径，**替代**表示现有目标已被新目标取代。本轮没有足够证据把任何一张卡标为替代；表中 70 个 ID 各出现一次。

| ID | 当前状态 | 范围 | 处置 | 理由 / 下一道门 |
|---|---|---|---|---|
| `R-00183` | backlog | Rust Server 原始规划 | 条件/后置 | 这是 Rust DS V1 的来源记录；bootstrap 只保留为后续里程碑输入，不把它当 MVP 完成证据。 |
| `R-00186` | done | 规则镜像同步 | 保留 | 治理资产仍有效；W0 重新 pin 后复核镜像锚点，不回退已完成状态。 |
| `R-00188` | done | Cargo workspace 与 Rust 质量基线 | 保留 | Rust DS 基础仍是后续主线需要的底座；与 C# bootstrap 不冲突。 |
| `R-00190` | done | 架构/依赖守卫 xtask | 保留 | 零环、零无界和封锁守卫仍是两条 profile 都需要的约束。 |
| `R-00200` | done | 非生产 Reference Host testkit | 保留 | 可服务 Rust reference-host 验证；明确不等于生产 DS 或 C# bootstrap 宿主。 |
| `R-00201` | backlog | 只读生成契约包 | 保留 | 两条 profile 都必须消费同一生成物；当前先受 generator/`validate`/下游 pin 门阻塞。 |
| `R-00202` | backlog | `protocol-dispatch` 零实现封锁 | 保留 | D-009 未解封前的硬护栏；不得为 A1 便利新增 RPC、dispatch 或字段。 |
| `R-00206` | backlog | Host profile/capability plan | 条件/后置 | 现有 RemoteDS/LocalEmbedded/LocalSplitProcess 预设有效；先由 W0.5 决定 bootstrap 与 DS V1 的验收别名，再执行 Rust profile 主线。 |
| `R-00207` | backlog | Rust bounded MPSC/SPSC ports | 条件/后置 | 是 Rust DS 的底层原语；C# bootstrap 已有等价 host-runtime 面，不能把两者合并为同一完成证据。 |
| `R-00209` | done | 模块 README/实现映射 | 保留 | 已完成的治理修正继续有效；随 W0 pin 做一次消费路径复核。 |
| `R-00210` | backlog | CoreCLR ABI facade/thread tokens | 条件/后置 | 属 Rust DS/CoreCLR 生产宿主；bootstrap 仅使用 C# reference harness，不提前宣称该能力。 |
| `R-00211` | backlog | 故障 profile 与生产禁用守卫 | 条件/后置 | 只有选 DS V1 或进入 production hardening 才进入关键路径；D-009/D-011 仍按现有封锁处理。 |
| `R-00212` | backlog | Rust 单调时钟与 timer 投递 | 条件/后置 | Rust host 依赖，C# bootstrap 的已交付 timer 面不能替代其 Rust 实现。 |
| `R-00213` | backlog | Rust diagnostic/metrics/trace | 条件/后置 | 生产 DS 观测面后置；bootstrap 使用既有 C# audit/trace 最小面，不把两套实现混为一谈。 |
| `R-00214` | backlog | Rust pacing decision core | 修订 | 明确只拥有 pacing/permit 原语；GameRuntime 的优先级、饥饿上限、频率门和回流语义不得下沉或重复实现。 |
| `R-00215` | backlog | 本地文件原子存储 adapter | 修订 | 保留文件/目录原子性机制，但把 ack/flush 语义绑定到待确认的 D-005 profile，避免 adapter 默认选择耐久档。 |
| `R-00216` | backlog | ReleaseCatalog/ExactRelease | 条件/后置 | Rust Release Pool/production host 能力；不是 bootstrap A1 的最短路径。 |
| `R-00217` | backlog | Rust vendor-neutral envelope core | 条件/后置 | 契约形状有效但属于 Rust DS host；bootstrap 继续使用已登记 C# envelope，不另造公共协议。 |
| `R-00218` | backlog | Rust auth behavior/verifier port | 条件/后置 | D-011 wire/算法仍未冻结；保留 supplier-neutral SPI，待 DS V1 profile 决定后执行。 |
| `R-00219` | backlog | CoreCLR/Runtime/Gameplay scope lifecycle | 条件/后置 | Rust host 的生命周期接缝，不能用 C# mvp-host 的 reference stub 代称生产 CoreCLR。 |
| `R-00220` | backlog | Rust supervision/cancellation/join | 条件/后置 | DS 进程边界的基础能力；bootstrap 只沿 C# harness 的显式线程/退出证据推进。 |
| `R-00221` | backlog | Audit durable pipeline/ack | 修订 | 审计 ack 必须引用 Confirmation Record 的三档 profile，同时保持与 `PersistenceCommitAck` 类型和队列独立。 |
| `R-00222` | backlog | Timer-driven permit scheduler | 修订 | 明确它只调度 Rust permit；不得把 GameRuntime 的语义发送调度或 token bucket 规则复制进 pacing 卡。 |
| `R-00223` | backlog | Rust process components/lifecycle | 条件/后置 | Rust DS 组装层；bootstrap 的 C# App 另有独立验收，不互相替代。 |
| `R-00224` | backlog | Rust auth replay/grant/epoch | 条件/后置 | 生产准入安全面后置；保留 epoch 竞态要求，待 DS V1 选定后执行。 |
| `R-00225` | backlog | Rust control-plane fencing/idempotency | 条件/后置 | DS 控制面后置；bootstrap 只使用显式 test-control 入口，不把它升级成生产控制面。 |
| `R-00226` | backlog | Rust hostfxr/nethost adapter | 条件/后置 | 仅 DS V1/CoreCLR 生产宿主需要；不作为 C# semantic harness 的完成条件。 |
| `R-00227` | backlog | Failure Bundle/emergency path | 条件/后置 | Rust crash-safe hardening 后置；bootstrap 的失败证据仍走现有 trace/audit 约束。 |
| `R-00228` | backlog | Durable streams/CommitAck | 修订 | 先由 D-005 冻结完整、异步 flush、snapshot-only 三档及确认点，再实现四类 stream；禁止隐含默认档。 |
| `R-00229` | backlog | ConnectionRegistry/有界队列 | 修订 | 卡面需把 Rust connection/token-bucket 原语与 GameRuntime 语义发送调度分开；Server 不拥有复制优先级。 |
| `R-00230` | backlog | Injected channel/status report | 条件/后置 | Rust 控制面实现后置；不作为 C# test-control 或 bootstrap trace 的替代。 |
| `R-00231` | backlog | Recovery/Checkpoint/Migration adapter | 修订 | 明确各 D-005 profile 的可恢复范围与损失边界，并把 live migration 作为触发条件而非默认 V1 行为。 |
| `R-00232` | backlog | Signal/process watchdog/crash evidence | 条件/后置 | Rust 进程 hardening 后置；保留独立 watchdog 与证据要求。 |
| `R-00233` | backlog | Release member health/report | 条件/后置 | Release Pool/production hardening 后置；bootstrap 只需 C# App readiness。 |
| `R-00234` | backlog | Rust LocalEmbedded carrier | 条件/后置 | 是 Rust reference-host/DS 适配器；C# bootstrap 的跨进程路径不能由它代验。 |
| `R-00235` | backlog | WorldSlot aggregate/epoch/quota | 修订 | 明写 V1 一进程一 Release、单槽运行；多槽只是预留，不能把未来多世界能力混入当前验收。 |
| `R-00236` | backlog | Persistence durability fault matrix | 修订 | 故障终态和 ack 子集必须按 D-005 三档分别判定；不能用未决 policy 直接封闭矩阵。 |
| `R-00237` | backlog | Rust session/admission saga | 修订（DS V1 后置） | 这是 Rust DS V1 的准入主线；保留五步/八步顺序，但必须明确后置 profile、与 C# bootstrap harness 的证据分离，以及重新启用的替换条件。 |
| `R-00238` | backlog | Rust remote carrier/fault decorator | 条件/后置 | 生产远端 carrier 后置；bootstrap 的 `ws://127.0.0.1` C# 路径独立验收。 |
| `R-00239` | backlog | Rust owner thread/tick barrier | 条件/后置 | Rust DS 主链后置；其 owner-thread 纪律仍是未来 profile 的必要条件。 |
| `R-00240` | backlog | Reconnect window/epoch races | 修订 | DS 裁决的 V1 语义是登出加完整重登录；保留窗口只作触发式预留，不能按当前卡面作为默认 V1 能力。 |
| `R-00241` | backlog | Quiesce/aggregate migration/fault | 修订 | 分开 V1 graceful quiesce/drain 与尚未决的 live migration；后者须有产品触发和独立验收。 |
| `R-00242` | backlog | Maintenance command/deadline/idempotency | 条件/后置 | 依赖后置 Rust maintenance 链；保留 deadline/幂等约束。 |
| `R-00243` | backlog | Slot resource/watchdog/soak | 条件/后置 | production WorldSlot hardening 后置；不占 bootstrap 关键路径。 |
| `R-00244` | backlog | Session drain/kick/fault isolation | 条件/后置 | Rust session/maintenance 后置；单连接故障隔离原则继续保留。 |
| `R-00245` | backlog | Maintenance dual durable ack | 修订 | 双 ack 的独立性保留，但 `ReadyToExit` 和 deadline 行为必须引用 D-005 profile 与实际确认记录。 |
| `R-00246` | backlog | Rust process startup/readiness/shutdown | 条件/后置 | Rust production 组装后置；不替代 C# `R-00280` 的 semantic harness。 |
| `R-00247` | backlog | Rust E2E reference-host shell | 条件/后置 | 属 DS/reference-host 证据面，不是 bootstrap A0/A1 关键路径；选 DS V1 后再启用。 |
| `R-00248` | backlog | Rust DAG/queue/source/license gate | 保留 | 治理门禁本身有效；执行时机后置，不把未跑的 Rust gate 算作 bootstrap 通过。 |
| `R-00249` | backlog | Rust LocalEmbedded vertical skeleton | 条件/后置 | reference-host 垂直骨架后置；不得用它替代 C# bootstrap 的真实进程验收。 |
| `R-00250` | backlog | Rust LocalSplitProcess carrier | 条件/后置 | DS reference carrier 后置；C# A1-alpha 的 `ws://` 证据另行判定。 |
| `R-00251` | backlog | Rust maintenance/fault/stale-epoch E2E | 条件/后置 | 依赖 Rust maintenance 与 D-005，属于后续 DS hardening。 |
| `R-00252` | backlog | Rust provisional-default benchmarks | 条件/后置 | 仅测量临时默认、不冻结公共常量；在 DS profile 和实现面具备后执行。 |
| `R-00257` | done | Voxel P2 D-014 confirmation | 保留 | 已完成的 P2 决策记录不回退；其 P2 数值不阻塞 bootstrap，但消费证据仍随 W0 pin 复核。 |
| `R-00258` | done | WebSocket TransportProfile registration | 保留 | 已登记的 WS 档与公共传输契约继续有效；不借此解封 D-009/D-011。 |
| `R-00259` | done | Game scaffold/content design | 保留 | 设计卡仍是 Place/Dig 内容实现的来源；设计完成不等于 Game 生产实现完成。 |
| `R-00260` | acceptance | C# MVP host design | 修订 | 明确 `mvp-host` 是 semantic/acceptance harness，Rust DS V1 是后续 profile；验收名称和替换条件必须写清。 |
| `R-00270` | done | C# host build root/absence gate | 保留 | bootstrap 基础已完成；继续执行既有隔离与缺席清单，不扩充缺席项。 |
| `R-00271` | done | C# generated mirror/schema/fixture | 保留 | 只读镜像纪律有效；W0 绿后重新核对 compiler/input/root ABI pin。 |
| `R-00272` | done | C# host-runtime primitives | 保留 | 是 bootstrap 可复用的语义宿主基础；不能改称 Rust DS runtime。 |
| `R-00273` | done | C# envelope/JSON/gate | 保留 | 继续复用生成契约与 permission gate；不得因 A1-beta 缺字段手写 body。 |
| `R-00274` | done | C# cross-module contracts/audit gate | 保留 | 作为 harness 的跨模块边界；公共 schema 仍归架构仓。 |
| `R-00275` | done | C# transport core/queues | 保留 | connection epoch、队列和故障装饰器可支撑 bootstrap；不宣称 Rust carrier 已落地。 |
| `R-00276` | acceptance | C# auth stub | 保留 | injected verifier/anti-replay/gate 语义可用于 bootstrap；正式 D-011 仍保持未冻结。 |
| `R-00277` | in_progress | C# WorldSlot/reference simulation | 修订 | 卡面必须标注 bootstrap harness 与 DS V1 的不同分母，并锁定单槽、owner-thread 和可宣称的验收边界。 |
| `R-00278` | backlog | C# WebSocket carrier | 修订 | 明确这是 bootstrap C# carrier 的 `ws://` 验收，不是 Rust DS V1 传输生产实现；WSS/生产边界按 profile 写明。 |
| `R-00279` | backlog | C# session/admission/reconnect/replication | 修订 | 同时收窄到 GameRuntime 语义调度与 Server 原语边界，并把 V1 重登录和同连接 Resync 分开。 |
| `R-00280` | backlog | C# App/SmokeClient assembly | 修订 | 明确唯一组装根属于 bootstrap harness，不能把它作为 Rust DS V1 App/进程边界完成证据。 |
| `R-00281` | backlog | C# A1-alpha lifecycle integration | 修订 | 将可宣称范围固定为协议/生命周期闭环；A1-beta 世界状态互见仍受 D-009 与 ADR-028 双前置阻塞。 |
| `R-00282` | backlog | C# standards/Windows SDK evidence | 保留 | 卡面已区分 C# MVP 与 Rust future host；完成条件是补真实 Windows SDK 证据，不是改变 profile。 |

## 9. 四个待补能力面

| 能力面 | 已有覆盖 | 仍缺的可核对断言 | 建议落点 |
|---|---|---|---|
| Runtime 复制发送调度 | `R-00172`、`R-00182`、`R-00214`、`R-00222`、`R-00229`、`R-00279` 分散覆盖映射、预算、队列和会话 | 没有一张卡冻结完整的 GameRuntime 语义束：预算余量、类别优先级、等待时长饥饿上限、频率门/抖动、截断回流和慢客户端阶梯 | 新建一张 GameRuntime replication requirement；Server 只提供 token bucket 余量，不复制其语义 |
| 客户端 Chunk 三态 | Voxel `R-00151`、`R-00153`、`R-00155`；ADR-024/035 已有 `Unallocated/Loading/Ready` 类状态与“缺失不等于空气”原则 | 没有卡明确客户端可见的“未请求 / 在途 / 已到达”状态以及渲染/查询禁令 | 新建一张 Client requirement；优先复用现有消息路径，不新增公开 wire 字段 |
| Confirmation Record / 耐久 profile | D-005、ADR-032/036；`R-00141`、`R-00174`、`R-00176`、`R-00228`、`R-00231`、`R-00236` 分别覆盖编码、事务、快照、流和故障矩阵 | 没有单独记录完整耐久、MVP 异步 flush（有界损失）和 snapshot-only fallback 三档，以及每档的确认点和损失边界 | 新建 Architecture decision requirement；决策确认与实现卡分离 |
| Client RTT/2 与离群剔除 | 现有时钟、节拍和传输卡不等价于该语义；DS 定稿裁决 18 只写原则 | 没有专卡冻结 RTT/2 校正、异常样本剔除、报文捎带方式和可测验收 | 新建 Client requirement；参数归实现仓，只有公共载荷确有缺口时才回 ADR/Schema |

四张新 Requirement 的依赖锚点固定写入各自正文（四组、共八个锚点，不调用关系 API）：

| 新 Requirement（暂定标题） | 正文依赖锚点 | 对应审计评论目标 |
|---|---|---|
| GameRuntime 复制发送调度 | `R-00172`、`R-00279` | `R-00279` |
| Client Chunk 三态 | `R-00151`、`R-00155` | `R-00155` |
| Architecture D-005 三档 Confirmation Record | `R-00141`、`R-00228` | `R-00141` |
| Client RTT/2 与离群剔除 | `R-00055`、`R-00281` | 无（只写正文锚点） |

七条审计评论的职责分别是：`R-00260`（profile 命名）、`R-00235`（WorldSlot 单槽边界）、`R-00240`（V1 重登录）、`R-00241`（quiesce 与 live migration 分离）、`R-00279`（Runtime/Server 调度边界）、`R-00141`（D-005 确认记录）和 `R-00155`（客户端 Chunk 三态）。四张新卡的原生验收项建议见 §12.3；`R-00237` 与 `R-00279` 的现有五项原生验收项只复用和补充，不在新建项数量中重复计算。

## 10. 垂直切片覆盖表

下表把候选卡映射到可独立验收的垂直切片。`wave` 是实施计划，不表示前置已经满足；“可宣称边界”只在对应证据实测后成立。

| 垂直切片 | 现有卡 | 新卡 | 阻塞前置 | 当前可宣称边界 |
|---|---|---|---|---|
| A1-alpha 协议/生命周期闭环 | `R-00260`、`R-00276`、`R-00277`–`R-00281` | 无 | W0 `validate` 与下游 pin；W0.5 选择 bootstrap；按 `R-00277 -> R-00278/R-00279 -> R-00280 -> R-00281` 串行 | 只可宣称 C# bootstrap 的 WS/准入/会话/快照/增量/重连闭环（有独立进程证据时）；不等同 Rust DS V1，也不包含方块互见 |
| A1-beta 方块互见 | `R-00151`、`R-00155`、`R-00172`、`R-00279`、`R-00281` | `NEW-01`、`NEW-02` | D-009、ADR-028/ADR-049 状态 payload 公共契约；Runtime/Client 复制实现；W0 pin | 当前 **BLOCKED**；在公共契约和双端 trace 完成前，不宣称第二客户端看见方块 |
| 断线重登 | `R-00240`、`R-00241`、`R-00279`、`R-00281` | 无 | profile 决策；新 generation、重新 auth/handshake、同连接 Resync 与完整重登录的边界先冻结 | 仅在 trace 同时证明同连接 Resync 与新代次完整重登录后宣称；不引入 Resume Token |
| AOI 双半径 | `R-00172`、`R-00182`、`R-00279` | `NEW-01` | Runtime identity/mapping、AOI 半径策略、预算耦合和实测阈值 | 现阶段无可宣称证据；不能由 A1-alpha 的单场景替代 |
| 慢客户端阶梯 | `R-00214`、`R-00222`、`R-00229`、`R-00279` | `NEW-01` | GameRuntime 调度语义、Server token bucket 原语、队列上限和观测阈值 | 现阶段不宣称公平性、饥饿上限或性能等级 |
| 崩溃恢复与双端 hash | `R-00141`、`R-00174`、`R-00176`、`R-00231`、`R-00236`、`R-00245` | `NEW-03` | D-005 三档裁决；W0 pin；CoreEngine/Voxel hash gate；失败注入与恢复实测 | 当前不宣称耐久、恢复或双端 hash 闭环；需保留失败注入和首差异证据 |

## 11. ADR 候选覆盖矩阵

这是覆盖矩阵，不是自动接受 ADR 或自动创建实现卡；每一行仍需 Owner 在对应 profile 下确认。

| ADR 候选覆盖面 | 现有覆盖 | 本账本落点 | 目标仓库 / wave | 未覆盖的硬边界 |
|---|---|---|---|---|
| DS 会话与准入契约 | `R-00237`（现有 5 项原生验收）及 DS 定稿 §4 | 更新 `R-00237`，标记“修订（DS V1 后置）” | `LumioServer` / W4 DS V1 | 不把 C# harness 的 A1-alpha 证据合并成 Rust DS V1 |
| 复制发送调度契约 | `R-00172`、`R-00182`、`R-00214`、`R-00222`、`R-00229`、`R-00279` | `NEW-01`，并更新边界卡 | `LumioGameRuntime` / W1→W2→W3 | Server 只给预算/permit，不拥有 Runtime 优先级、饥饿和回流语义 |
| 客户端 Chunk 三态 | `R-00151`、`R-00153`、`R-00155`、ADR-024/035 | `NEW-02`，评论 `R-00155` | `LumioClient` / W1-support→W3 | 缺失不等于空气；没有公共字段授权就不改 wire |
| D-005 耐久 profile / Confirmation Record | `R-00141`、`R-00174`、`R-00176`、`R-00228`、`R-00231`、`R-00236`、`R-00245` | `NEW-03`，更新并评论 `R-00141` | `LumioGameEngineArchitecture` 决策→W4 consumers | 完整、异步 flush、snapshot-only 三档必须分别给确认点和损失边界，禁止隐含默认档 |
| 时间同步与 RTT/2 | `R-00055`、`R-00281` 的原则性约束 | `NEW-04`，只在正文写锚点 | `LumioClient` / W1-support→W3/W4 | 参数可在实现仓冻结；公共载荷缺口必须回架构 ADR/Schema |
| 预留位登记（WorldSlot / profile） | DS 定稿与 `R-00235` 的预留语义 | 只做架构定稿登记，不创建实现 Requirement | `LumioGameEngineArchitecture` / W0.5 | 预留位不是实现承诺，不增加 WorkItem 或伪关系 |

## 12. 待授权 Workflow 写入账本（提案，不执行）

以下账本按推荐的 **MVP bootstrap profile** 编制；若 Owner 选择 DS V1，必须按 Rust 关键路径重新计算。评论是独立写入动作，不能由“更新卡面”隐含代替。第 8 节的仓内计划文档修订建议也不属于 Workflow 写入。

**关系端点校正**：当前 Workflow 合同的 `POST /schedule/relations` 只接受 `sourceWorkItemId`、`targetWorkItemId` 和 `type=finish_to_start`，不能把 `R-*` Requirement 作为关系端点；`/object-links` 当前也只有只读 `GET`。因此八个依赖锚点必须写进新卡正文或审计评论中的 displayKey，不计为八次关系写入，也不创建 WorkItem 绕过边界。该判断按 2026-08-30 读取的公开 OpenAPI 合同核对；验收项类型和初始状态则必须在真正写入前重新 GET，不能凭记忆填 ID。

| 写入类型 | 数量 | 精确对象或范围 |
|---|---:|---|
| 更新现有 Requirement | 23 | `R-00141`、`R-00172`、`R-00174`、`R-00176`、`R-00214`、`R-00215`、`R-00221`、`R-00222`、`R-00228`、`R-00229`、`R-00231`、`R-00235`、`R-00236`、`R-00237`、`R-00240`、`R-00241`、`R-00245`、`R-00260`、`R-00277`、`R-00278`、`R-00279`、`R-00280`、`R-00281`；只追加 profile/边界/依赖说明，不做状态流转。 |
| 关闭/否决现有对象 | 0 | 无；没有一张现有卡具备本轮可复核的关闭或否决证据。 |
| 新建 Requirement | 4 | `NEW-01` GameRuntime 复制发送调度；`NEW-02` Client Chunk 三态；`NEW-03` Architecture D-005 三档 Confirmation Record；`NEW-04` Client RTT/2 与离群剔除。Workflow 建单后分配正式 ID。 |
| 新增审计评论 | 7 | `R-00260`、`R-00235`、`R-00240`、`R-00241`、`R-00279`、`R-00141`、`R-00155`。 |
| 新增关系 | 0 | 八个依赖锚点只写正文/评论（发送调度→`R-00172`/`R-00279`；Chunk 三态→`R-00151`/`R-00155`；耐久→`R-00141`/`R-00228`；RTT/2→`R-00055`/`R-00281`）。 |
| 新建 Requirement 的原生 acceptance items | 16（建议） | 每张新卡 4 条，详见 §12.3；创建前重新读取 active type 与 `not_started` status。`R-00237`、`R-00279` 已有各 5 项，当前只复用/补充，不计入这 16 条。 |

因此，基础写入动作是 **34**（23 更新 + 4 新建 + 7 评论）；若按建议同步创建 16 条新卡原生验收项，最终明确写入动作是 **50（34 + 16）**。唯一 Requirement 对象是 **28**（23 个更新对象 + 4 个新建对象 + 仅评论的 `R-00155`）；验收项是其下属对象，不另算 Requirement。关闭/否决、关系、附件、状态流转、Room/里程碑改动和 Baseline 变更均为 0。

### 12.1 更新清单（单号 / wave / 目标仓库 / 前置阻塞）

这里的 wave 是实现计划归属，不是本次授权后的状态流转；所有前置均为待核对条件。

关键对象已读回 UUID：`R-00237` = `01a043cf-cdbe-7a45-bf2a-fa74911a7034`，`R-00279` = `01a04c08-7fcd-7064-943c-ff8c160e1aa4`；其余对象在授权后仍须按 displayKey 逐项 GET 并读回确认。

| 单号 | wave | 目标仓库 | 前置阻塞 |
|---|---|---|---|
| `R-00141` | W4/A3 | `LumioGameRuntime` | W0 validate/pin；D-005 Binary profile |
| `R-00172` | W1→W2/A0→W3 | `LumioGameRuntime` | W0 pin；ECS identity/ports；A1-beta 另需 D-009 |
| `R-00174` | W1 | `LumioGameRuntime` | `R-00164`/`R-00167` 与 W0 pin |
| `R-00176` | W1 | `LumioGameRuntime` | `R-00174` 与 W0 pin |
| `R-00214` | W4/DS V1 | `LumioServer` | 选择 DS V1；Rust pacing 边界 |
| `R-00215` | W4/DS V1 | `LumioServer` | D-005 三档确认 |
| `R-00221` | W4/DS V1 | `LumioServer` | D-005；audit pipeline |
| `R-00222` | W4/DS V1 | `LumioServer` | `R-00214`；Rust timer/permit |
| `R-00228` | W4/DS V1 | `LumioServer` | D-005；Durable Stream 范围 |
| `R-00229` | W4/DS V1 | `LumioServer` | `R-00237`；connection/token boundary |
| `R-00231` | W4/DS V1 | `LumioServer` | D-005；migration 触发条件 |
| `R-00235` | W4/DS V1 | `LumioServer` | W0.5 profile；单进程单槽裁决 |
| `R-00236` | W4/DS V1 | `LumioServer` | D-005；故障矩阵 |
| `R-00237` | W4/DS V1 | `LumioServer` | W0.5 选择 DS V1；Rust host foundation |
| `R-00240` | W4/DS V1 | `LumioServer` | `R-00237`；完整重登录语义 |
| `R-00241` | W4/DS V1 | `LumioServer` | `R-00235`/`R-00240`；产品 migration 触发 |
| `R-00245` | W4/DS V1 | `LumioServer` | D-005；maintenance dual ack |
| `R-00260` | W0.5 | `LumioServer` | Owner profile；现有验收项读回 |
| `R-00277` | W1-Server | `LumioServer` | W0 绿；W0.5 bootstrap 决策 |
| `R-00278` | W1-Server→W3 | `LumioServer` | `R-00277`；carrier 接口稳定 |
| `R-00279` | W1-Server→W3 | `LumioServer` | `R-00277`；Runtime/Server 调度边界 |
| `R-00280` | W1-Server→W3 | `LumioServer` | `R-00278`/`R-00279` |
| `R-00281` | W3 | `LumioServer` | `R-00280`；D-009/ADR-028 payload（A1-beta） |

### 12.2 新建清单（临时编号 / wave / 目标仓库 / 前置阻塞）

| 临时编号与标题 | wave | 目标仓库 | 前置阻塞 |
|---|---|---|---|
| `NEW-01` GameRuntime 复制发送调度 | W1-Runtime→W2/A0→W3/A1-beta | `LumioGameRuntime` | W0 pin；ECS/identity；D-009（beta） |
| `NEW-02` Client Chunk 三态 contract | W1-support→W3 | `LumioClient` | W0 pin；`R-00151`/`R-00155`；不得新增 wire 字段 |
| `NEW-03` D-005 三档 Confirmation Record | W0.5 决策→W4 consumers | `LumioGameEngineArchitecture` | Owner 确认 D-005；ADR/fixture 路径；不启动实现 |
| `NEW-04` Client RTT/2 与离群剔除 | W1-support→W3/W4 | `LumioClient` | W0 pin；`R-00055`/`R-00281`；先检查公共 payload |

### 12.3 新卡原生验收项建议（4 × 4 = 16）

下列是建议写入“验收”节并同步为原生 acceptance items 的 16 条原子判据，不是已写入的线上对象。创建时必须先读取项目 active acceptance type 和 `systemSemantic=not_started` 的 status；类型、状态或权限不满足时，只保留正文并报告缺口。

| 新卡 | 四条原子验收判据 |
|---|---|
| `NEW-01` | 1. 相同 revision/预算输入产生确定性的类别、优先级和等待时长排序，重复运行输出一致。<br>2. 频率门、抖动和饥饿上限有正反测试；截断余量按原 revision 回流且不丢失。<br>3. 慢客户端阶梯在有界队列和明确阈值下产生可复核 trace，不出现类别永久饥饿。<br>4. 契约测试证明 Server 只提供 token bucket 余量/permit，优先级、饥饿和回流语义不在 Server 重复实现。 |
| `NEW-02` | 1. 客户端显式区分 Unrequested、InFlight、Ready，并覆盖请求、到达、失败和重试转换。<br>2. Unrequested/InFlight 不得被渲染或查询为 Ready/Air，保留“没收到不等于空气”。<br>3. 重复、乱序和过期响应不能回退状态或覆盖更新 revision。<br>4. 正反测试复用 `R-00151`/`R-00155` 现有消息路径；若需公共字段则以 BLOCKED 证据停在架构源。 |
| `NEW-03` | 1. 完整耐久档明确确认点、可恢复材料和丢失上界。<br>2. MVP 异步 flush 档明确确认点、允许的有界损失和恢复动作。<br>3. snapshot-only fallback 档明确 snapshot 确认点、损失范围和重放边界。<br>4. 矩阵逐项映射 CommitIntent、SnapshotCut、Durable Stream 与 ack，并拒绝未声明的隐含默认档。 |
| `NEW-04` | 1. 使用单调发送/接收时间和明确采样窗口计算 RTT/2 校正，重复输入结果确定。<br>2. 离群剔除规则、阈值和最小样本数有正反测试，单个异常样本不能造成时钟跳变。<br>3. 优先捎带现有消息字段；发现公共载荷缺口时必须回到 ADR/Schema，不在 Client 私加字段。<br>4. 覆盖正常延迟、离群样本、时钟偏移和恢复场景，并输出可复核的校正/剔除指标。 |

### 12.4 评论清单

| 目标 | 评论主题 | 关联 wave |
|---|---|---|
| `R-00260` | 把 `mvp-host` 命名为 semantic/acceptance harness，列出 Rust DS V1 替换条件 | W0.5 |
| `R-00235` | 单进程单槽 V1 与多槽预留边界 | W0.5→W4 |
| `R-00240` | V1 采用完整重登录，保留窗口只作触发式预留 | W4/DS V1 |
| `R-00241` | graceful quiesce/drain 与 live migration 分离 | W4/DS V1 |
| `R-00279` | GameRuntime 语义发送调度与 Server token bucket/队列边界 | W1→W3 |
| `R-00141` | D-005 Confirmation Record 三档确认点与实现卡分离 | W0.5→W4 |
| `R-00155` | 客户端 Chunk 三态与“缺失不等于空气”缺口 | W1-support→W3 |

### 12.5 状态、关系与对象边界

- 更新只改正文/验收说明，不做 `backlog`、`in_review`、`in_progress`、`acceptance` 或 `done` 流转。
- 新 Requirement 不传 `status`，不启动实现、不创建 WorkItem、不自动归属 Room 或里程碑。
- `R-00237` 与 `R-00279` 各已有 5 个原生验收项；本账本先读回并复用，任何新增/删除项必须另列数量并重新授权。
- 新卡的八个依赖锚点写正文/评论 displayKey；关系 API 写入保持 0。

## 13. 本轮实际动作与证据边界

本轮 Workflow 写入仍为 **0**：没有 `POST`/`PATCH`、状态 transition、评论、附件、关系或新 Requirement。只读取既有 JSON 快照、架构源文档、公开合同和各仓 Git 锚点，并修改了本地审计附录；没有修改 `packages/`，没有 push 架构仓。报告中的 `23/4/7/0/16` 是待授权提案数字，不是线上已落库结果。

在得到 profile 和精确账本授权前，任何实现仓都不应把候选卡、百分比、验收建议或 wave 表当作已落单、已批准或已完成。

## 14. 下一步授权闸门

当前推荐 **MVP bootstrap profile**。请一次性确认以下全部内容：

1. `MS-00001` 采用 bootstrap profile（C# `mvp-host` 仅为 semantic/acceptance harness，Rust DS V1 后置），还是改选 DS V1 profile（改选后本账本作废并重算）。
2. 项目/profile 为 `lumiogamesengine`（项目 `LumioGamesEngine`）。
3. 更新对象为 §12.1 的 23 张指定 Requirement，关闭/否决 0。
4. 新建 §12.2 的 4 张 Requirement，标题分别为 GameRuntime 复制发送调度、Client Chunk 三态、D-005 三档 Confirmation Record、Client RTT/2 与离群剔除。
5. 评论目标严格为 `R-00260`、`R-00235`、`R-00240`、`R-00241`、`R-00279`、`R-00141`、`R-00155`。
6. 关系写入为 0；八个依赖锚点只进正文/评论。
7. 为 4 张新 Requirement 创建共 **16 条**原生 acceptance items（每张 4 条，具体判据见 §12.3）；创建前重新读取 active type/status，不能猜 ID。

按此授权，明确写入动作是 **34 + 16 = 50**，唯一 Requirement 对象 **28**；不含附件、状态流转、WorkItem、Room/里程碑或 Baseline 变更。`R-00237` 与 `R-00279` 现有各 5 项验收只复用，不计入 16 条。

> **授权问题：是否在项目 `lumiogamesengine` 按 MVP bootstrap profile 执行上述 23 张更新、0 关闭/否决、4 张新建、7 条评论、0 关系，并为新卡创建 16 条原生 acceptance items（总计 50 个明确写入动作、28 个唯一 Requirement 对象）？**

在收到明确的 profile、项目、对象清单和数量确认前，继续保持只读；不能据此宣称任务可安全关闭。
