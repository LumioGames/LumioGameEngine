---
name: 2026-09-02-rm-00011-phase1-complete
description: RM-00011 Phase 1 五仓管线已合入目标仓 main 并 Workflow 关闭；下一张是 R-00354；编排 Phase 2 前查
metadata:
  type: doc
  status: 已交付
---

# RM-00011 Phase 1 complete — five pipelines on main

Date: 2026-09-02  
Dispatcher: architecture-repo main session; isolated worktrees only.  
Supersedes the foundation-stage snapshot `.spec/reviews/2026-09-02-rm-00011-foundation-stage.md` for **implementation-card** status. Wave 0 contract facts in that file still hold.

Live Workflow GET the same day. Repo SHAs from `git fetch origin` + `git rev-parse origin/main`.

## 1. 执行摘要

Phase 1 五条实现管线全部 Workflow `done/100` 且代码在各仓 `origin/main`。下一张是 **R-00354**（101-entity C# E2E）。不要重开已完成的 R-00344 / 348 / 349 / 352。

| 管线 | 卡 | origin/main | Workflow |
|------|----|-------------|----------|
| LumioServer | R-00344 → 346 → 350 | `e9b66b2` | done/100 |
| LumioGameRuntime | R-00347 → 351 → 353 | `9edbe11` | done/100 |
| LumioGame | R-00348 | `28d7285` | done/100 |
| LumioClient | R-00349 | `cf218a8` | done/100 |
| LumioNativeCore | R-00352 | `a5a44ac` | done/100 |

Architecture `origin/main` at this report: `ccd656e` (foundation-stage report PR #57). C-1…C-4 unchanged.

## 2. Runtime 本阶段新合入

| Card | SHA / PR | 验证 | Review |
|------|----------|------|--------|
| R-00347 binding/query | `4d1d84c` PR #21 | `EntityBindingQueryTests` 34/34 `dotnet run` | P1×3 then Approved |
| R-00351 chat mapping | `28491bf` PR #23 | `ChatMappingTests` 15/15; NU1004 lockfile `55a8f1c` | P1×2 then Approved |
| R-00353 persist snapshot | `9edbe11` PR #24 | `ChatComponentSnapshotTests` 9/9 `--filter-class` | Approved P0=0 P1=0 |

C-2 vendor blob `cadae092…`; C-1 vendor blob `f558b404…`. hello-wire-v1 not extended. `modules/persistence/src` not created. LumioBinV1 not invented.

## 3. 未开工 / 仍开放

- **R-00354** backlog — Phase 2 C# 101-entity E2E（100 Bot + 1 Browser）。FullGraph 仍不调用 Disconnect（R-00350 已知）。
- **R-00359** backlog — 最小 Rust 宿主复跑同一考卷。
- **R-00345** backlog — 不是开工闸门。
- **R-00164** in_progress after retro 退回 — 不挡 R-00354 开工，除非卡面 Preconditions 写明。
- **R-00141** persistence codec — 不得发明 LumioBinV1。

## 4. 下一 wave

1. R-00354 on LumioServer (or the verification-plane repo named on the live card). GET-then-POST after reading the live body. Isolated worktree. Independent review.
2. On pass: R-00359 Rust host replay; then freeze/retire C# MVP host per rulings.

Sibling Phase 1 cards R-00344 / 348 / 349 / 352 were already `done/100` when this dispatcher checked live GET; they were **not** restarted.
