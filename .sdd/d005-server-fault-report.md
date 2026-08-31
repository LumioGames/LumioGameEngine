# D-005 Consumer Track: R-00236 Durability Fault Matrix

## Result

**BLOCKED (preflight).** The assigned isolated LumioServer worktree does not
contain a `lumio-persistence-host` crate, its `Recovery` implementation, or
either of the two files authorized by R-00236. R-00231 is not at its required
completion gate, so this track was stopped before editing and no production
code, fake contract, test, Workflow object, or architecture source was
changed.

## Blocking Evidence

### R-00231 Workflow read-back

The R-00231 UUID from the brief is
`01a043cd-8264-7b77-b4b1-19a7151ef2cd`.

Read-only API calls were made against the configured
`lumiogamesengine.workflow.games/api/v1` host after the required identity and
project checks (`GET /me` and `GET /projects/current`, both HTTP 200). The
project response identified `LumioGamesEngine` / `lumiogamesengine`; no write
endpoint was called.

| Read-back | HTTP result | Evidence |
| --- | ---: | --- |
| `GET /requirements/01a043cd-8264-7b77-b4b1-19a7151ef2cd` | 200 | `displayKey=R-00231`, `status=backlog`, `progress=0`, `module=persistence-host`, `updatedAt=2026-08-30T00:59:02Z` |
| `GET /requirements/01a043cd-8264-7b77-b4b1-19a7151ef2cd/acceptance-items` | 200 | `count=5`; all five items have `systemSemantic=not_started` (the five acceptance texts are listed below) |
| `GET /comments?targetType=requirement&targetId=01a043cd-8264-7b77-b4b1-19a7151ef2cd` | 200 | `items=[]` |
| `GET /attachments?targetType=requirement&targetId=01a043cd-8264-7b77-b4b1-19a7151ef2cd` | 200 | `items=[]` |
| `GET /documents?requirementId=01a043cd-8264-7b77-b4b1-19a7151ef2cd` | 200 | `items=[]`, `nextCursor=""` |
| `GET /work-items?requirementId=01a043cd-8264-7b77-b4b1-19a7151ef2cd` | 200 | `items=[]`, `nextCursor=""` |
| `GET /requirements/01a043cd-8264-7b77-b4b1-19a7151ef2cd/activity` | 200 | 8 events; kinds are only `requirement.create`, `requirement.acceptance_item.create`, `requirement.milestone.update`, and `requirement.update`; no implementation or delivery evidence |

The five acceptance items returned by the API, each still `not_started`, are:

1. `01a043cd-94bb-7b89-82e6-de974dbca5de`: bad tails must not be silently
   swallowed; emit explicit truncate/indeterminate/fatal plan evidence.
2. `01a043cd-9b2b-70a1-a2d1-440a012f370b`: checkpoint consumes only
   `TimerFired` or explicit Runtime tick evidence, never wall clock or sleep.
3. `01a043cd-9d1f-7d01-a9a5-21f7983c2883`: migration executes only the
   upstream manifest's defined nodes/order and does not invent a DAG here.
4. `01a043cd-a2f4-76cc-af6d-81a8fe9f639d`: listener/admission stays closed
   until successful `RecoveryCompleted` (process integration gate).
5. `01a043cd-91ba-7657-bbf9-47f76445581a`: recovery rejects bad
   hash/length/activation state with deterministic selection rules.

This is insufficient evidence for the R-00236 hard dependency: the
requirement is still backlog, every acceptance item is not started, and there
is no linked delivery artifact, comment, document, or work item.

### Persistence-host shell read-back

The assigned worktree is
`C:/Users/g923/orca/workspaces/LumioServer/d005-server-fault`, branch
`Go1c/d005-server-fault`, at source commit
`37d4af470d25c28b4f0dd23cdf969fed03720ef0`.

The exact read-only preflight produced:

```text
PRESENT modules/persistence-host
PRESENT modules/persistence-host/README.md
ABSENT modules/persistence-host/Cargo.toml
ABSENT modules/persistence-host/src
ABSENT modules/persistence-host/src/lib.rs
ABSENT modules/persistence-host/src/recovery.rs
ABSENT modules/persistence-host/src/checkpoint.rs
ABSENT modules/persistence-host/src/migration.rs
ABSENT modules/persistence-host/tests
ABSENT modules/persistence-host/tests/durability_fault_matrix_test.rs
ABSENT modules/persistence-host/tests/recovery_property_test.rs

git ls-tree -r --name-only HEAD -- modules/persistence-host
modules/persistence-host/README.md

cargo metadata exit=0; packages=lumio-host-testkit,lumio-architecture-contracts,lumio-server-process,lumio-server-xtask

cargo test -p lumio-persistence-host --no-run
error: package ID specification `lumio-persistence-host` did not match any packages
cargo test lookup exit=101

rg --glob '*.rs' 'RecoveryPlan|PersistenceHost|recover\\s*(' modules/persistence-host
no Rust source files/symbol hits
rg exit=1

git diff --exit-code -- modules/persistence-host
git diff exit=0
```

The brief's target path `C:/Work/LumioGames/LumioServer` was also checked
read-only. It exists at commit
`fe1f3eebbf7ff42be02d4e0f63c9252d48b48bf1`, and its
`modules/persistence-host` tree likewise contains only `README.md`; no crate
manifest, `src`, `Recovery`, or tests are present there either.

## Required Card Context

- Task: R-00236 / `implement-persistence-durability-fault-matrix` (D-005,
  persistence-host, Wave 7).
- Authorized target files, none touched:
  `modules/persistence-host/tests/durability_fault_matrix_test.rs` and
  `modules/persistence-host/tests/recovery_property_test.rs`.
- The brief-recorded architecture inputs are baseline
  `LGE-V1.4-2026-08-27`, architecture commits
  `c14df420ac05b0d23f1fb674977b9a4c957edac5` and
  `f71cac137733b7f1609ae8235676d44c9f324858`, and source hashes
  `d69c69374ef960b1968f0e8b2fdd4195d1abd52ed5ab34fd00b406fa85f141f1` and
  `82ed79a72ced56913c79ffa0bfb6d3763221ff2312c13c4a4d34f56e89b56f7c`.
  The current architecture document read-back remains
  `docs/architecture/LumioGameEngine_Architecture_v1.4.md` SHA-256
  `f1d36acf33a1f5e8326a9e58d609fcf7d9fa85177f9b5b60bb3f4742c1afebd0`.

## Delivery Format

Task ID: `R-00236` / `implement-persistence-durability-fault-matrix`

Source commit: `37d4af470d25c28b4f0dd23cdf969fed03720ef0`

Architecture baseline/commit: `LGE-V1.4-2026-08-27` /
`c14df420ac05b0d23f1fb674977b9a4c957edac5` +
`f71cac137733b7f1609ae8235676d44c9f324858`

Files created/modified: only this prescribed report artifact was created;
target worktree files: **none**. No Workflow or architecture source change
was made.

Commands executed (read-only unless noted):

- `GET /me` and `GET /projects/current` via `curl.exe`: HTTP 200 each.
- R-00231 detail, acceptance-items, comments, attachments, documents,
  linked work-items, and activity GETs: HTTP 200 (summarized above).
- `git rev-parse --show-toplevel`, `git rev-parse HEAD`, `git status
  --short --branch`, `git ls-tree`, and `git log --all -- modules/persistence-host`:
  exit 0; only the README is tracked and history contains documentation-only
  entries.
- `cargo metadata --no-deps --format-version 1`: exit 0; no persistence
  package in the four-package workspace.
- `cargo test -p lumio-persistence-host --no-run`: exit 101 because the
  package does not exist (expected blocking negative evidence).
- `rg` implementation-symbol scan: exit 1; no Rust source/symbol hits.
- `git diff --exit-code -- modules/persistence-host`: exit 0; no edits.

Generated artifacts and digests: none. The report is not a generated
contract artifact.

Tests passed: none; test execution is prohibited by the missing dependency
and crate shell.

Negative tests passed: none; adding test-only fakes or placeholders is
explicitly forbidden by the brief.

Dependency/package-content report: **BLOCKED** on R-00231 completion and the
absent `lumio-persistence-host` package/Recovery implementation. The README is
only a descriptive module shell and is not executable evidence.

Architecture Gates still blocked: D-005 persistence durability policy and
the R-00231 implementation prerequisite remain unresolved. No implicit
complete, async-flush, or snapshot-only tier was selected.

Deviations from task card: stopped at the mandated preflight; no tests or
implementation were added.

New dependency/ADR impact: none.

Knowledge synchronization or explicit exemption: none; this is a blocked
precondition report, not a new pattern.

## Escalation

Blocking evidence: R-00231 is `backlog` with `progress=0`, all five
acceptance items are `not_started`, and no delivery evidence is attached;
the target workspace has no persistence-host crate or Recovery source.

Impact range: R-00236's two authorized test files and the persistence-host
module; no other files were changed.

Read-only checks completed: Workflow identity/project and R-00231 evidence
GETs; target and isolated-worktree status/tree/package/symbol checks; expected
missing-package Cargo probe; clean diff check.

Still possible without conflict: none for this track. Once the persistence
owner lands a verifiably complete R-00231 implementation and registers the
crate in the workspace, rerun this preflight and then implement only the two
R-00236 test files.

Required owner decision: persistence-host owner must complete R-00231 and
provide its implementation/acceptance evidence; the architecture/workflow
owner must decide any prerequisite sequencing. No blueprints or architecture
contracts should be changed by this track.

Blueprint revision needed: no determination requested from this blocked
consumer; do not increment it here.
