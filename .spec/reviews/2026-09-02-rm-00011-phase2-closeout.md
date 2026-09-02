---
name: 2026-09-02-rm-00011-phase2-closeout
description: RM-00011 Phase 2 closeout — live-11 on mvp-host, rust identical Game verifier, C# frozen, R-00344–R-00359 all done
metadata:
  type: doc
  status: 已交付
---

# RM-00011 Phase 2 closeout

Date: 2026-09-02  
Dispatcher: architecture-repo main session; isolated git worktrees only.  
Path: `.spec/reviews/` (spec-lint 8b maps 审查 → `.spec/reviews/`; `docs/reviews/` is forbidden except `docs/adr/` as git mode 120000).

Supersedes `.spec/reviews/2026-09-02-rm-00011-phase2-complete.md` (GameRoomHost `9e185d7`) and `.spec/reviews/2026-09-02-rm-00011-phase2-mvp-host-rust-oracle.md` (Game `2260c85` honest sibling-gap + forked rust oracle).

Live Workflow GET the same day. Repo SHAs from `git fetch origin` + `git rev-parse origin/main`. Honest not-ok was never encoded as SUCCESS.

## 1. 执行摘要

| 面 | origin/main | 说明 |
| --- | --- | --- |
| Architecture | `ee5f3fd39f2f4a6532562bb1551703ad7e3d6717` | `docs/adr/` restored as 55× git 120000 including ADR-045/052/053/054/055 (PR #61) |
| LumioGame | `a120f0d171e0c9de21c6b85c543370601d32225f` | verifier still PR #8 `1169a66`; README-only commit on top |
| LumioServer | `0e10b0488fe437112a10b5291f53fb0bb7ca549c` | PR #23 rust identical-suite; C# live-11 is PR #22 `e3f112f` |

| 卡 | 仓 | PR | Workflow | Independent review |
| --- | --- | --- | --- | --- |
| R-00354 101-entity live-11 | LumioGame + LumioServer mvp-host | Game [#8](https://github.com/LumioGames/LumioGame/pull/8), Server [#22](https://github.com/LumioGames/LumioServer/pull/22) | done/100 | 通过 P0=0 P1=0 |
| R-00359 rust identical suite | LumioServer `lumio-entity-chat-replay` | Server [#23](https://github.com/LumioGames/LumioServer/pull/23) | done/100 | 通过 P0=0 P1=0 (archive `7e288b8`) |

C# `mvp-host/` is frozen/reference **after** the identical suite passed on rust (Server ADR 0006, replacing 0005). Source kept. hello-wire-v1 not extended. LumioConfig not written. Shared architecture checkout was not used for construction.

## 2. Workflow GET R-00344–R-00359

GET 2026-09-02 (after R-00359 closeout). Every card is `done` / progress 100. None required close or escalate.

| key | uuid | status | progress | title |
| --- | --- | --- | --- | --- |
| R-00344 | `01a05b5a-75a6-77f1-a568-7b044e1a0053` | done | 100 | [Account] Account Server login-or-register and AccountEntity |
| R-00345 | `01a05b5a-75ad-7062-bfb9-b9df66ea7ca2` | done | 100 | [Original Requirement] Formal ECS Entity and Chat Vertical Slice |
| R-00346 | `01a05b5a-783a-797a-9726-10e26f9e5c76` | done | 100 | [Game Server] Room admission and Player/Bot Entity lifecycle |
| R-00347 | `01a05b5a-78c3-747f-8235-fe05e55bc4b1` | done | 100 | [Runtime] Common connection binding and NetEntityId Attribute Query |
| R-00348 | `01a05b5d-1812-7f95-889c-b191e395dc01` | done | 100 | [ECS] ChatComponent field declarations and SetMessage |
| R-00349 | `01a05b5d-19c6-7365-859a-810b1810638a` | done | 100 | [Client] ReplicaWorld entity mapping and chat presentation |
| R-00350 | `01a05b5d-19e0-79fc-af62-2007435d1348` | done | 100 | [Server] Five-minute reconnect and expiry lifecycle |
| R-00351 | `01a05b5d-1ad1-7d07-83e8-0aecc3f6c31b` | done | 100 | [Replication] ChatInput and ChatMessageEvent typed mapping |
| R-00352 | `01a05b5d-1d0f-73ec-be64-fae3392ca684` | done | 100 | [NativeCore] Fixed Tick/Frame Timer Manager |
| R-00353 | `01a05b5d-1d12-7762-a802-f08e6639f27b` | done | 100 | [Runtime] ECS Snapshot/Restore for ChatComponent state |
| R-00354 | `01a05b5d-1d29-77e0-8220-a50e39aafc06` | done | 100 | [Integration] 100 Bot plus Browser 101 Entity acceptance |
| R-00355 | `01a05b9c-5002-7c46-bba3-f6a02c88fa84` | done | 100 | [Architecture] Gameplay command envelope (C-1) |
| R-00356 | `01a05b9c-555f-7fa3-aa2d-4e951f0b0923` | done | 100 | [Architecture] Binding and Attribute Query (C-2) |
| R-00357 | `01a05b9c-595a-7a80-8021-329938e4ec68` | done | 100 | [Architecture] Account Port (C-3) |
| R-00358 | `01a05b9c-5de0-7c4a-b047-acfad0dc444e` | done | 100 | [Architecture] Native Timer ABI (C-4) |
| R-00359 | `01a05b9c-61b4-7a22-9120-7fc1fdbff0e5` | done | 100 | [Server] Slice-scoped minimal Rust host and acceptance replay |

R-00345 was closed as the planning-record parent after origin/main already held the DAG/room-review materials. R-00359 closeout comment `01a06027-32cb-79bf-9cf6-fea59db5776a`.

## 3. R-00354 — 11 scenarios actually pass on `lumio-mvp-host`

SUCCESS pack is sibling `lumio-mvp-host` (FullGraph MaxConnections/MaxSessions **128**, Server PR #18) + live Account Server + Playwright Chromium. `GameRoomHost` is a unit-test double only. Sibling-gap / S8 nent-gap packs FAIL the Game `1169a66` verifier.

Live-11 evidence (Game `integration/entity-chat/evidence/live-run-11`, gitignored; dispatcher recapture in scratch `R-00354-gate-1.log` / `gate-2.log` / `launch-1.log` / `launch-2.log`):

- Census 100 Bot + 1 Player from host `nent_*` (17-key audit + `/test-control/bindings`)
- S1–S11 `ok: true` both rounds
- S5 C-2 query outcomes live
- S6 `timerManagerInvoked: true`, `tickSource` `test-control/tick`
- S7 `snapshotSource: live-mvp-host`, `historyCountMax: 0`, `restoredWindow: 0`
- S8 rebound same host `nent_*` (not sessionId / login AccountId)
- S9 expiry new nent; S10 isolation; S11 two-round compare
- Playwright `injected: false`

Game PR #8 (`1169a66`) is the identical-suite oracle: `hostProcess.process` is `lumio-mvp-host` **or** `lumio-entity-chat-replay`. Honest not-ok is pack FAIL.

## 4. R-00359 — rust pack passes the identical Game verifier

Slice host: `EntityChatHost` + CoreCLR `ChatRoomWorld` + real Account Server. Not FullGraph. Does **not** impersonate `lumio-mvp-host`. Forked `modules/process/tests/verify_rust_evidence.mjs` is **not** the SUCCESS predicate.

Dispatcher recapture (scratch `R-00359-gate-1.log` / `R-00359-gate-2.log`):

```
node C:\Work\LumioGames\wt-game\r-00354-live11\integration\entity-chat\verify-evidence.mjs --dir %TEMP%\lumio-r-00359-entity-chat-evidence
node ... --dir %TEMP%\lumio-r-00359-entity-chat-evidence-b
```

Both exit 0, `ok: true`, `failures: []`.

Stamps on both rounds of both packs:

- `hostProcess.process`: `lumio-entity-chat-replay`
- S3 Playwright `{ ran: true, browser: chromium, channel: chrome, receivedFromNetwork: true, injected: false }` plus `browser-result.json` / `browser-console.ndjson`
- S6 `timerManagerInvoked: true`, `tickSource`/`cadence` `host-timer` (HostTimer callback, not a for-loop)
- S7 `snapshotSource: lumio-entity-chat-replay`, `windowBeforeSnapshot: 101`, `historyCountMax: 0`, `restoredWindow: 0`; restore on the same CoreCLR world, not `LocalGameplay`

`cargo test -p lumio-server-process --test entity_chat_acceptance --locked` 1 passed (~37s); two independent two-round packs. Independent review 通过 P0=0 P1=0.

## 5. C# MVP freeze

| ADR (LumioServer `.spec/decisions/`) | 状态 |
| --- | --- |
| 0004 csharp-mvp-host-frozen-reference | 被 0005 取代 |
| 0005 csharp-mvp-host-unfrozen-until-live-11 | 被 0006 取代 |
| 0006 csharp-mvp-host-frozen-after-rust-identical-suite | 生效 |

Freeze happened **after** live-11 on mvp-host **and** the rust pack passing Game `1169a66` `verify-evidence.mjs`. `mvp-host/` source is kept. Whole-directory delete waits for the 51-card Rust mainline.

## 6. Skeptic-panel gaps (closed)

| Gap | Close |
| --- | --- |
| R-00354 sibling-gap SUCCESS | Live-11: all 11 `ok: true` on `lumio-mvp-host`; GameRoomHost-green packs FAIL |
| R-00359 forked rust oracle | Game `1169a66` `verify-evidence.mjs --dir` is the SUCCESS oracle; two green packs |
| C# frozen too early | Unfrozen until live-11 + rust identical pass; now 0006 |
| `docs/adr/` missing | PR #61 restored 55× git 120000 including ADR-045/052/053 |
| R-00345 backlog | done/100 (planning record) |
| Missing gate/launch logs | Scratch `R-00354-gate-1/2.log`, `R-00354-launch-1/2.log`, `R-00359-gate-1/2.log` |

## 7. Known gaps (not SUCCESS-on-not-ok)

- 51-card Rust host-runtime / world-slot surface is out of this slice.
- FullGraph connection budget 128 is not the 101-path product host (`EntityChatHost` stays slice-scoped).
- Server CI MVP C# host policy CS0234 `Lumio.Engine.SDK` sibling missing on GitHub runners — pre-existing (PR #18/#21/#22/#23); README policy green. Merged `--admin`.
- `modules/process/tests/verify_rust_evidence.mjs` remains a repo-local helper, not the SUCCESS predicate.

No extra product work from this report.
