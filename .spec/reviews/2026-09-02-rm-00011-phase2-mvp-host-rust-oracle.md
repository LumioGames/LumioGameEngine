---
name: 2026-09-02-rm-00011-phase2-mvp-host-rust-oracle
description: RM-00011 Phase 2 live SHAs after Game PR #7 mvp-host 101 and Server PR #21 rust oracle
metadata:
  type: doc
  status: 已交付
---

# RM-00011 Phase 2 — mvp-host 101 + rust oracle (live SHAs)

Date: 2026-09-02  
Dispatcher: architecture-repo main session; isolated git worktrees only.  
Supersedes `.spec/reviews/2026-09-02-rm-00011-phase2-complete.md` for **live origin/main SHAs**. That report closed Phase 2 on Game `9e185d7` (in-process `GameRoomHost`) and Server `e358a07`. This file records the suite that actually matches the Wave 2 cards after independent re-review.

Live Workflow GET the same day. Repo SHAs from `git fetch origin` + `git rev-parse origin/main`.

## 1. 执行摘要

| 卡 | 仓 | origin/main | PR | Workflow | Independent review |
| --- | --- | --- | --- | --- | --- |
| R-00354 101-entity C# harness | LumioGame | `2260c859e13d121f75c00f68dedc5e97fc8c80d6` | [#7](https://github.com/LumioGames/LumioGame/pull/7) | done | 通过 P0=0 P1=0 (HEAD `6ff6b3d` then rebase onto PR #6) |
| R-00359 Rust host replay | LumioServer | `5353589e681d68ecd976ceccb3b3ff38c1623328` | [#21](https://github.com/LumioGames/LumioServer/pull/21) | done | 通过 P0=0 P1=0 (`b68000d`) |

C# MVP host remains frozen/reference (Server ADR 0004). hello-wire-v1 not extended. LumioConfig not written. Shared architecture checkout was not used for construction.

## 2. R-00354 (C# mvp-host 101)

SUCCESS requires sibling `lumio-mvp-host` (FullGraph MaxConnections/MaxSessions **128**, Server PR #18) + origin/main Account Server. `GameRoomHost` is a unit-test double only. No `r-00344` fallback.

Dispatcher live run (`LUMIO_SERVER_ROOT=wt-server/r-00354-cap`): launcher exit 0, live admits **101/101 twice**, HTTP 101 + `lumio.mvp.v0` Handshake + FullSnapshot. Playwright chromium `injected: false`.

Node after rebase onto PR #6: `node --test integration/entity-chat/verify-evidence.mjs integration/entity-chat/bot-credential.mjs` **40/0**; `game-client.mjs` **7/0**; spec-lint OK; spec-lint tests **13/0**.

Honest not-ok on C# host (not GameRoomHost green):

- S6 Timer tick-batched (`timerManagerInvoked: false`).
- S5/S7/S9/S10/S11 sibling-gap (`ReferenceWorldSimulation` has no ChatComponent/C-2).
- S8 Entity A: FullSnapshot / 17-key audit / test-control do not project `ConnectionBinding.NetEntityId` (`nent_*`) → `ok: false` + projection-gap `blockedReason`. sessionId alias and login AccountId are not rebind.

## 3. R-00359 (slice-scoped Rust replay)

Slice host: `EntityChatHost` + CoreCLR `ChatRoomWorld` + real Account Server. Not FullGraph. `hostProcess.process = lumio-entity-chat-replay` (does not impersonate `lumio-mvp-host`).

Dispatcher: `node --test modules/process/tests/verify_rust_evidence.mjs` **9/0**; `cargo test -p lumio-server-process --test entity_chat_acceptance --locked` **1/0** (~16s); spec-lint OK.

Rust SUCCESS: 101 per-entity `nent_*` census; S8 same host NetEntityId; S5–S11 executed; C-1 InputCommand on admitted chat; two OS-process rounds compared.

Known gap: Game `verify-evidence.mjs` still requires `lumio-mvp-host` and treats rust S5/7/9/10/11 `ok:true` as suite-double. That is the C# MVP SUCCESS predicate. Rust closeout is the rust oracle. Follow-up would be a Game host-agnostic oracle, not a rust process-name lie.

CI: Server README policy green. MVP C# host policy CS0234 `Lumio.Engine.SDK` sibling missing on GitHub runners — pre-existing (same as PR #18), not this diff.

## 4. 与先前 Phase 2 报告的差异

| 先前 (`phase2-complete.md`) | 现况 |
| --- | --- |
| Game `9e185d7` / PR #5 `GameRoomHost` SUCCESS | Game `2260c85` / PR #7 mvp-host 101 Handshake |
| Server `e358a07` / PR #19 vs Game `9e185d7` verifier | Server `5353589` / PR #21 rust oracle vs Game `2260c85` suite |
| FullGraph cap 64, 101 not on mvp-host | FullGraph 128; 101 live on mvp-host; rust host stays slice-scoped |

## 5. 下一动作

RM-00011 Wave 2 cards are `done` on the SHAs in §1. Game oracle remaining lock (`lumio-mvp-host` process name) is a known gap, not a reopen of R-00359. Do not start extra product/capacity work from this report.
