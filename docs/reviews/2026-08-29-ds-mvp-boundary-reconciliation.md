# 2026-08-30 - Dedicated Server 与 MS-00001 目标剖面对齐审计附录

> **性质**：写入后增量审计；本轮只记录已授权写入及读回结果，不改 BaselineId、不改生成物、不写 `packages/`，不执行状态流转。
> **写入前 Workflow 快照**：`2026-08-29T21:18:52.801Z` UTC；profile `lumiogamesengine`，项目 `LumioGamesEngine`（项目 ID `proj_b6979c277715a6c6c490a541ac69709b`）。基线为 294 张 Requirement、8 个 Room、12 个 WorkItem；`MS-00001` 为 `planned`、目标日 `2026-10-31`、恰有 70 张 Requirement（14 done、2 acceptance、1 in_progress、53 backlog）。
> **本轮执行与当前读回**：按已确认的 MVP bootstrap profile，于 `2026-08-30T00:58:30Z`–`2026-08-30T01:01:09.069724Z` UTC 完成 23 张 Requirement 追加、4 张 Requirement 新建、16 个原生 acceptance item 新建和 7 条评论，共 50 个写入动作。`2026-08-30T01:19:28.998Z` UTC 的写入复核确认 298 张 Requirement；`2026-08-30T01:15:44.600Z` UTC 的边界复核确认 24 个 WorkItem、8 个 Room、0 relations。
> **连接身份与授权结果**：`admin@lumio.games`，项目角色为 administrator；CAS PATCH、Requirement POST、acceptance item POST 和评论 POST 均成功并完成读回。没有执行状态 transition、附件、关系、Room 或里程碑写入。
> **写入前目标明细刷新**：`2026-08-29T19:34:56.324Z`–`2026-08-29T19:35:22.410Z` UTC；按 UUID 读取本账本的 23 个更新目标，随后以 3 页 cursor 分页读取完整 294 张 Requirement，作为本轮 CAS 基线。
> **写入前查重复核**：`2026-08-29T21:18:52.801Z` UTC；四个拟建标题在完整 Requirement 列表和 `scope=title` 精确搜索中均为 0 条；完整列表中没有 `workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2` 标记。
> **写入后边界复核**：当前 298 张 Requirement 中新增仅为 `R-00295`–`R-00298`；23 个更新目标均 `stableEqual=true`、`exactAppend=true`、`suffixCount=1`，7 条评论和 16 个 acceptance item 均按 marker 读回。`R-00182` 仍为 VoxelEngine `backlog`（4 个验收项、3 条评论、0 个附件，`updatedAt=2026-08-27T14:52:34Z`）；Room 数量及字段 8→8，`MS-00001` 仍为 `planned`、目标日 `2026-10-31`、70 张需求，relations 仍为 0。
> **验收配置读回**：本轮使用 Requirement active type=`atype_2c92d7e5acc361f7ad82b1733ab4c223`（`需求验收`）和 `not_started` status=`astat_20e2c7f5c6d891ad0966208b55da0372`（`未提交`），16 个新 item 的类型、状态、正文和顺序均读回一致；Quality active type/status 只保留为历史配置证据。
> **架构锚点**：快照证据对应本地审计 checkout `e282eb9`，较 `origin/main` `d59afa9` 超前 4 个提交；当前本地 `HEAD` 为 `6e3d80b`，包含已审查的 LF 字节规范修复以及其后的未提交审计/计划差异。本文件的未提交修改不计入实现完成证据。
> **架构基线**：`LGE-V1.4-2026-08-27`，本轮零 BaselineId 变更。
> **关联文档**：[`2026-08-29-ds-server-architecture.md`](../specs/2026-08-29-ds-server-architecture.md)、[`mvp-browser-voxel-multiplayer.md`](../plans/mvp-browser-voxel-multiplayer.md)、[`2026-08-29-seven-repo-progress-assessment.md`](2026-08-29-seven-repo-progress-assessment.md)、[`DECISIONS_PENDING.md`](../architecture/DECISIONS_PENDING.md)。

## 1. 执行摘要

当前仍不能宣称 `MS-00001`「MVP · 多浏览器联机体素世界」完成。Dedicated Server 定稿已经把目标架构的所有权画清，但它没有把实现仓的完成度向前推进；W0 字节规范虽已在本地双 checkout 复核，官方发布门和下游 pin 仍未收口。

- DS 定稿的生产边界是 Rust `LumioServer`（准入、连接代次、会话、WS、预算、NativeCore spatial、WorldSlot、进程边界），C# `LumioGameRuntime`（13 相 Tick、ECS/GAS 真值、复制变更集、视野表、语义发送调度），VoxelEngine（自治体素同步，接入同一连接预算、提交点和确认/回滚单元）。
- Adopted MVP 计划把 C# `mvp-host` 放在 A0/A1 首发、Rust Host 后置；本轮已按该 MVP bootstrap profile 落账，Rust DS V1 仍单列为后续 profile。
- W0 的首个 CRLF 候选 `753920e` 虽可重复生成，却不是可发布身份；该候选已拒绝。LF 字节规范修复 `6e3d80b` 已在双 checkout 中使 `validate` 与生成稳定性通过，但官方 Ubuntu policy run 和下游 pin 尚未完成，因此架构发布仍未收口。
- `MS-00001` 关联 70 张需求，其中 14 done、2 acceptance、1 in_progress、53 backlog。真正的 A1-alpha 直接路径只有 17 张，当前仍卡在 `R-00277` 及其后置卡。
- 本轮采用 **MVP bootstrap profile**：C# 宿主只作为语义/验收 harness，不称为 DS V1；Rust DS V1 单列为后续里程碑。账本已执行，但没有因此启动实现、派发 WorkItem 或流转状态。

## 2. DS 定稿与实现仓事实

### 2.1 所有权收口

| 面 | 定稿事实 | 当前可证明范围 |
|---|---|---|
| DS 底层核心 | Rust `LumioServer` 负责 TLS/WSS、未验证限额、连接代次、五步准入、会话、每连接 token bucket、NativeCore spatial、WorldSlot 与进程边界 | Rust 生产核心尚未形成可消费的完整实现 |
| 语义层 | C# `LumioGameRuntime` 负责固定 13 相 Tick、唯一 `GasAndEventFinalize` 提交点、ECS/GAS、视野表、变更集与发送调度 | `origin/main` 主要仍是 observability/generated contracts；生产 Runtime 主串尚未落地 |
| 体素同步 | VoxelEngine 独立自治；共享连接/带宽、提交点和确认/回滚单元；“没收到不等于空气” | 既有 Voxel 状态机和 P0 基础存在；客户端可见三态已登记为 `R-00296`，实现证据仍缺 |
| 持久化与观测 | 沿用既有 ADR；耐久档位和丢失边界仍属于 D-005 Confirmation Record，不由 `DurabilityAck` 单卡替代 | `R-00297` 已记录三档政策口径；具体 D-005 决策和消费者实现仍未完成 |

### 2.2 八仓锚点

所有实现仓在复核时均为 `HEAD == origin/main`；本地差异只作为环境信息，不计入完成证据。

| 仓库 | `origin/main` | 本地差异/审计口径 |
|---|---|---|
| `LumioGameEngineArchitecture` | `d59afa9` | 快照证据 checkout 为 `e282eb9`（领先 4 个提交）；当前 `HEAD` `6e3d80b` 含 LF 字节规范修复，审计/计划文档仍有未提交差异 |
| `LumioNativeCore` | `e2a801e` | 无实现差异 |
| `LumioVoxelEngine` | `fe2b800` | 无实现差异 |
| `LumioCoreEngine` | `980c83f` | 本地 `.agents/skills`、`.claude/agents`、`.claude/skills` 有用户侧删除/未跟踪差异，排除 |
| `LumioGameRuntime` | `ef822a7` | 本地未跟踪 `modules/ecs/src/`，排除 |
| `LumioServer` | `37d4af4` | 本地 `.agents/skills`、`.claude/agents`、`.claude/skills` 有用户侧删除/未跟踪差异，排除 |
| `LumioClient` | `45d804b` | 无实现差异 |
| `LumioGame` | `4b6dd0e` | 无实现差异 |

实现仓的既有实测证据仍应按上一份七仓评估解释：NativeCore 3 个镜像/生成 hash 测试失败，Voxel 2 个发布 hash 测试失败，CoreEngine 2 个 `freeze_atomicity` 测试失败；Server 的 C# restore/build 与 312 项非 Integration 测试通过，但 `verify-all.ps1` 受本机缺少 `pwsh` 阻塞；Runtime、Client 和 Game 仍分别受生产模块、SDK/测试宿主和内容实现缺口限制。上述本轮未重新执行，不把它们改写成新的通过证据。

## 3. Workflow 快照对账

### 3.1 全局状态（当前读回）

| 项 | 数量 |
|---|---:|
| Requirements | 298 |
| Work Items | 24 |
| Rooms | 8 |
| done | 148 |
| in_review | 13 |
| acceptance | 3 |
| in_progress | 1 |
| backlog | 133 |

写入前基线为 294 张 Requirement、12 个 WorkItem、129 张 backlog。当前增加的 4 张 Requirement 均为本轮新卡并处于 `backlog`，因此状态总数只体现 backlog +4。

### 3.1.1 WorkItem 索引与时间线缺口

下表保留写入前快照中的 12 个 WorkItem。`Room` 为空表示该 WorkItem 的 `roomId` 为空；`Requirement` 由其 `requirementId` 反查得到。当前 `/work-items` 与 `/schedule/snapshot` 均读回 24 个 WorkItem；额外的 `T-00010`–`T-00021` 创建于 `2026-08-30T00:50:28Z`–`2026-08-30T00:50:52Z`，早于本轮第一次 Requirement PATCH（`2026-08-30T00:58:30Z`），因此不能归因于本轮，也不能声称当前仍只有 12 个。该时间线/历史归属缺口单独保留；本轮没有创建 WorkItem。

| WorkItem | 类型 | 状态 | Room | Requirement |
|---|---|---|---|---|
| `B-00001` | bug | done | CoreEngine | - |
| `B-00002` | bug | done | CoreEngine | - |
| `B-00003` | bug | done | CoreEngine | - |
| `T-00001` | task | done | Client | `R-00055` |
| `T-00002` | task | done | Client | `R-00055` |
| `T-00003` | task | done | Client | `R-00055` |
| `T-00004` | task | todo | Client | `R-00055` |
| `T-00005` | task | todo | Client | `R-00055` |
| `T-00006` | task | done | - | `R-00055` |
| `T-00007` | task | done | - | `R-00254` |
| `T-00008` | task | done | - | `R-00055` |
| `T-00009` | task | todo | - | `R-00255` |

### 3.2 Room × 状态与验收缺口

| Room | 总数 | done | in_review | acceptance | in_progress | backlog | 缺验收标准 | 未通过验收项 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Architecture | 13 | 10 | 0 | 0 | 0 | 3 | 6 | 14 |
| NativeCore | 68 | 68 | 0 | 0 | 0 | 0 | 0 | 9 |
| VoxelEngine | 55 | 28 | 13 | 0 | 0 | 14 | 2 | 157 |
| CoreEngine | 40 | 13 | 0 | 0 | 0 | 27 | 2 | 134 |
| GameRuntime | 35 | 8 | 0 | 0 | 0 | 27 | 3 | 258 |
| Server | 67 | 11 | 0 | 2 | 1 | 53 | 1 | 329 |
| Client | 18 | 9 | 0 | 0 | 0 | 9 | 6 | 56 |
| Game | 2 | 1 | 0 | 1 | 0 | 0 | 2 | 0 |

“缺验收标准/未通过验收项”取当前 Room overview 读回（`2026-08-30T01:15:44.600Z` UTC）；它们是当前总账指标，不把相对写入前的差异单独归因于本轮描述追加。新增卡的 16 个验收项均为 `not_started`，因此会进入相应 Room 的未通过项统计。

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

### 3.4 全量 Requirement 归属与本轮处置口径

写入前的 3 页 cursor 结果包含 **294 张、无重复 ID**；写入后当前读回为 **298 张、无重复 ID**。`MS-00001` 仍只包含 Server Room 的 67 张、Architecture Room 的 2 张和 Game Room 的 1 张，合计 70；其余当前 228 张不能被隐含地当作里程碑卡。下表同时列出本轮已执行触及的对象；“评论目标”不是额外的 Requirement 状态变更。

| Room | 全量 | `MS-00001` | 里程碑外 | 状态（done / in_review / acceptance / in_progress / backlog） | 更新 | 新建 | 评论目标 |
|---|---:|---:|---:|---|---:|---:|---:|
| Architecture | 13 | 2 | 11 | 10 / 0 / 0 / 0 / 3 | 0 | 1 | 0 |
| NativeCore | 68 | 0 | 68 | 68 / 0 / 0 / 0 / 0 | 0 | 0 | 0 |
| VoxelEngine | 55 | 0 | 55 | 28 / 13 / 0 / 0 / 14 | 0 | 0 | 1 |
| CoreEngine | 40 | 0 | 40 | 13 / 0 / 0 / 0 / 27 | 0 | 0 | 0 |
| GameRuntime | 35 | 0 | 35 | 8 / 0 / 0 / 0 / 27 | 4 | 1 | 1 |
| Server | 67 | 67 | 0 | 11 / 0 / 2 / 1 / 53 | 19 | 0 | 5 |
| Client | 18 | 0 | 18 | 9 / 0 / 0 / 0 / 9 | 0 | 2 | 0 |
| Game | 2 | 1 | 1 | 1 / 0 / 1 / 0 / 0 | 0 | 0 | 0 |
| **合计** | **298** | **70** | **228** | **148 / 13 / 3 / 1 / 133** | **23** | **4** | **7** |

本附录对处置采用五种互斥分类：`retain`（边界仍有效，本轮不改正文）、`revise`（列入 23 张已执行更新）、`conditional/post-MVP`（仍保留，但不进入 bootstrap 关键路径）、`superseded`（已有目标被明确替代；本轮为 0）和 `no-action`（与本轮 DS 边界无关，保持原样）。这些是审计分类，不是 Workflow 状态，也不替代实现验收。

此前工作记录中的 `70 + 6` 只表示当时挑出的明细行，**不是全量清单**；全量对账以写入前 294 张、当前 298 张为准。23 张更新卡中，19 张来自 `MS-00001` 的 70 张里程碑卡，4 张来自里程碑外的 `R-00141`、`R-00172`、`R-00174`、`R-00176`；`R-00155` 是评论目标但不在更新集，`R-00055` 只作为新卡正文依赖锚点。因此 23 张更新、7 条评论和 4 张新建仍是互不替代的三组对象集合，且均已按 §12 读回。

### 3.5 跨 Room 相关卡逐项分类

Server Room 的 67 张已在 §8.1 按 `MS-00001` 逐卡列出。下表补齐 DS 边界分析实际引用的其他 Room 卡；未列入本表的卡仍按上表所属 Room 的既有职责处理，不因“no-action”而被视为完成或关闭。

| 边界 / Room | 相关 Requirement | 分类 | 对账结论与写入影响 |
|---|---|---|---|
| GameRuntime 语义链（保留） | `R-00049`、`R-00112`、`R-00127`、`R-00131`、`R-00133`、`R-00138`–`R-00140`、`R-00149`–`R-00150`、`R-00152`、`R-00154`、`R-00157`、`R-00159`、`R-00162`、`R-00164`、`R-00167`、`R-00178`、`R-00181`、`R-00184`、`R-00187`、`R-00189`、`R-00191`–`R-00192`、`R-00195`、`R-00197`、`R-00199`、`R-00284`–`R-00286` | `retain` | Runtime 仍是 Tick/ECS/GAS/事务/测试语义所有者；这些卡没有本轮边界冲突，不进入 PATCH 集。 |
| GameRuntime 复制与耐久锚点 | `R-00141`、`R-00172`、`R-00174`、`R-00176` | `revise` | 四张均是 23 张更新对象；只追加 §12.1.1 的 profile、所有权和依赖说明，不流转状态。 |
| Voxel streaming / apply | `R-00151`、`R-00153`、`R-00155` | `retain` | 继续由 VoxelEngine 负责 Demand/Fetch/Apply；`R-00155` 另为 7 条评论中的一个目标，但不改卡面、不改状态。 |
| Voxel projection boundary | `R-00182` | `retain` | 只读回为 `backlog`、VoxelEngine Room、`updatedAt=2026-08-27T14:52:34Z`，3 条既有评论、0 个附件、4 个既有验收项；其 Revision-safe Source Router 边界与本轮分层相容，因此本轮 `no-action`，不列入 23 张更新、7 条评论或 4 张新建。 |
| Voxel spatial / mesh / integration hardening | `R-00163`、`R-00166`、`R-00193`、`R-00194`、`R-00196`、`R-00198` | `conditional/post-MVP` | 这些卡仍是后续 AOI、mesh/collision、migration 和长稳验证输入；依赖与证据未满足前不并入 bootstrap 账本，也不做状态操作。 |
| Client foundation / slice anchors | `R-00001`、`R-00019`、`R-00031`、`R-00055`、`R-00065`、`R-00067`、`R-00256`、`R-00287`、`R-00288`、`R-00291` | `retain` | 已有基础与测试/镜像卡保持原样；`R-00055` 仅是 `NEW-04` 的正文锚点，不产生独立写入。 |
| Client AOT / WSS / documentation follow-ups | `R-00253`、`R-00254`、`R-00255`、`R-00289`、`R-00292`、`R-00294` | `conditional/post-MVP` | 依赖具体客户端平台或后续 WSS/文档证据；不把它们误计为 A1-beta 已完成，也不加入本轮更新。 |
| Architecture / Game governance gates | `R-00003`–`R-00006`、`R-00008`–`R-00009`、`R-00257`–`R-00259`、`R-00267`–`R-00269`、`R-00283`、`R-00293` | `retain` | 生成契约、TransportProfile、D-014、内容设计和发布门继续有效；W0 pin/发布门仍是前置证据，不因本轮新卡而替代或关闭。 |

除 §8.1 的 Server 逐卡表和上表明确列出的跨 Room 卡外，NativeCore、CoreEngine 及其余独立职责卡在本轮均为 `no-action`：它们的既有状态、验收和失败证据继续有效，不能被本附录的 bootstrap 账本数字覆盖。

## 4. W0 发布门实测（候选复审前快照）

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
| `node .spec/tools/spec-lint.mjs` | FAIL，exit 1 | 输出 3 处 Windows symlink 校验不一致：`.claude\agents`、`.claude\skills`、`.agents\skills` 未解析进 `.spec/` |
| `node --test .spec/tools/spec-lint.test.mjs` | FAIL，exit 1 | 13 项全部在 fixture 创建 symlink 时被 `EPERM` 阻塞（Windows 无创建 symlink 权限） |
| `python3 -m py_compile tools/lumio_contract.py` | FAIL，exit 1 | `python3` 命中 WindowsApps Microsoft Store alias；实际输出为 “Python was not found” |
| `python3 tools/lumio_contract.py validate` | FAIL，exit 1 | 同一 `python3` alias 阻塞，未进入 validator |
| `python -m py_compile tools/lumio_contract.py`（已安装解释器回退） | PASS，exit 0 | Python 语法可执行 |
| `python tools/lumio_contract.py validate`（已安装解释器回退） | FAIL，exit 1 | 已发布 Root ABI bundle compiler digest `0aaf61d65153aadc4ddda1b36fa1b7bfb38373d52e8ba3299457cefe16864bff` 与锁定 compiler hash `6f51b99ebd1b64f3045aff9a3bbd8047bd707ff2d5ec0c9b80e476b83d89e745` 不一致；validator 建议重新 generate |

`packages/`、下游镜像和 `.baseline.sha256` 本轮均未修改。

### 4.1 候选复审与当前字节规范

上述 `6f51b99...` 候选来自 CRLF 工作树，独立 reviewer 已判定其不可发布；不得把该段的失败身份当作当前 pin。修复提交 `6e3d80b` 在 `.gitattributes` 中为编译器、哈希输入和 `packages/**` 建立 LF 字节权威，且已在 `core.autocrlf=true` 与 `false` 的全新 checkout 中复核 `validate`（201 fixtures、0 failures）、双次生成（70/70、0 mismatch）和三方 KAT 均为 exit 0。完整证据见 [`2026-08-30-w0-byte-authority-review.md`](2026-08-30-w0-byte-authority-review.md)；官方 Ubuntu policy run 与下游 re-pin 仍未执行。

## 5. Host profile 冲突与已选裁决

| Profile | 承诺的首发路径 | 可以宣称 | 不能宣称 | 必要动作 |
|---|---|---|---|---|
| **MVP bootstrap** | C# `mvp-host` 承载 A0/A1 语义与验收；Rust DS 后置 | 固定 Tick、事务、复制、预测、断线恢复等语义闭环 | Rust Dedicated Server V1 的底层边界、性能和宿主契约已落地 | 已按本轮账本将 `mvp-host` 命名为 semantic/acceptance harness，并把 Rust DS V1 单列为后续 profile；保留现有 A1 路径 |
| **DS V1** | Rust `LumioServer` 承载准入、连接、会话、WS、预算、WorldSlot；C# Runtime 作为调用方 | 定稿 §4 的真实生产宿主路径 | 当前 C# A1 进度等同于 DS V1 完成 | 把 Rust DS 核心加入关键路径，重排 `R-00277` 以后卡，补 Rust↔C# 接缝、真实 WS、WorldSlot 和重新估算目标日 |

这不是语言偏好，而是验收名称、进程边界和完成度分母不同。本轮已选定 bootstrap，因此同一份跨进程测试只可用于 MVP harness 口径，不能同时充当 DS V1 证据。

**裁决**：`MS-00001` 采用 bootstrap profile；对外文案和验收评论必须写明“不等同 DS V1”。若未来要求 `MS-00001` 本身就是 DS V1，必须另开 profile、重排计划/依赖图并重新估算目标日，本账本不自动延伸。

## 6. 完成度修正

这些是排程用的能力面区间，不是卡数或代码行数：

| 能力面 | 估计 | 依据 |
|---|---:|---|
| 架构语义 / Governance | 约 90% | DS 分层、复制调度、慢客户端、准入顺序和回图条件已有文档落点 |
| 架构发布可消费性 | 未收口 | LF 字节规范已合入并在双 checkout 通过；官方 Ubuntu policy run 与下游 pin 仍待完成 |
| Server C# bootstrap | 30%–40% | platform/wire/transport/auth 基础已在，WorldSlot/Session/App/真实跨进程仍缺 |
| Server DS V1（Rust） | <10% | Rust 必须的连接、准入、会话、预算、WorldSlot 生产面尚未形成 |
| `MS-00001` 有效垂直切片 | 15%–20% | profile 已裁决，但 Runtime 主串、跨进程复制和客户端闭环仍未证实 |

设计定稿本身不增加实现完成度；本机未跟踪的 Runtime 文件也不进入分子。

## 7. W0.5 守门与后续 Wave

顺序固定为：

`W0 generator/validate -> W0.5 profile 决策 -> 下游 pin -> Runtime/Server/Voxel/CoreEngine/Client/Game foundation -> A0 -> A1-alpha`。

在 W0/W0.5 剩余门禁完成前：

1. 不把 `R-00277` 及其后置 Server 卡标为 DS V1 完成。
2. 不新增或重开 Workflow 卡，不流转现有状态。
3. 不在 Server 中私造 Rust/C# 双套公共协议，不绕过 D-009 或状态 payload 前置。
4. 不把本机未跟踪 Runtime 文件计入完成度。

W0 绿后，bootstrap 路径的短线重点是 Runtime 最小 ECS/Txn/Tick/Replication、`R-00277` 后的 C# A1 宿主、Voxel hash/ReferencePort/differential、CoreEngine freeze atomicity、Client remote bot/resync 和 Game Place/Dig 内容；A0 通过后才进入 A1-alpha。

## 8. 保留、修订与否决清单

下表是审计处置口径；它不等同于 Workflow 状态流转，实际写入结果见 §12–§13。

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

上面四组 Requirement“修订”对象去重后为 23 张：其中 19 张属于 `MS-00001` 的逐卡表，另外四张支撑性卡（`R-00141`、`R-00172`、`R-00174`、`R-00176`）不属于该里程碑的 70 个 `requirementIds`，本轮已同步补充边界说明。`R-00237` 是本轮纳入的 DS V1 后置修订对象，并保留其现有 5 个原生验收项供后续复用。仅评论的 Voxel 支撑卡 `R-00155` 也不在这 70 个 ID 中；它不进入更新集，只接收客户端三态缺口评论。`R-00279` 同时出现在发送调度组和 C# 组，按对象只计一次。

`R-00215` 与 `R-00221` 虽然对应 Rust/持久化后置实现，仍保留在“更新”集：本账本只收紧未决 D-005 的 ack/flush 语义，消除隐含的默认耐久档，不启动这两张卡的实现，也不把它们移入 bootstrap 关键路径。

### 8.1 逐卡处置表（推荐 bootstrap profile）

本表按 `MS-00001` 的实时 `requirementIds` 逐行展开，每行恰有一张 Requirement；四种处置互斥：**保留**表示目标和边界可继续使用（执行仍须满足门禁），**修订**表示本轮已改卡面/验收说明、后续派发仍需满足门禁，**条件/后置**表示卡仍有价值但不进入 bootstrap 关键路径，**替代**表示现有目标已被新目标取代。本轮没有足够证据把任何一张卡标为替代；表中 70 个 ID 各出现一次。

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

## 9. 四个已登记能力面与剩余断言

| 能力面 | 已有覆盖 | 仍缺的可核对断言 | 已登记落点 |
|---|---|---|---|
| Runtime 复制发送调度 | `R-00172`、`R-00182`、`R-00214`、`R-00222`、`R-00229`、`R-00279` 分散覆盖映射、预算、队列和会话 | `R-00295` 已冻结完整 GameRuntime 语义束；仍缺实现 trace、预算边界和慢客户端实测 | `R-00295`；Server 只提供 token bucket 余量，不复制其语义 |
| 客户端 Chunk 三态 | Voxel `R-00151`、`R-00153`、`R-00155`；ADR-024/035 已有 `Unallocated/Loading/Ready` 类状态与“缺失不等于空气”原则 | `R-00296` 已冻结客户端可见的“未请求 / 在途 / 已到达”状态；仍缺实现与渲染/查询实测 | `R-00296`；优先复用现有消息路径，不新增公开 wire 字段 |
| Confirmation Record / 耐久 profile | D-005、ADR-032/036；`R-00141`、`R-00174`、`R-00176`、`R-00228`、`R-00231`、`R-00236` 分别覆盖编码、事务、快照、流和故障矩阵 | `R-00297` 已记录完整耐久、MVP 异步 flush 和 snapshot-only fallback 三档；仍缺 Owner 决策落地及消费者证据 | `R-00297`；决策确认与实现卡分离 |
| Client RTT/2 与离群剔除 | 现有时钟、节拍和传输卡不等价于该语义；DS 定稿裁决 18 只写原则 | `R-00298` 已冻结 RTT/2 校正、异常样本剔除、报文捎带方式和可测验收；仍缺实现验证 | `R-00298`；参数归实现仓，只有公共载荷确有缺口时才回 ADR/Schema |

四张新 Requirement（`R-00295`–`R-00298`）的依赖锚点已固定写入各自正文（四组、共八个锚点，不调用关系 API）：

| 新 Requirement（正式单号） | 正文依赖锚点 | 对应审计评论目标 |
|---|---|---|
| `R-00295` GameRuntime 复制发送调度 | `R-00172`、`R-00279` | `R-00279` |
| `R-00296` Client Chunk 三态 | `R-00151`、`R-00155` | `R-00155` |
| `R-00297` Architecture D-005 三档 Confirmation Record | `R-00141`、`R-00228` | `R-00141` |
| `R-00298` Client RTT/2 与离群剔除 | `R-00055`、`R-00281` | 无（只写正文锚点） |

七条审计评论的职责分别是：`R-00260`（profile 命名）、`R-00235`（WorldSlot 单槽边界）、`R-00240`（V1 重登录）、`R-00241`（quiesce 与 live migration 分离）、`R-00279`（Runtime/Server 调度边界）、`R-00141`（D-005 确认记录）和 `R-00155`（客户端 Chunk 三态）。四张新卡的原生验收项见 §12.3；`R-00237` 与 `R-00279` 的现有五项原生验收项只复用和补充，不在新建项数量中重复计算。

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

## 12. 已授权 Workflow 执行账本（已写入并复核）

以下账本按已确认的 **MVP bootstrap profile** 执行；C# `mvp-host` 仅作为 semantic/acceptance harness，Rust DS V1 保持后置。写入窗口为 `2026-08-30T00:58:30Z`–`2026-08-30T01:01:09.069724Z` UTC，随后完成 CAS、对象和子对象读回。评论是独立写入动作，不能由“更新卡面”隐含代替。第 8 节的仓内计划文档修订建议不属于 Workflow 写入。

**关系端点校正**：当前 Workflow 合同的 `POST /schedule/relations` 只接受 `sourceWorkItemId`、`targetWorkItemId` 和 `type=finish_to_start`，不能把 `R-*` Requirement 作为关系端点；`/object-links` 当前也只有只读 `GET`。因此八个依赖锚点已写进新卡正文或审计评论中的 displayKey，没有创建关系或 WorkItem。该判断按 2026-08-30 读取的公开 OpenAPI 合同核对；验收项使用执行前重新 GET 到的 active Requirement acceptance type/status。

| 写入类型 | 数量 | 精确对象或范围 |
|---|---:|---|
| 更新现有 Requirement | 23（已完成） | `R-00141`、`R-00172`、`R-00174`、`R-00176`、`R-00214`、`R-00215`、`R-00221`、`R-00222`、`R-00228`、`R-00229`、`R-00231`、`R-00235`、`R-00236`、`R-00237`、`R-00240`、`R-00241`、`R-00245`、`R-00260`、`R-00277`、`R-00278`、`R-00279`、`R-00280`、`R-00281`；CAS 通过，严格追加 profile/边界/依赖说明，不做状态流转。 |
| 关闭/否决现有对象 | 0 | 无；没有一张现有卡具备本轮可复核的关闭或否决证据。 |
| 新建 Requirement | 4（已完成） | `R-00295` GameRuntime 复制发送调度；`R-00296` Client Chunk 三态；`R-00297` D-005 三档 Confirmation Record；`R-00298` Client RTT/2 与离群剔除。正式单号、时间和读回见下表。 |
| 新增审计评论 | 7（已完成） | `R-00260`、`R-00235`、`R-00240`、`R-00241`、`R-00279`、`R-00141`、`R-00155`；评论 ID 与时间见 §12.4。 |
| 新增关系 | 0 | 八个依赖锚点只写入正文/评论（发送调度→`R-00172`/`R-00279`；Chunk 三态→`R-00151`/`R-00155`；耐久→`R-00141`/`R-00228`；RTT/2→`R-00055`/`R-00281`）。 |
| 新建 Requirement 的原生 acceptance items | 16（已完成） | 每张新卡 4 条，均使用执行前读回的 Requirement type=`atype_2c92d7e5acc361f7ad82b1733ab4c223` 和 status=`astat_20e2c7f5c6d891ad0966208b55da0372`，正文、顺序和状态已读回一致。`R-00237`、`R-00279` 原有各 5 项只复用，不计入这 16 条。 |

因此，本轮实际写入动作是 **50**（23 更新 + 4 新建 + 16 acceptance item + 7 评论）；基础 Requirement/评论动作是 34。唯一 Requirement 对象是 **28**（23 个更新对象 + 4 个新建对象 + 仅评论的 `R-00155`）；验收项是其下属对象，不另算 Requirement。关闭/否决、关系、附件、状态流转、Room 创建/PATCH、里程碑归属和 Baseline 变更均为 0；新卡 POST 中的已读 `roomId` 已包含在四个创建动作内。

### 12.0 执行结果摘要

| 临时编号 | 正式 Requirement | 线上 ID | 创建/读回时间（UTC） | 当前状态 | 原生验收 | 评论 | 附件 |
|---|---|---|---|---|---:|---:|---:|
| `NEW-01` | `R-00295` | `01a0502e-0dda-716e-872c-2e14533f5405` | `2026-08-30T00:59:50Z` | `backlog` | 4 | 0 | 0 |
| `NEW-02` | `R-00296` | `01a0502e-46c5-7958-818e-989acb414a8a` | `2026-08-30T01:00:05Z` | `backlog` | 4 | 0 | 0 |
| `NEW-03` | `R-00297` | `01a0502e-795b-7252-ba3a-63efa54e8865` | `2026-08-30T01:00:18Z` | `backlog` | 4 | 0 | 0 |
| `NEW-04` | `R-00298` | `01a0502e-a5db-7928-8070-6751517bd729` | `2026-08-30T01:00:29Z` | `backlog` | 4 | 0 | 0 |

四张新卡的 `title`、`roomId`、`module`、`category` 与请求字段读回一致；23 张更新卡全部返回 200，且读回 `stableEqual=true`、`exactAppend=true`、`suffixCount=1`。所有新卡保持 `backlog`，本轮没有借写入动作启动实现或改变任何既有状态。

### 12.1 更新清单（单号 / wave / 目标仓库 / 前置阻塞）

这里的 wave 是实现计划归属，不是本次已执行的状态流转；表中前置仍是后续实现/验收的门禁。

关键对象已读回 UUID：`R-00237` = `01a043cf-cdbe-7a45-bf2a-fa74911a7034`，`R-00279` = `01a04c08-7fcd-7064-943c-ff8c160e1aa4`；其余 21 个更新对象也已按 displayKey/UUID 完成 GET 与 CAS 读回，结果汇总见 §12.1.1。

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

### 12.1.1 现有卡的精确追加后缀与 PATCH 规则

下列代码块是 23 张更新卡各自实际使用的**精确追加后缀**；代码围栏不属于 payload。每个 suffix 的字节串严格为 `\n\n` + 代码块内从 `---` 到 `Revision` 行的文本（不再附加空格或换行）。执行时每个 `PATCH` 均以写入前刚刚 GET 到的 `expectedUpdatedAt` 做 CAS，并保持 `existing description + exact suffix`；返回 200 后再次 GET 验证。除 `description` 外没有改动任何字段：没有改 status、owner、priority、risk、module、category、room、里程碑、日期或 estimate。

为便于审计，下表同时记录 CAS 前基线和实际读回的 `updatedAt`；23 行均返回 200，且 `stableEqual=true`、`exactAppend=true`、`suffixCount=1`：

| key | CAS 前 `updatedAt` | 读回 `updatedAt` |
|---|---|---|
| `R-00141` | `2026-08-29T11:59:07Z` | `2026-08-30T00:58:30Z` |
| `R-00172` | `2026-08-27T14:49:27Z` | `2026-08-30T00:58:32Z` |
| `R-00174` | `2026-08-27T14:49:48Z` | `2026-08-30T00:58:36Z` |
| `R-00176` | `2026-08-27T14:51:18Z` | `2026-08-30T00:58:40Z` |
| `R-00214` | `2026-08-27T15:07:21Z` | `2026-08-30T00:58:44Z` |
| `R-00215` | `2026-08-27T15:08:41Z` | `2026-08-30T00:58:47Z` |
| `R-00221` | `2026-08-27T15:12:04Z` | `2026-08-30T00:58:50Z` |
| `R-00222` | `2026-08-27T15:12:28Z` | `2026-08-30T00:58:54Z` |
| `R-00228` | `2026-08-27T15:17:39Z` | `2026-08-30T00:58:57Z` |
| `R-00229` | `2026-08-27T15:18:02Z` | `2026-08-30T00:58:59Z` |
| `R-00231` | `2026-08-27T15:18:56Z` | `2026-08-30T00:59:02Z` |
| `R-00235` | `2026-08-27T15:20:44Z` | `2026-08-30T00:59:04Z` |
| `R-00236` | `2026-08-27T15:21:05Z` | `2026-08-30T00:59:07Z` |
| `R-00237` | `2026-08-27T15:21:27Z` | `2026-08-30T00:59:10Z` |
| `R-00240` | `2026-08-27T15:42:37Z` | `2026-08-30T00:59:13Z` |
| `R-00241` | `2026-08-27T15:43:04Z` | `2026-08-30T00:59:16Z` |
| `R-00245` | `2026-08-27T15:44:56Z` | `2026-08-30T00:59:18Z` |
| `R-00260` | `2026-08-28T11:56:21Z` | `2026-08-30T00:59:22Z` |
| `R-00277` | `2026-08-29T14:26:33Z` | `2026-08-30T00:59:24Z` |
| `R-00278` | `2026-08-29T05:40:12Z` | `2026-08-30T00:59:28Z` |
| `R-00279` | `2026-08-29T05:40:20Z` | `2026-08-30T00:59:31Z` |
| `R-00280` | `2026-08-29T05:40:32Z` | `2026-08-30T00:59:34Z` |
| `R-00281` | `2026-08-29T05:40:41Z` | `2026-08-30T00:59:36Z` |

#### `R-00141`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; C# mvp-host is a semantic/acceptance harness; Rust DS V1 is post-MVP.
Scope: D-005 Confirmation Record policy is separate from codec, adapter, audit, stream, and maintenance implementation.
Dependencies: D-005; R-00174; R-00176; R-00228; R-00231; R-00236; R-00245.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00141
```

#### `R-00172`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; GameRuntime remains the semantic owner of replication scheduling.
Scope: Runtime owns priority, starvation cap, frequency gate, jitter, truncation, and refill; Server supplies only token-bucket budget/permit primitives.
Dependencies: R-00279; D-009 for A1-beta.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00172
```

#### `R-00174`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; persistence semantics remain governed by the pending D-005 decision.
Scope: CommitIntent-before-apply ordering is explicit; this card does not choose a durability tier or replace the Confirmation Record.
Dependencies: R-00141; R-00176; D-005.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00174
```

#### `R-00176`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; SnapshotCut is an implementation consumer of the D-005 policy.
Scope: SnapshotCut is tied to one revision vector and a declared confirmation point; it does not silently establish a durability default.
Dependencies: R-00141; R-00174; D-005.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00176
```

#### `R-00214`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap for the near path; Rust pacing remains DS V1 post-MVP.
Scope: This card owns Rust pacing/permit primitives only; GameRuntime priority, starvation, and semantic send ordering stay out of Server.
Dependencies: R-00222; R-00229; R-00279.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00214
```

#### `R-00215`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; the adapter remains DS V1 post-MVP work.
Scope: Preserve local file/directory atomicity, but take ack/flush behavior from the explicitly selected D-005 tier rather than an implicit default.
Dependencies: D-005; R-00141.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00215
```

#### `R-00221`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; the Rust audit pipeline is post-MVP.
Scope: Audit durable ack references the selected Confirmation Record tier and remains independent from PersistenceCommitAck and its queue.
Dependencies: D-005; R-00141; R-00228.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00221
```

#### `R-00222`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; timer-driven permit scheduling is a Rust DS V1 primitive.
Scope: Schedule permits and deadlines only; do not duplicate GameRuntime semantic dispatch, priority, starvation, or token-bucket policy.
Dependencies: R-00214; R-00279.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00222
```

#### `R-00228`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; Durable Stream implementation remains DS V1 post-MVP.
Scope: Map each stream and CommitAck to an explicit D-005 tier and confirmation point; prohibit an undeclared default durability tier.
Dependencies: D-005; R-00141; R-00231.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00228
```

#### `R-00229`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; connection and queue primitives are separate from Runtime semantics.
Scope: Bound the registry and per-connection queue while leaving replication priority, starvation, and refill decisions to GameRuntime.
Dependencies: R-00237; R-00279; D-009.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00229
```

#### `R-00231`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; Rust recovery and migration adapters are post-MVP.
Scope: Declare recoverable material and loss bounds per D-005 tier; live migration is trigger-driven and not an implicit V1 behavior.
Dependencies: D-005; R-00141; R-00240; R-00241.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00231
```

#### `R-00235`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap near path; Rust DS V1 remains a separately named profile.
Scope: V1 is one process and one Release/WorldSlot; multi-slot capacity is a reservation, not current acceptance.
Dependencies: R-00237; W0.5 profile decision.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00235
```

#### `R-00236`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; the fault matrix must not imply a durability tier before D-005 is selected.
Scope: Classify terminal states and ack subsets separately for complete, async-flush, and snapshot-only profiles.
Dependencies: D-005; R-00141; R-00228; R-00231.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00236
```

#### `R-00237`

```text
---
TD boundary reconciliation r2
Profile: Rust DS V1 admission/session path is post-MVP under the recommended bootstrap profile.
Scope: Preserve the five-step/eight-step admission saga, but keep its production evidence separate from the C# semantic harness and state the reactivation condition.
Dependencies: W0.5 profile decision; R-00260; R-00277.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00237
```

#### `R-00240`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap near path; Rust DS V1 reconnect semantics remain post-MVP.
Scope: V1 means logout plus complete new-generation re-login; a reconnect window is trigger-only and never an implicit Resume Token.
Dependencies: R-00237; R-00241; D-009.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00240
```

#### `R-00241`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap near path; live migration is not a default V1 promise.
Scope: Separate graceful quiesce/drain from live migration; enable migration only after a product trigger and independent acceptance evidence.
Dependencies: R-00235; R-00240; R-00231.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00241
```

#### `R-00245`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; maintenance orchestration remains DS V1 post-MVP.
Scope: Keep the two durable acknowledgements independent and bind ReadyToExit/deadline behavior to the selected D-005 Confirmation Record tier.
Dependencies: D-005; R-00141; R-00228; R-00231.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00245
```

#### `R-00260`

```text
---
TD boundary reconciliation r2
Profile: Adopt MVP bootstrap: C# mvp-host is a semantic/acceptance harness, not Rust DS V1.
Scope: Keep the existing C# A1 path; list Rust DS V1 admission, WSS, WorldSlot, and process-boundary evidence as replacement conditions.
Dependencies: W0 validate/pin; R-00237; R-00277.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00260
```

#### `R-00277`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; this is C# reference simulation evidence, not Rust DS V1 completion.
Scope: Lock one-slot/owner-thread semantics and the bootstrap acceptance boundary; do not merge Rust production evidence into this card.
Dependencies: W0 validate/pin; W0.5 bootstrap decision; R-00260.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00277
```

#### `R-00278`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; the C# carrier is an acceptance harness path.
Scope: Validate the declared ws:// carrier boundary only; WSS, Rust transport production, and DS V1 process ownership remain out of scope.
Dependencies: R-00277; W0 pin.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00278
```

#### `R-00279`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; C# session evidence and GameRuntime semantic scheduling are distinct ownerships.
Scope: Keep admission/reconnect/session lifecycle in the harness while Runtime owns replication priority, starvation and refill; Server owns only budget/permit primitives.
Dependencies: R-00172; R-00214; R-00222; R-00229; D-009.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00279
```

#### `R-00280`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; the executable root and SmokeClient are harness assembly.
Scope: Prove the C# assembly boundary and smoke path without claiming Rust DS V1 App/process ownership.
Dependencies: R-00277; R-00278; R-00279.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00280
```

#### `R-00281`

```text
---
TD boundary reconciliation r2
Profile: MVP bootstrap; A1-alpha is a C# protocol/lifecycle claim only.
Scope: Limit acceptance to protocol/lifecycle closure; A1-beta block-state visibility remains gated by D-009 and ADR-028/ADR-049 public payload evidence.
Dependencies: R-00280; D-009; ADR-028; ADR-049.
Revision: workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/R-00281
```

### 12.2 新建清单（正式单号 / wave / 目标仓库 / 前置阻塞）

| 临时编号 / 正式单号与标题 | wave | 目标仓库 | 前置阻塞 |
|---|---|---|---|
| `NEW-01` / `R-00295` GameRuntime 复制发送调度 | W1-Runtime→W2/A0→W3/A1-beta | `LumioGameRuntime` | W0 pin；ECS/identity；D-009（beta） |
| `NEW-02` / `R-00296` Client Chunk 三态 | W1-support→W3 | `LumioClient` | W0 pin；`R-00151`/`R-00155`；不得新增 wire 字段 |
| `NEW-03` / `R-00297` D-005 三档 Confirmation Record | W0.5 决策→W4 consumers | `LumioGameEngineArchitecture` | Owner 确认 D-005；ADR/fixture 路径；不启动实现 |
| `NEW-04` / `R-00298` Client RTT/2 与离群剔除 | W1-support→W3/W4 | `LumioClient` | W0 pin；`R-00055`/`R-00281`；先检查公共 payload |

### 12.2.1 新卡的实际 PM 字段与读回

以下记录创建请求实际使用的字段和读回结果。三个已有 Room 的 UUID 在写入前复核；`roomId` 作为 Requirement 自身字段发送，没有创建或 PATCH Room，也没有做里程碑归属。

| 临时编号 / 正式单号 | 精确标题 | module | category | roomId（读回） | ownerId | priority | risk | status（读回） | releaseWindowId / startOn / targetOn | milestoneId | estimateDays |
|---|---|---|---|---|---|---|---|---|---|---|---:|
| `NEW-01` / `R-00295` | `GameRuntime 复制发送调度` | `replication` | `""`（沿用惯例） | `01a04225-7526-70be-8950-32f83dd061fd`（GameRuntime） | 省略；采用创建人默认，不指定成员 | 省略；API 默认 `P1` | 省略；API 默认 `medium` | `backlog` | 全部省略；未臆造日期或发布窗口 | 省略；不归属 `MS-00001` | 省略；未填 `0` |
| `NEW-02` / `R-00296` | `Client Chunk 三态` | `LumioClient` | `""`（沿用惯例） | `01a04225-86b1-7be9-870a-adcecb10807c`（Client） | 省略；采用创建人默认，不指定成员 | 省略；API 默认 `P1` | 省略；API 默认 `medium` | `backlog` | 全部省略；未臆造日期或发布窗口 | 省略；不归属 `MS-00001` | 省略；未填 `0` |
| `NEW-03` / `R-00297` | `D-005 三档 Confirmation Record` | `LumioGameEngineArchitecture` | `""`（沿用惯例） | `01a04225-4fc2-737e-afb3-8aaa8ba80754`（Architecture） | 省略；采用创建人默认，不指定成员 | 省略；API 默认 `P1` | 省略；API 默认 `medium` | `backlog` | 全部省略；未臆造日期或发布窗口 | 省略；不归属 `MS-00001` | 省略；未填 `0` |
| `NEW-04` / `R-00298` | `Client RTT/2 与离群剔除` | `LumioClient` | `""`（沿用惯例） | `01a04225-86b1-7be9-870a-adcecb10807c`（Client） | 省略；采用创建人默认，不指定成员 | 省略；API 默认 `P1` | 省略；API 默认 `medium` | `backlog` | 全部省略；未臆造日期或发布窗口 | 省略；不归属 `MS-00001` | 省略；未填 `0` |

四个 POST 实际使用的 `reason` 依次为 `workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-01`、`workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-02`、`workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-03` 和 `workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-04`。`module`、`category` 和 `roomId` 读回一致；owner、priority、risk、日期、releaseWindow、milestone 和 estimate 的省略按已确认账本执行。

### 12.2.2 四张新卡的完整正文（description 读回一致）

代码块内容是不含标题字段的完整四节正文；每张卡的 `title` 取 §12.2.1 的正式标题，且已写入并读回一致。正文保留依赖 displayKey，不调用关系 API。

#### `NEW-01` · `GameRuntime 复制发送调度`

```markdown
## 背景
`R-00172` 与 `R-00279` 分散描述了 Runtime 的复制映射、会话和队列边界，但没有冻结一份完整的 GameRuntime 语义发送调度契约。Server 的 token bucket/permit 只能提供预算原语；优先级、饥饿上限、频率门、抖动、截断回流和慢客户端阶梯必须由 Runtime 负责。

## 目标
在 `LumioGameRuntime` 内形成由 revision 和预算输入决定、可重复运行并可用 trace 验证的复制发送调度；让 Server 与 Runtime 的所有权可以由契约测试直接区分。

## 验收
- 相同 revision/预算输入产生相同的类别、优先级和等待时长排序，重复运行输出一致。
- 频率门、抖动和饥饿上限有正反测试；截断余量按原 revision 回流且不丢失。
- 慢客户端阶梯在有界队列和明确阈值下产生可复核 trace，不出现类别永久饥饿。
- 契约测试证明 Server 只提供 token bucket 余量/permit，优先级、饥饿和回流语义不在 Server 重复实现。

## 边界
只覆盖 Runtime 的语义复制调度，不实现或改写 Server pacing/token bucket，不新增 D-009 或 ADR-028/ADR-049 未授权的公共 wire 字段，不创建 WorkItem。前置锚点：`R-00172`、`R-00279`；W0 pin、ECS identity 和 D-009 公共契约仍是阻塞条件。`workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-01`
```

#### `NEW-02` · `Client Chunk 三态`

```markdown
## 背景
Voxel `R-00151`、`R-00153`、`R-00155` 和 ADR-024/035 已覆盖服务端 Demand/Fetch/Apply 及“没收到不等于空气”的原则，但 `LumioClient` 没有一张卡冻结客户端可见的未请求、在途、已到达三态。

## 目标
让客户端显式维护 `Unrequested`、`InFlight`、`Ready` 三态及其请求、到达、失败、重试、重复和乱序转换，并阻止未收到的数据被当成 Ready 或 Air。

## 验收
- 显式区分 `Unrequested`、`InFlight`、`Ready`，并覆盖请求、到达、失败和重试转换。
- `Unrequested`/`InFlight` 不得被渲染或查询为 `Ready`/`Air`，保留“没收到不等于空气”。
- 重复、乱序和过期响应不能回退状态或覆盖更新 revision。
- 正反测试复用 `R-00151`/`R-00155` 现有消息路径；若确需公共字段，必须以 BLOCKED 证据回到架构源。

## 边界
只覆盖 `LumioClient` 状态和既有消息路径，不新增或私改公开 wire 字段，不替代 VoxelEngine 的服务器侧状态机，不创建 WorkItem。前置锚点：`R-00151`、`R-00155`；W0 pin 和公共 payload 授权仍是阻塞条件。`workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-02`
```

#### `NEW-03` · `D-005 三档 Confirmation Record`

```markdown
## 背景
D-005、ADR-032/036 以及 `R-00141`、`R-00174`、`R-00176`、`R-00228`、`R-00231`、`R-00236`、`R-00245` 分别覆盖编码、事务、快照、流和故障处理，但尚未由一张架构需求记录完整耐久、MVP 异步 flush 和 snapshot-only fallback 三档 Confirmation Record 政策。

## 目标
冻结三档政策各自的确认点、可恢复材料、允许损失上界和重放边界，使实现卡只能消费已声明的档位，不能从 adapter 或 ack 名称推断默认耐久语义。

## 验收
- 完整耐久档明确确认点、可恢复材料和丢失上界。
- MVP 异步 flush 档明确确认点、允许的有界损失和恢复动作。
- snapshot-only fallback 档明确 snapshot 确认点、损失范围和重放边界。
- 矩阵逐项映射 `CommitIntent`、`SnapshotCut`、`Durable Stream` 与 ack，并拒绝未声明的隐含默认档。

## 边界
这是架构决策与验收口径卡，不直接实现 codec、存储 adapter、Durable Stream、migration 或 maintenance；不改 Baseline、Schema、ID 或既有状态。前置锚点：`R-00141`、`R-00228`；Owner 对 D-005 的确认和 ADR/fixture 路径是启动条件。`workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-03`
```

#### `NEW-04` · `Client RTT/2 与离群剔除`

```markdown
## 背景
`R-00055` 和 `R-00281` 只有时间/生命周期原则，尚未冻结客户端用发送与接收时间计算 RTT/2、识别离群样本及报告校正指标的独立语义。公共 payload 是否足够也尚未经过实现验证。

## 目标
在 `LumioClient` 形成确定性的 RTT/2 校正和离群剔除规则，能在正常延迟、异常样本、时钟偏移与恢复场景中输出可复核指标。

## 验收
- 使用单调发送/接收时间和明确采样窗口计算 RTT/2 校正，重复输入结果确定。
- 离群剔除规则、阈值和最小样本数有正反测试，单个异常样本不能造成时钟跳变。
- 优先捎带现有消息字段；发现公共载荷缺口时必须回到 ADR/Schema，不在 Client 私加字段。
- 覆盖正常延迟、离群样本、时钟偏移和恢复场景，并输出可复核的校正/剔除指标。

## 边界
只冻结客户端算法和可观测验收，不在本卡决定跨仓公共参数、wire 字段或 Server 时钟；公共载荷不足时停在架构 ADR/Schema。前置锚点：`R-00055`、`R-00281`；W0 pin 和 payload 复核是阻塞条件。`workflow-plan:ds-mvp-boundary-reconciliation-20260830/r2/NEW-04`
```

### 12.3 新卡原生验收项（已创建并读回，4 × 4 = 16）

下列 16 条原子判据已写入四张新卡并逐项读回。执行前重新读取项目 active acceptance type 和 `systemSemantic=not_started` 的 status；实际使用的类型与状态见 §12 顶部和执行摘要。

本账本的 16 条均隶属于 Requirement，使用 Requirement type/status；Quality type/status 只作为配置审计记录，不在本次数量中使用。四张卡各读回 4 条、`sortOrder` 为 1–4、`statusId` 均为 `astat_20e2c7f5c6d891ad0966208b55da0372`。

| 新卡 | 四条原子验收判据 |
|---|---|
| `NEW-01` | 1. 相同 revision/预算输入产生确定性的类别、优先级和等待时长排序，重复运行输出一致。<br>2. 频率门、抖动和饥饿上限有正反测试；截断余量按原 revision 回流且不丢失。<br>3. 慢客户端阶梯在有界队列和明确阈值下产生可复核 trace，不出现类别永久饥饿。<br>4. 契约测试证明 Server 只提供 token bucket 余量/permit，优先级、饥饿和回流语义不在 Server 重复实现。 |
| `NEW-02` | 1. 客户端显式区分 Unrequested、InFlight、Ready，并覆盖请求、到达、失败和重试转换。<br>2. Unrequested/InFlight 不得被渲染或查询为 Ready/Air，保留“没收到不等于空气”。<br>3. 重复、乱序和过期响应不能回退状态或覆盖更新 revision。<br>4. 正反测试复用 `R-00151`/`R-00155` 现有消息路径；若需公共字段则以 BLOCKED 证据停在架构源。 |
| `NEW-03` | 1. 完整耐久档明确确认点、可恢复材料和丢失上界。<br>2. MVP 异步 flush 档明确确认点、允许的有界损失和恢复动作。<br>3. snapshot-only fallback 档明确 snapshot 确认点、损失范围和重放边界。<br>4. 矩阵逐项映射 CommitIntent、SnapshotCut、Durable Stream 与 ack，并拒绝未声明的隐含默认档。 |
| `NEW-04` | 1. 使用单调发送/接收时间和明确采样窗口计算 RTT/2 校正，重复输入结果确定。<br>2. 离群剔除规则、阈值和最小样本数有正反测试，单个异常样本不能造成时钟跳变。<br>3. 优先捎带现有消息字段；发现公共载荷缺口时必须回到 ADR/Schema，不在 Client 私加字段。<br>4. 覆盖正常延迟、离群样本、时钟偏移和恢复场景，并输出可复核的校正/剔除指标。 |

### 12.4 评论清单

| 目标 | 评论主题 | 关联 wave | 评论 ID | 读回时间（UTC） |
|---|---|---|---|---|
| `R-00260` | 把 `mvp-host` 命名为 semantic/acceptance harness，列出 Rust DS V1 替换条件 | W0.5 | `01a0502f-1870-71da-a22d-6d2dbee7172e` | `2026-08-30T01:00:58.864651Z` |
| `R-00235` | 单进程单槽 V1 与多槽预留边界 | W0.5→W4 | `01a0502f-21c2-7217-896d-7923bd133755` | `2026-08-30T01:01:01.250956Z` |
| `R-00240` | V1 采用完整重登录，保留窗口只作触发式预留 | W4/DS V1 | `01a0502f-2671-7925-bbcf-9ace6c254acb` | `2026-08-30T01:01:02.449907Z` |
| `R-00241` | graceful quiesce/drain 与 live migration 分离 | W4/DS V1 | `01a0502f-2b2b-7159-b5b7-b231964925be` | `2026-08-30T01:01:03.659567Z` |
| `R-00279` | GameRuntime 语义发送调度与 Server token bucket/队列边界 | W1→W3 | `01a0502f-2fba-78cf-b508-acb71c567285` | `2026-08-30T01:01:04.826009Z` |
| `R-00141` | D-005 Confirmation Record 三档确认点与实现卡分离 | W0.5→W4 | `01a0502f-39c7-7e2f-8e30-b5ba8dff7b6f` | `2026-08-30T01:01:07.399446Z` |
| `R-00155` | 客户端 Chunk 三态与“缺失不等于空气”缺口 | W1-support→W3 | `01a0502f-404d-74e2-a378-d9ae99c3f3f5` | `2026-08-30T01:01:09.069724Z` |

7 条评论均以对应 marker 创建并读回，目标 Requirement、评论正文和 ID 一致；评论不会改变目标卡状态。

### 12.5 状态、关系与对象边界

- 23 张更新只改正文/验收说明，没有做 `backlog`、`in_review`、`in_progress`、`acceptance` 或 `done` 流转。
- 四张新 Requirement 创建时未传显式 `status`，读回均为 `backlog`；没有启动实现、创建 WorkItem、创建或 PATCH Room，也没有归属里程碑。
- `R-00237` 与 `R-00279` 各已有 5 个原生验收项，本轮只读回并复用；没有新增或删除它们的验收项。
- 新卡的八个依赖锚点写入正文/评论 displayKey；关系 API 写入保持 0。

## 13. 本轮实际动作与证据边界

本轮 Workflow 实际写入 **50** 个动作：23 张既有 Requirement 的 CAS 追加、4 张新 Requirement（正式单号 `R-00295`–`R-00298`）、16 个原生 acceptance item 和 7 条评论。所有写入均已读回；没有 `status transition`、附件、关系、WorkItem、Room 创建/PATCH、里程碑归属或 Baseline 变更。没有修改 `packages/`，没有 push 架构仓。

验证结果：`node tools/.workflow_ds_reconcile.mjs verify` 读回 298 张 Requirement、23 个严格追加、4 张新卡、16 个 acceptance item 和 7 条评论；独立 boundary 复核确认新增仅为 `R-00295`–`R-00298`，`R-00182` 的字段、验收项、评论和附件形状不变，Room 为 8→8，relations 为 0，`MS-00001` 的 70 个 Requirement ID 集合不变。当前 WorkItem 为 24；其中 `T-00010`–`T-00021` 在本轮第一次 PATCH 之前已创建，时间线归属缺口已在 §3.1.1 记录，本轮未创建 WorkItem。

## 14. 收口结论与剩余边界

本账本已按项目 `lumiogamesengine` 的 **MVP bootstrap profile** 执行并完成复核，不再有待确认的 Workflow 写入闸门。C# `mvp-host` 的定位已固定为 semantic/acceptance harness；Rust DS V1 为后续独立 profile。账本落库不等于实现完成，也不改变任何卡的工作状态。

剩余边界如下：

1. `D-009`、`D-011` 及 ADR-028/ADR-049 公共 payload 约束仍然有效；本轮没有新增或修改 wire 字段、Schema、ID、Baseline 或生成物。
2. W0 `validate`/下游 pin、Runtime/Client/Voxel/CoreEngine 的实现与实测门禁仍需按各卡验收；A1-beta 方块互见继续保持 BLOCKED，不能由建卡或评论代替证据。
3. Rust DS V1 的准入、WSS、WorldSlot、进程边界、持久化和 migration 仍是后置工作；本轮 C# harness 的账本证据不计入 Rust 生产完成度。
4. WorkItem 历史快照与当前查询数量存在时间线差异；在归属证据补齐前，不把 12 个额外 WorkItem 归因于本轮 Requirement 写入。

## 15. 开发派发提示词（已交付）

W0 候选复审与跨平台字节规范修复的独立记录见 [`2026-08-30-w0-byte-authority-review.md`](2026-08-30-w0-byte-authority-review.md)。

建单完成后的开发提示词已单独落盘：[`docs/plans/2026-08-30-ds-mvp-parallel-agent-prompt.md`](../plans/2026-08-30-ds-mvp-parallel-agent-prompt.md)。该提示词要求主 loop 先读取架构源、22 条裁决、Workflow 卡四路内容和七仓模块规范，再按互斥文件集扇出 Runtime、Server C# harness、Client、Voxel/Core/Native/Game 轨道；共享生成物、工程文件和集成测试保持单一 owner，批间依赖串行。

提示词交付前的只读复核（`2026-08-30T01:46:37Z` UTC）确认项目/profile、298 张 Requirement、24 个 WorkItem、8 个 Room、0 relations，以及 `R-00295`–`R-00298` 各 4 个 acceptance item；没有新增 Workflow 写入。提示词只指导后续实现，不把建单证据当作实现完成证据。

至此，本轮 **Workflow 对账、获授权建单和开发提示词交付可以安全关闭**；后续实现仍须按提示词的 wave、reviewer 和真实测试门禁单独验收。
