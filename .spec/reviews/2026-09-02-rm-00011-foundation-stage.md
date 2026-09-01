---
name: 2026-09-02-rm-00011-foundation-stage
description: RM-00011 foundation-stage 对账：Wave 0 已冻结、Runtime 基础链已合入、下一张是 R-00347；编排派活前查
metadata:
  type: doc
  status: 已交付
---

# RM-00011 foundation stage — Wave 0 frozen, Phase 1 pipelines on main, R-00347 unblocked

Date: 2026-09-02  
Dispatcher: architecture-repo main session (continuation of interrupted Claude `e74952fd`; isolated worktrees only).  
Architecture `origin/main` at report time: `fb3dca451aef5b392876e284ba871b05e58186bb` (PR #56).  
This report supersedes `docs/reviews/2026-09-01-rm-00011-phase1-progress.md` (PR #56), which was stale on arrival (it still listed R-00346 / R-00350 / R-00150 as not started) and reintroduced a forbidden `docs/` root (spec-lint 8b). Canonical reviews live under `.spec/reviews/` after PR #55.

Live Workflow GET 2026-09-02 (not scratch JSON). Repo SHAs from `git fetch origin` + `git rev-parse origin/main` the same day.

## 1. 执行摘要

| 仓库 | origin/main | RM-00011 相对状态 |
|------|-------------|-------------------|
| LumioGameEngineArchitecture | `fb3dca4` | Wave 0 C-1…C-4 在 main；ADR-049/053/054/055 Accepted 于 `.spec/decisions/` |
| LumioGameRuntime | `d974cad` | 基础链 R-00149→150→152→172→178→189 已合入；C-2 绑定/查询实现仍缺（R-00347） |
| LumioServer | `e9b66b2` | R-00344→346→350 已合入；C-2/C-3 JSON 与架构仓 byte-identical |
| LumioGame | `28d7285` | R-00348 ChatComponent 已合入 |
| LumioClient | `cf218a8` | R-00349 ReplicaWorld + ordered chat 已合入 |
| LumioNativeCore | `a5a44ac` | R-00352 lumio-timer 已合入；C-4 JSON byte-identical |

**三个核心结论:**

1. **C-1 与 C-2 已在 architecture `origin/main`，下游已消费冻结 JSON。** Runtime 尚未实现 C-2 绑定五元组 / Attribute Query。R-00347 的契约前置与同仓前置（R-00149/150/152/172）均为 Workflow `done/100`。
2. **`docs/reviews/2026-09-01-rm-00011-phase1-progress.md` 不是执行真值。** 它在 PR #56 合入时已经过期，并且把 `docs/` 写回仓根，使当前 `origin/main` 的 `node .spec/tools/spec-lint.mjs` 在 8b 上失败。本报告落到 `.spec/reviews/` 并删除该 `docs/` 回流。
3. **下一张可派的 Phase 1 卡是 Runtime R-00347。** 不得开工 R-00351 / R-00353，直到 R-00347 独立审查通过、合入 Runtime main、Workflow 关闭。R-00354 / R-00359 更后。R-00141 仍不得发明 LumioBinV1。

## 2. Wave 0 契约（architecture origin/main）

Room review and Owner landing **exist on origin/main** (do not invent rulings). PR #55 moved them:

| 文件 | origin/main 路径 |
|------|------------------|
| Room review | `.spec/reviews/2026-09-01-rm-00011-room-review.md` |
| Room Review Rulings (2026-09-01) | `.spec/reviews/2026-09-01-ecs-formal-entity-chat-decision-log.md` 该标题下 |
| Owner C-1 wire landing | `.spec/reviews/2026-09-01-owner-wire-landing.md` |

| Card | Workflow | SHA / PR | Surface |
|------|----------|----------|---------|
| R-00355 C-1 | done/100 | `935a8a9` PR #52；随 PR #54 入 `2b7e321` | ADR-049 Accepted；`engine/wire/gameplay-command-envelope-v1.json` blob `f558b404…`；`eng/verify-wire.mjs` |
| R-00356 C-2 | done/100 | `2b7e321` PR #54 | ADR-053 Accepted；`engine/wire/entity-binding-and-query-v1.json` blob `cadae092…`（`lumio.entity-binding-query.v1`） |
| R-00357 C-3 | done/100 | `2b7e321` PR #54 | ADR-054 Accepted；`engine/wire/account-port-v1.json` blob `9760d922…` |
| R-00358 C-4 | done/100 | `2b7e321` PR #54 + `de040dc` window fix | ADR-055 Accepted；`engine/wire/native-timer-abi-v1.json` blob `31cbdbc4…` |

hello-wire-v1 blob **unchanged**: `ac19891d…`. Schema/ID/Fixture/Baseline/seven-repo mirrors were not restored. RM-00010 / LumioConfig were not touched.

**ADR numbering after PR #55:** union max is ADR-055 in `.spec/decisions/` (ADR-049…055 present). `docs/adr` 120000 兼容软链已随治理收敛删除，这是有意迁移，不是 Wave 0 丢失。本报告不再重建 `docs/adr`。

C-2 冻结语义（R-00347 必须按该 JSON 实现，不得另写一份真值）:

- 绑定五元组 `AccountId + RoomId + NetEntityId + EntityType + ConnectionGeneration`
- 操作 `selfLookup` / `resolveByConnection` / `resolveByNetEntityId`
- AttributeId 文法 `^[A-Z][A-Za-z0-9]*\.[a-z][A-Za-z0-9]*$`
- 五结局 `non_existent` / `stale_generation` / `invisible` / `unauthorized` / `tombstoned`（`resolvesToReplacement` 恒 false）
- 调用域 `server-authoritative`（Simulation Owner Thread 语义角色）与 `client-replica`（ReplicaWorld 本地读）

## 3. 下游消费（byte-identical JSON 或已合入的实现）

| 仓 | 消费面 | 证据 |
|----|--------|------|
| LumioServer | C-2 `mvp-host/contract/entity-binding-and-query-v1.json` | blob `cadae092` = architecture origin/main |
| LumioServer | C-3 `account-server/contract/account-port-v1.json` | blob `9760d922` = architecture origin/main |
| LumioNativeCore | C-4 `docs/architecture/wire/native-timer-abi-v1.json` | blob `31cbdbc4` = architecture origin/main |
| LumioGame | C-1 ChatComponent / SetMessage | `28d7285`；Workflow R-00348 done/100 |
| LumioClient | C-1 envelope fixtures + ReplicaWorld | `cf218a8` PR #14；`GameplayEnvelopeContractTests`；Workflow R-00349 done/100 |
| LumioGameRuntime | C-1 envelope validator 已有；**C-2 绑定/查询实现缺失** | `d974cad` 无 AttributeQuery / ConnectionBinding 生产类型；R-00347 真缺口 |

## 4. Phase 1 实现卡（已合入目标仓 main）

| Card | Repo | Workflow | origin/main | Review / 验证 |
|------|------|----------|-------------|---------------|
| R-00344 | LumioServer | done/100 | `93515ae` | 独立审查通过；account-server 32/32 |
| R-00346 | LumioServer | done/100 | `3ba5e7c` PR #13 + lockfiles `294bc0b` PR #15 | P1-1 跨房 live Admit 拒绝；Admission.Tests 后续随 R-00350 26/26 |
| R-00350 | LumioServer | done/100 | `e9b66b2` PR #17 | P1-1/P1-2 ReferenceEquals 守卫；300s Host ITimerService |
| R-00348 | LumioGame | done/100 | `28d7285` | ChatComponent；`dotnet exec` 14/14 ×2 |
| R-00349 | LumioClient | done/100 | `cf218a8` PR #14 | replica / session / JS 测试绿；CI Bot.Host 缺 sibling SDK 为既有布局问题 |
| R-00352 | LumioNativeCore | done/100 | `a5a44ac` PR #5 | P1-1 destroy_scope tombstone；`cargo test -p lumio-timer` 30/30 |

LumioServer FullGraph 仍不调用 Disconnect（R-00354 范围）。mvp-host CI `CS0234 Lumio.Engine.SDK` sibling 路径自 2026-08-31 起即存在，不是本批引入。

## 5. GameRuntime foundation

同仓前置（R-00347 卡面 Preconditions）全部 Workflow `done/100` 且代码在 `origin/main`:

| Card | Workflow | origin/main | 备注 |
|------|----------|-------------|------|
| R-00139 T06 | done | `ef127bd` PR #10 + `9ac5739` | 六层 merge；勿合并 leftover local validator |
| R-00140 T07 | done | `a5a536c` PR #11 | ConfigSnapshot + Barrier |
| R-00149 | done | 追溯 Approved（archive 18/18） | World / LocalEntityId / Generation |
| R-00150 | done | `d95a197` PR #12 `844e13c` | Query / View / ChangeSet |
| R-00152 | done | `65bc481` PR #13 `ff89b06` | owner-thread fail-stop |
| R-00154 | done | `54def78` PR #14 | Deferred token |
| R-00157 | done | `5b45ce4` PR #18 | Prepare / Apply 已接线 |
| R-00159 | done | `b5a830e` PR #15 + `4354a3a` | GAS；R-00347 可引用已存在的 Gas 工程 |
| R-00172 | done | `e7187f2` PR #16 `0e86abf` | Mapping Registry / Net↔Local |
| R-00178 | done | `6eb433c` PR #17 | `IRuntimeSession` 在 `Tick/IRuntimeSession.cs` |
| R-00189 | done | `d974cad` PR #19 | bounded ingress + native barrier |

**Workflow 与仓库漂移（不挡 R-00347）:**

| Card | Workflow 2026-09-02 | origin | 处置 |
|------|---------------------|--------|------|
| R-00164 T14 | `in_progress`/50（曾 done，后被独立 retro **退回** P0=1 P1=4 重开） | 文件在 `54def78` 起已存在 | **不关闭**。评论 `01a05db3-c2c0-7ab1-8d47-5cb3f426cf0c`。缺口修复不在 R-00347 范围。 |
| R-00141 | backlog | `modules/persistence/` README-only | **不得发明 LumioBinV1** |
| R-00162 / 167 / 174 / 176 / 184 | backlog | 部分代码已在 main（A-class 欠账） | 不作为 R-00347 开工闸门；R-00184 PhaseGraph 已在 main，R-00189 已消费 |

本机 `dotnet test --project` 对部分 SDK 布局返回 0 tests / exit 5。绿证据以 `dotnet exec` / `dotnet run --project` 为准。

## 6. 关键路径与下一 wave

1. **立刻：R-00347**（LumioGameRuntime）。GET-then-POST `backlog → in_review → approved → in_progress` 后在隔离 worktree TDD。Vendor architecture `origin/main` 的 C-2 JSON（blob `cadae092`），禁止第二份语义。
2. R-00347 审查通过并合入 Runtime main、Workflow `acceptance → done` 之后才派 **R-00351 / R-00353**。
3. 切片 E2E：**R-00354**（C# 101-entity，两次）→ **R-00359**（最小 Rust 宿主复跑同一验收）。
4. **R-00345** 仍 backlog（DAG / 房间级卡片，不是本阶段开工闸门）。
5. 不要扩展 hello-wire-v1；不要恢复已删 Schema/ID/Fixture；不要碰 RM-00010 / LumioConfig。

## 7. 风险与开放项

- R-00164 重开后的 P0（`TxnParticipantState` vs 已发布 `cross-world-txn` schema）仍开放；有 `wt-runtime/r-00164-fix` 痕迹，本阶段不并入 R-00347。
- R-00141 persistence codec 未发布。
- 架构仓 `eng/dev-run.ps1` 依赖 sibling 仓布局；本报告 worktree 在 `C:\Work\LumioGames\wt-arch\foundation-20260902`，`wt-arch` 下有实现仓 junction。
- PR #56 把过期 Phase 1 报告写入 `docs/`，与 PR #55 / spec-lint 8b 冲突。本提交删除该回流。`td-progress-audit` 技能正文仍写 `docs/reviews/`——路径已失效，技能修订不在本报告范围。

## 8. 本次已执行 / 下一步

**已执行（本报告提交时）**

- 全仓 `git fetch origin`；C-1…C-4 文件、ADR-049/053/054/055、room-review、Rulings、Owner landing 均在 architecture `origin/main`。
- 下游 C-2/C-3/C-4 JSON sha 对账通过。
- 剩余 RM-00011 卡 live GET：R-00347/351/353/354/359/345 = backlog；C 卡 R-00355…358 = done/100。
- 删除仓根 `docs/`（仅含过期 phase1-progress），报告改落到 `.spec/reviews/`。

**下一步（本报告合入 architecture main 之后，同一调度会话）**

- Workflow 流转 R-00347 到 `in_progress`。
- 隔离 Runtime worktree TDD 实现 C-2；两级独立审查；合入 Runtime main；GET-then-POST `acceptance → done`。
