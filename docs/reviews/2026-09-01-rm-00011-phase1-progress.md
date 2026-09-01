# RM-00011 progress — Wave 0 frozen, Phase 1 pipelines started

Date: 2026-09-01  
Dispatcher worktrees only (shared architecture `main` checkout was not a commit target).  
Architecture `origin/main` at report time: `6aeb087` (includes Wave 0 via PR #52 + PR #54).

## Wave 0 (architecture contracts) — on origin/main

| Card | Workflow | SHA / PR | Surface |
| --- | --- | --- | --- |
| R-00355 C-1 | done, acceptance items passed | `935a8a9` PR #52 | ADR-049 Accepted; `engine/wire/gameplay-command-envelope-v1.json`; `eng/verify-wire.mjs` |
| R-00356 C-2 | done, items passed | `2b7e321` PR #54 | ADR-053; `entity-binding-and-query-v1.json` |
| R-00357 C-3 | done, items passed | `2b7e321` PR #54 | ADR-054; `account-port-v1.json` |
| R-00358 C-4 | done, items passed | `2b7e321` PR #54 + `de040dc` window fix | ADR-055; `native-timer-abi-v1.json` |

Owner landing: `docs/reviews/2026-09-01-owner-wire-landing.md`. hello-wire-v1 blob unchanged vs `d012c5c` (`ac19891…`). `docs/adr` ADR-045/052/053/054/055 are git mode `120000`. Deleted Schema/ID/Fixture toolchain was not restored.

Validators on the Wave 0 tree: spec-lint OK; generate-abi zero-diff `DEFINITION_SHA256=1dfc86da…`; verify-hello-wire 9/9 ×2; verify-wire 5 contracts green (envelope 14, binding 20, account 18, timer 22). `eng/dev-run.ps1` cannot start from `wt-arch/merge-wave0` (sibling host paths); honest unavailability captured.

## Phase 1 implementation (started as each C card landed)

| Card | Repo | Workflow | origin/main | Review |
| --- | --- | --- | --- | --- |
| R-00348 | LumioGame | done | `28d7285` ChatComponent + SetMessage | 通过 P0=0 P1=0; `dotnet exec` 14/14 ×2 |
| R-00344 | LumioServer | done | `93515ae` account-server process | 通过 P0=0 P1=0; `dotnet exec` 32/32 ×2 |
| R-00352 | LumioNativeCore | done | `13f53b5` PR #5 lumio-timer | 通过 P0=0 P1=0; `cargo test -p lumio-timer` 28/28 ×2 |
| R-00349 | LumioClient | done | `cf218a8` PR #14 ReplicaWorld + ordered chat | 通过 after P1 AOI gate; replica 31/31. CI Bot.Host missing sibling SDK (layout) |
| R-00346 | LumioServer | not started | waits live Preconditions including R-00277 | |
| R-00347 / R-00351 / R-00353 | LumioGameRuntime | not started | wait remaining ECS foundation (R-00150 Query still absent on Runtime main) | |
| R-00350 | LumioServer | not started | after R-00346 | |

## GameRuntime foundation

| Card | Workflow | origin/main | Notes |
| --- | --- | --- | --- |
| R-00139 T06 | done | `ef127bd` PR #10 | six-layer merge; P1 dangling-ref row-key fixed `9ac5739`; 25/25 `dotnet exec` |
| R-00140 T07 | done | `a5a536c` PR #11 | ConfigSnapshot + Barrier; 16/16 focused, 41/41 module |
| R-00141 | backlog | — | card still blocked on unpublished LumioBinV1 codec (must not invent encoder) |
| R-00150 | backlog | Query/View still missing | true gap |

`dotnet test --project` finds 0 tests on this machine (MTP apphost vs user-local SDK). Green evidence is `dotnet exec` of the test dll, recorded as such.

## Boundaries held

- RM-00010 / hello-wire-v1 not extended.
- LumioConfig: no tracked writes from this goal (`ee10aaa` unchanged).
- Shared architecture checkout not used for commits; merges via isolated worktrees + GitHub PRs.
- Secrets not in repo/prompts/logs.

## Next

1. R-00346 when R-00277 (or live Preconditions) is actually done.
2. R-00150 ECS Query then Runtime R-00347→351→353.
3. R-00350 reconnect/expiry.
4. Phase 2 R-00354 C# 101-entity E2E twice, then R-00359 Rust replay.
