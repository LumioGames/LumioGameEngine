# RM-00011 Phase 0 progress — Wave 0 contracts frozen

Date: 2026-09-01  
Dispatcher: architecture main session continuing interrupted Claude `e74952fd`  
Branch: `rm00011/merge-wave0` (isolated worktree `C:/Work/LumioGames/wt-arch/merge-wave0`)  
Base: `origin/main` `d012c5c`

## Outcome

Wave 0 C-1…C-4 are merged on `rm00011/merge-wave0` with ADR index + `docs/adr/` mode `120000` symlinks + `engine/wire/README.md` rows. Owner wire landing is `docs/reviews/2026-09-01-owner-wire-landing.md`. Hello-wire-v1 is not extended. Old Schema/ID/Fixture/Baseline/seven-repo-mirror toolchain is not restored.

| Card | Branch tip | ADR | Contract | Review |
| --- | --- | --- | --- | --- |
| R-00355 C-1 | `3ddfd39` | ADR-049 Accepted (existing number, finalized) | `engine/wire/gameplay-command-envelope-v1.json` + `eng/verify-wire.mjs` | Independent review Needs-fixes P1-1, then Approved after fix |
| R-00356 C-2 | `b075e51` | ADR-053 | `engine/wire/entity-binding-and-query-v1.json` | Returned P1×2, fix Approved |
| R-00357 C-3 | `e8a758f` | ADR-054 | `engine/wire/account-port-v1.json` | Returned P1×2, fix Approved |
| R-00358 C-4 | `de040dc` (merge-wave0) | ADR-055 | `engine/wire/native-timer-abi-v1.json` | Returned P1×2 then uniqueness fix; later P1 repeating window aligned to (committed, toTick] including 12/17/22 |

Dual-directory ADR max at first merge was ADR-052. New numbers 053–055 assigned at merge time. Missing `docs/adr/` links for ADR-045 and ADR-052 repaired in the C-1 registration commit.

## Architecture 收口门槛 (merge-wave0, twice)

- `node .spec/tools/spec-lint.mjs` → `spec-lint: OK`
- `node eng/generate-abi.mjs` → `ABI_VERSION=1` / `ENTRY_SYMBOL=lumio_engine_get_api_v1` / `DEFINITION_SHA256=1dfc86dad1ebbd8d6196d16946a9eb8542e951c83fa5e6163f696abee831fb8e` / generated files zero-diff
- `node --test eng/verify-hello-wire.mjs` → 9/9
- `node eng/verify-wire.mjs` → 5 contracts green (envelope 14 cases, binding 20, account 18, timer 22, hello-wire structural)
- `node --test eng/verify-wire.mjs` → 5/5 (drives shipped `validateContract` / `admitMessage`)
- `eng/dev-run.ps1`: not run from this worktree (sibling repos resolve as `wt-arch/../LumioServer`, which is not the real sibling layout). Recorded as merge-time remaining; dispatcher will run with explicit NativeCore/Voxel/Server/Client roots or from a layout where `..` is `LumioGames`.

## GameRuntime foundation (parallel line)

- R-00139 T06 implementer `6abcff6` on `feat/r-00139-config-merge` (not on Runtime origin/main). Status DONE_WITH_CONCERNS: no generated `ConfigTable.Validate` API after architecture `59866ec`; validator consumes generated column metadata. Independent review in flight. Tests 24/24 via `dotnet exec` (MTP apphost cannot see user-local SDK).
- R-00159 remains blocked on R-00140/152/154 completion evidence.
- A-class retro-review of already-committed ECS/command modules is next after R-00139 review.

## Workflow (live)

- R-00355 C-1: `done` on origin/main `935a8a9` (PR #52). Acceptance items `passed`.
- R-00356–358: still `in_progress` until merge-wave0 lands on origin/main.
- R-00348: `in_progress` (C-1 consumer, LumioGame worktree `feat/r-00348-chat-component`).

## Known gaps

1. `eng/dev-run.ps1` not runnable from `wt-arch/merge-wave0` (sibling host paths). Honest unavailability; validators are the bar for this launch.
2. C-2/C-3/C-4 not yet on architecture `origin/main` (branch `rm00011/merge-wave0` at `de040dc`).
3. Card Core Prompts still recite deleted Baseline/mirror chain; closeout comments must cite the Owner landing file.
4. R-00139 generated-config validator landing is an Owner architecture decision if review rejects the metadata-consumption approach.

## 沉淀

- ADR-049…055 + `engine/wire/*-v1.json` + `eng/verify-wire.mjs`
- `knowledge/standards/repository-architecture.md` development-state change order updated to name `engine/wire` + `verify-wire.mjs`
- Decision-log Change Log records the Owner wire landing
