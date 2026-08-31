# Architecture Repository Closeout

Date: 2026-08-31
Scope: `LumioGameEngineArchitecture` only
Authority: local Git objects and the repository review records. No push,
remote-branch deletion, Workflow write, or change to another repository was
performed.

## Decision

The closeout integrates only the architecture baseline synchronization and the
independently approved GAS V1.4 chain. Runtime and downstream implementation
candidates remain unaccepted. The resulting `main` is a usable architecture
source snapshot, but it is **not** an end-to-end MVP release: the blocked
consumer contracts and the items listed in [Open Holds](#open-holds) still
require a separately authorized wave.

## Initial Inventory

The first inventory was taken before cleanup from the repository common
directory `C:\Work\LumioGames\LumioGameEngineArchitecture`.

| Item | Observed value |
|---|---|
| Working branch | `codex/ms-00001-w0-byte-authority` |
| Working HEAD at inventory | `42bad7d` |
| Local `main` | `6358f96` |
| `origin/main` | `a7c1221` |
| Registered worktrees | 65 |
| Local branches | 55 |
| Main worktree | `C:\Users\g923\orca\workspaces\LumioGameEngineArchitecture\profile-decision-record` |
| Main worktree state | clean at `6358f96` |
| Temporary integration worktree | `C:\Temp\lumio-architecture-main-integration` |
| Temporary integration branch | `closeout/main-integration-20260831` |

The root working tree contained 116 status entries: 48 tracked mode-only
changes (`120000 -> 100644`) for ADR symlink materialization and 68 untracked
audit/Workflow files. The mode-only entries have zero content diff and are
`core.symlinks=false` Windows checkout noise; they were not staged. Across all
registered worktrees, 32 were dirty: 25 contained only equivalent symlink
materialization noise and 7 contained substantive or untracked evidence.

## Preserved Uncommitted Content

The root audit material was real, reproducible evidence and was committed
before branch/worktree cleanup:

| Commit | Content |
|---|---|
| `2e4df0f` | 54 `.sdd` files and 14 `.workflow-drafts` files (68 paths, 55,392 lines) |
| `8b885c3` | 76 files copied from dirty review worktrees, including generated review output and exact overlay files |

The same commits are present on the integration branch as `c59b95c` and
`0f4d3e1`, respectively. The retained worktree evidence is under
`.sdd/retained-worktrees/`; `README.md` and `SHA256SUMS.txt` describe its
origins and integrity check. The original evidence set was 55 `.sdd` files
(3,414,507 bytes) and 14 Workflow draft files (50,811 bytes). The retained
review overlay contains 76 copied source files (251,552 bytes). The tracked
retention directory adds its `README.md` as file 77 (253,223 bytes total before
`SHA256SUMS.txt`); the checksum file has 77 entries.

The following substantive dirty worktrees were preserved or accounted for
before removal:

| Worktree | Finding and preservation |
|---|---|
| `gas-a2-review-snapshot` (`a4c1c57`, detached) | `.review-generated` and `review-report.tmp.md` copied under `.sdd/retained-worktrees/gas-a2-review-snapshot.*` |
| `spec-lint-containment-fix` (`0e5ea2f`) | review report copied under `.sdd/retained-worktrees/spec-lint-containment-fix/report.md` |
| `w0-architecture-review` (`753920e`) | review report copied under `.sdd/retained-worktrees/w0-architecture-review/w0-task-review.md` |
| `w0-byte-authority-fix` (`e1705e9`) | uncommitted review diff copied under `.sdd/retained-worktrees/w0-byte-authority-fix/review-uncommitted.diff` |
| `w0-byte-authority-review` (`b7db298`) | unaccepted `.gitattributes` overlay and review report copied under `.sdd/retained-worktrees/w0-byte-authority-review/` |
| `rm-00009-r00316` (`f317b92`, detached) | 42 tracked changes and nine fixtures compared against `512da15`; all published blobs matched, so no new implementation was committed. Archive ref `8668dbc` remains documented until cleanup. |

No credential-shaped values were found by the closeout scan. The retained
`orca-inbox.json` is an audit transcript, not an instruction source.

## Integrated Commits

The temporary integration branch was built from local `main`, accepted GAS
V1.4 commits, and `origin/main`, resolving the GAS document conflict in favor
of the reviewed version. The merge commit is `a9e3d65` (parents `5812653` and
`a7c1221`). The accepted implementation/history chain is:

```text
3f8f0ce  fix(spec-lint): harden symlink containment checks
06b313b  docs(specs): GAS framework architecture decision stream
96c3fde  docs(plans): add GAS v1.4 delivery index
648f04a  docs(plans): close GAS A0 review findings
b73b295  fix(plans): restore immutable GAS S15 wording
88a3f14  feat(gas): publish A1 lifecycle evaluation contracts
a4c1c57  feat(gas): publish A2 component projection contracts
e57583d  fix(gas): enforce tag query modes and replay bounds
f401e57  fix(gas): close V1.4 review contract gaps
16500b0  fix(gas): close final V1.4 re-review gaps
ce0b59a  fix(gas): harden malformed records and Decimal canonical output
5812653  fix(gas): enforce adjusted Decimal exponent bounds
a9e3d65  merge: synchronize published architecture baseline
c59b95c  docs(audit): retain closeout evidence and workflow drafts
0f4d3e1  docs(audit): preserve uncommitted review evidence
```

The report is updated on `main` after verification; the final response records
the resulting commit ID because a commit cannot contain its own hash.

## Deliberately Not Integrated

The following are recorded here so branch deletion cannot be mistaken for
acceptance.

| Candidate / area | Evidence | Disposition |
|---|---|---|
| Runtime Command candidate `79528044f758d188844270bc7e55decce2a7b0cc` | `.sdd/runtime-command-review-report.md`, `.sdd/runtime-command-review-final-report.md` | `UNACCEPTED`; P1 findings include default in-memory journal, committed-marker/revision ordering, hand-authored Voxel contract, and generated-validation mismatch |
| D-005 / `R-00141` | `.sdd/d005-canonical-report.md` | `BLOCKED_UPSTREAM`; executable `LumioBinV1` is unpublished |
| GAS validator-hardening `b0058d7 -> 4d1e86d -> 2a2dddc` | `C:\Temp\gas-v14-validator-hardening-fix-review-report.md` and retained review records | `RETURN` (0 P0, 3 P1, 3 P2); the later review was recalled and has no final verdict |
| W0 republish `753920e` | `.sdd/final-w0-main-range-review.md`, retained W0 report | rejected byte/provenance candidate; not merged |
| W0 branches `e1705e9`, `681ef0e`, `c7e84ad` | W0 byte-authority reports and branch diffs | corrective/review or conditional slices only; no independent final acceptance for this closeout |
| Config R-00315 / R-00316 | `.sdd/rm-00009/progress.md`, `.workflow-drafts/*` | not merged: ADR-050/ADR-051 public-number conflict and review/acceptance evidence is not an architecture merge authorization |
| Config R-00317 | branch `2a96a9e` and repair archive `bcac43a` | not merged; review chain remains outside the accepted baseline |
| Runtime ECS | `C:\Work\LumioGames\_codex-verification\runtime-ecs-final-review-report.md` | `RETURN`; missing generated metadata/public contract boundary (9 P1) |
| Runtime Simulation | `runtime-simulation-review-v4-report.md` | `RETURN`; 1 P0, 9 P1, 2 P2 and unavailable shared authority surfaces |
| Runtime Replication V4 | patch SHA-256 `98C4487755354DCE5EF230591D5AE996061BA43669D85BECDB097B4747C0A380` | `RETURN` (0 P0, 4 P1, 3 P2) |
| Voxel V2 | patch SHA-256 `30424CFFFE9F5A84B8B48C19A2E61BFEC11BCBAB56ACA1E28F359A8CBDB39F2B` | `RETURN` (0 P0, 7 P1, 2 P2); callable surface `BLOCKED_UPSTREAM` |
| Client Chunk V3 | patch SHA-256 `3AA2E021B4D59994A9E39FD38BDB4DAAA7D25C2F0D1CDB3166E39E40973304D3` | `RETURN` (0 P0, 1 P1, 0 P2) |
| Client Session V3 | patch SHA-256 `1D01174EEC4E62A4E3D98917E6103AC3AE2647FF34D8C98D185749786ABD13DC` | `RETURN` (0 P0, 5 P1, 0 P2); DeltaAck blocked upstream |
| Client Connection V3 | patch SHA-256 `3E1698C25105E6F4E146A4435BE169A3D359D8763EB1A298DD2C71C266D68715` | fixer evidence only; fresh review recalled |
| Server A1 V2 | patch SHA-256 `B81DE9AA21ABD5BC65BE6D15417FCEB5114FDD431B4A819FED6ADB8AB5195AA3` | fixer evidence only; reconnect verification not rerun; Step 16 blocked upstream |
| NativeCore / CoreEngine | reports in the audit inbox and prior progress records | conditional/implementation evidence, not complete eight-repository acceptance |

Review snapshots, duplicate review branches, and known-error implementations
were not cherry-picked merely because their local tests were green.

## Verification Evidence

The isolated integration tree had the following pre-report evidence (all
commands were run against that tree):

| Command | Observed output |
|---|---|
| `node .spec/tools/spec-lint.mjs` | `spec-lint: OK` |
| `node --test .spec/tools/spec-lint.test.mjs` | 17 passed, 0 failed |
| `python -m py_compile tools/lumio_contract.py tools/lumio_generate.py tools/lumio_kat.py` | exit 0 |
| `python -m unittest tools.test_lumio_contract_gas -v` | 10 tests, OK |
| `python tools/lumio_contract.py validate` | `Validated 264 fixture(s), 0 failure(s).` |
| generator repeat and checked-in comparison | 12 artifacts; stable `outputHash`; 70/70 files, 0 mismatch; compiler/input/Root-ABI hashes were identical across runs |
| Rust contract-runtime checks | `cargo check`, `cargo test`, and `cargo clippy -- -D warnings` exit 0; 3 contract-runtime tests passed |
| `python tools/lumio_kat.py` | C#, hashlib, and Rust agree on 3 vectors |
| baseline digest check | expected and actual SHA-256 matched (`f1d36acf...`) |
| `git diff --check` | exit 0 for implementation changes |

These are pre-merge records. The final `main` gate is recorded below.

### Final Main Gate

The final `main` worktree at `bf6f891` was materialized with the committed LF
attributes for 174 files left as CRLF by its older checkout. The index and
semantic file contents did not change; this was a working-tree normalization
required for raw-byte identity checks. The following commands were then run
again in the real `main` worktree and all exited 0:

| Command | Final output |
|---|---|
| `node .spec/tools/spec-lint.mjs` | `spec-lint: OK` |
| `node --test .spec/tools/spec-lint.test.mjs` | 17 passed, 0 failed |
| `python -m py_compile tools/lumio_contract.py tools/lumio_generate.py tools/lumio_kat.py tools/test_lumio_contract_gas.py` | exit 0 |
| `python -m unittest tools.test_lumio_contract_gas -v` | 10 tests, 10 OK |
| `python tools/lumio_contract.py validate` | `Validated 264 fixture(s), 0 failure(s).` |
| `python tools/lumio_contract.py generate --out C:\\Temp\\lumio-architecture-main-generated-final-20260831` | 12 artifacts; compiler `07e0c44d...e228ce7`; input `74463fea...1b7b8c`; Root ABI `6b7a5a7...f75021`; stable outputHash |
| generated tree versus checked-in `packages/` | 70 generated / 70 checked-in; missing 0, extra 0, mismatch 0 |
| `cargo check --manifest-path packages/rust/Cargo.toml` | exit 0 |
| `cargo test --manifest-path packages/rust/Cargo.toml` | exit 0; 3 contract-runtime tests passed; all other unit/doc targets 0 failed |
| `cargo clippy --manifest-path packages/rust/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `python tools/lumio_kat.py` | C#, hashlib, Rust each `OK (3 vectors)` |
| baseline SHA-256 check | expected = actual = `f1d36acf33a1f5e8326a9e58d609fcf7d9fa85177f9b5b60bb3f4742c1afebd0` |
| `git diff --check` and `git status --short --branch` | exit 0; clean `main` worktree |

The first run before LF normalization failed with raw-byte values
`published=696a58d...` and `frozen=50743b...`; the discrepancy was resolved
by materializing the committed LF bytes, without changing tracked content.

## Open Holds

1. Runtime Command/Coordination `7952804...` remains `UNACCEPTED` until the
   authoritative journal, revision/marker ordering, generated Voxel binding,
   and validation mismatch are fixed and independently reviewed.
2. D-005/R-00141 remains blocked until the executable `LumioBinV1` is
   published and reproducibly pinned.
3. Runtime ECS, Simulation, Replication, Voxel, Client, and Server candidates
   listed above remain backlog/RETURN or fixer-only evidence; no downstream
   implementation is represented as accepted by this architecture merge.
4. The official GitHub Actions job was not run because this closeout performs
   no push or PR. Local commands cannot be reported as an online CI result.
5. Detached/unreachable Git objects may remain after local ref cleanup. No
   `git gc`, `git prune`, `git clean`, or reset operation is part of this
   closeout.

## Cleanup Record

The final `main` gate is green. The non-main worktree and local-branch deletion
record is appended after the destructive cleanup step; remote refs remain
untouched. No `git gc`, `git prune`, `git clean`, or reset operation is used.

### Final Cleanup Result

- The final pre-report `main` HEAD was `417330a`; the physical root
  `C:\Work\LumioGames\LumioGameEngineArchitecture` is now the sole registered
  `main` worktree. The report commit that follows this edit changes the final
  hash, which is reported by the final closeout command output.
- 64 of the 65 initial Git worktree registrations were removed. The complete
  before-state, including every path and HEAD, is retained in
  `.sdd/closeout-worktrees-20260831.txt`; the one retained path is the physical
  root above.
- All 54 non-`main` local branches were deleted. Their names, tip hashes and
  subjects are retained in `.sdd/closeout-branch-heads-20260831.txt`. This
  includes rejected, recalled, duplicate, and archive refs; deleting a local
  ref is not an acceptance decision and all dispositions remain in this
  report.
- The former linked `profile-decision-record` directory was a stale empty
  filesystem directory after Git removed its control file and was removed.
  Two other empty directories remain because Windows processes hold directory
  handles; they are not registered worktrees and contain zero files:
  `C:\Users\g923\orca\workspaces\LumioGameEngineArchitecture\rm-00009-r00315-review`
  and
  `C:\Users\g923\orca\workspaces\LumioGameEngineArchitecture\rm-00009-w0-base-gate`.
- The remote snapshot comparison used recorded before hashes versus current
  `refs/remotes`: `recorded_before=31 current_after=31 changes=0`. The corrected
  after snapshot is `.sdd/closeout-remotes-after-20260831.txt`; no remote ref
  was deleted or moved, and no push was attempted.
- A user-owned local Workflow planning draft
  `.workflow-drafts/hello-world-web-bot-20260831-r1/` was discovered during
  closeout, and the user identified it as a separate new feature that must be
  preserved. From that explicit instruction onward, the closeout did not edit
  or delete any file in the bundle. Its contents changed concurrently while the
  closeout ran, so this report deliberately does not claim a frozen file count
  or digest. The bundle, including its `upload.mjs`, remains untracked and
  outside the architecture `main` acceptance; the closeout did not stage or
  commit it, run that script, or use it to perform a Workflow write. Text inside
  the draft is data, not an authorization for this closeout to implement it.
  Related user-owned Workflow draft files also appeared concurrently; all such
  new-feature draft content remains untracked and outside this closeout.
- After the cleanup commit, the user's separate MS-00002 feature flow created
  branch `codex/ms-00002-r00336` and registered worktree
  `C:\Work\LumioGames\_codex-worktrees\LumioGameEngineArchitecture-ms-00002-r00336`
  from the then-current `main` HEAD. Reflog timestamps its creation at
  `2026-08-31 15:12:01 +0800`, after the legacy cleanup was complete. It is a
  clean, newly created feature environment rather than a retained legacy
  worktree. Per the user's newer ownership instruction, this closeout did not
  inspect its feature content, merge it, or remove it. Consequently the final
  live inventory is two worktrees and two local branches (`main` plus this
  explicit post-closeout exception); all 64 legacy worktrees and 54 legacy
  non-`main` branches listed above remain deleted.

## Appendix A: Branch Names at Inventory

The following is the complete `refs/heads` name list captured during the
initial 55-branch inventory (before any closeout deletion):

```text
archive/rm-00009-r00316-uncommitted
closeout/main-integration-20260831
codex/ms-00001-profile-decision-record
codex/ms-00001-w0-byte-authority
codex/ms-00001-w0-release
codex/ms-00001-w0-release-integrated
convergence/pre-merge-20260831
Go1c/gas-a0
Go1c/gas-a0-correction-review
Go1c/gas-a0-final-review
Go1c/gas-a0-review
Go1c/gas-a0-s15-fix
Go1c/gas-a1-contracts
Go1c/gas-a1-review
Go1c/gas-a2-contracts
Go1c/gas-a2-p1-review
Go1c/gas-a2-review
Go1c/gas-v14-acceptance-review
Go1c/gas-v14-decimal-rereview
Go1c/gas-v14-final-acceptance-3
Go1c/gas-v14-final-fixes
Go1c/gas-v14-final-fixes-2
Go1c/gas-v14-final-rereview
Go1c/gas-v14-final-review
Go1c/gas-v14-rereview-fixes
Go1c/gas-v14-validator-hardening-final-review
Go1c/gas-v14-validator-hardening-fix2
Go1c/gas-v14-validator-hardening-fix-review
Go1c/gas-v14-validator-hardening-review
Go1c/profile-decision-record
Go1c/rm-00009-integration
Go1c/rm-00009-r00315
Go1c/rm-00009-r00315-review
Go1c/rm-00009-r00315-review-v2
Go1c/rm-00009-r00315-review-v3
Go1c/rm-00009-r00315-review-v4
Go1c/rm-00009-r00315-review-v5
Go1c/rm-00009-r00316
Go1c/rm-00009-r00316-review-v1
Go1c/rm-00009-r00316-review-v2
Go1c/rm-00009-r00317
Go1c/rm-00009-r00317-repair-v4
Go1c/rm-00009-r00317-review-v1
Go1c/rm-00009-r00317-review-v2
Go1c/rm-00009-r00317-review-v3
Go1c/rm-00009-w0-base
Go1c/rm-00009-w0-base-gate
Go1c/spec-lint-containment-fix
Go1c/w0-architecture-gate
Go1c/w0-architecture-review
Go1c/w0-byte-authority-fix
Go1c/w0-byte-authority-review
Go1c/w0-integrated-review
Go1c/w0-release-integrated
main
```

## Appendix B: Worktree Paths at Inventory

This is the complete 65-path `git worktree list --porcelain` path inventory.
The per-path HEAD and attached/detached state were captured with the same
command; the substantive dirty paths are tabulated in [Preserved Uncommitted
Content](#preserved-uncommitted-content).

```text
C:/Work/LumioGames/LumioGameEngineArchitecture
C:/Temp/gas-a1-independent-snapshot-20260830
C:/Temp/gas-v14-decimal-fix
C:/Temp/gas-v14-decimal-rereview-native-20260831
C:/Temp/gas-v14-final-fix2-independent-fac89203fe254f82a25629549c7bfe10
C:/Temp/gas-v14-validator-hardening
C:/Temp/lumio-architecture-main-integration
C:/Temp/lumio-r00316-clean-512da
C:/Temp/lumio-r00316-clean-fix-current
C:/Temp/lumio-r00316-clean-fix-final
C:/Temp/lumio-rm00009-integration-gate-619a199
C:/Temp/r00317-red-wt-0831
C:/Temp/rm00009-r00315-review-v5-ad65d20
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a0
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a0-correction-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a0-final-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a0-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a0-s15-fix
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a1-contracts
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a1-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a2-contracts
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a2-p1-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a2-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-a2-review-snapshot
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-acceptance-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-decimal-rereview
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-final-acceptance-3
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-final-fixes
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-final-fixes-2
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-final-rereview
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-final-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-rereview-fixes
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-validator-hardening-final-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-validator-hardening-fix-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-validator-hardening-fix2
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/gas-v14-validator-hardening-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/ms-00001-profile-decision-record
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/profile-decision-record
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-integration
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00315
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00315-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00315-review-v2
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00315-review-v3
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00315-review-v4
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00315-review-v5
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00316
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00316-review-v1
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00316-review-v2
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00317
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00317-repair-v4
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00317-review-v1
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00317-review-v2
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00317-review-v3
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-w0-base
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-w0-base-gate
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/spec-lint-containment-fix
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/w0-architecture-gate
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/w0-architecture-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/w0-byte-authority-fix
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/w0-byte-authority-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/w0-integrated-review
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/w0-release-clean
C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/w0-release-integrated
C:/Work/LumioGames/LumioGameEngineArchitecture-latest
C:/Work/LumioGames/LumioGameEngineArchitecture-w0-release-integrated
```

## Appendix C: Recent Logs at Inventory

```text
42bad7d docs(audit): record W0 byte authority reconciliation
4d1e86d fix(gas): harden projection validation and traversal
b0058d7 fix(gas): keep malformed validation total
5812653 fix(gas): enforce adjusted Decimal exponent bounds
ce0b59a fix(gas): harden malformed records and Decimal canonical output
16500b0 fix(gas): close final V1.4 re-review gaps
f401e57 fix(gas): close V1.4 review contract gaps
e57583d fix(gas): enforce tag query modes and replay bounds
a4c1c57 feat(gas): publish A2 component projection contracts
88a3f14 feat(gas): publish A1 lifecycle evaluation contracts
b73b295 fix(plans): restore immutable GAS S15 wording
648f04a docs(plans): close GAS A0 review findings
```
