# Independent Runtime Command/Coordination authority re-review brief

## Role and decision

Perform a fresh adversarial, read-only review of the complete corrected Runtime
Command/Coordination overlay. Make two explicit decisions:

1. local Runtime code verdict: `PASS_LOCAL` only if no local P0/P1 remains,
   otherwise `RETURN`;
2. D-005 consumption verdict: `READY` only if all required public generated
   Voxel and portable durability contracts/evidence actually exist, otherwise
   `BLOCKED_UPSTREAM` or `RETURN` with exact blockers.

Do not treat green ordinary tests as proof of the authority/recovery contract.

## Isolated environment

- Review clone:
  `C:\Work\LumioGames\_codex-verification\runtime-command-authority-final-review`
- Base/HEAD: `1c53e2b7bc14d7f01a24a38ce6dc5d52448b2708`
- Package:
  `C:\Work\LumioGames\_codex-verification\runtime-command-authority-final.patch`
- Package SHA-256:
  `5821F5E6E064062C9095D4759E549EF26A7041E000B7DF732A5CA6B98F1DBD42`
- Expected content boundary: 53 paths, only `global.json`,
  `modules/command/**`, and `modules/coordination/**`.

Do not edit source/tests, stage, commit, push, or write Workflow. Temporary
probes may be created only outside the clone and must be removed or listed.
Do not consume rejected ECS, Simulation, Replication, or GAS overlays.

## Read first

1. Clone `AGENTS.md` and its three `.spec/` core documents.
2. `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\runtime-command-review-final-report.md`
3. `C:\Work\LumioGames\_codex-verification\runtime-command-review-v3-report.md`
4. `C:\Work\LumioGames\_codex-verification\runtime-command-v3-targeted-revision-audit.md`
5. `C:\Work\LumioGames\_codex-verification\runtime-command-v3-architecture-triage.md`
6. `C:\Work\LumioGames\_codex-verification\runtime-command-dependency-feasibility.md`
7. `C:\Work\LumioGames\_codex-verification\runtime-command-authority-fix-report.md`
8. `C:\Work\LumioGames\_codex-verification\runtime-txn-durability-contract-blueprint.md`
9. `C:\Work\LumioGames\_codex-verification\runtime-voxel-generated-port-blueprint.md`
10. The full 53-path patch before sampling implementation files.

Treat implementer and main-loop claims as untrusted until reproduced.

## Required audit

- Reconcile all original D-005 P1/P2, all V3 P1-01 through P1-11/P2, and both
  targeted revision/recovery findings with source plus independent probes.
- Audit the single authority operation across commit/recovery, durable intent,
  participant apply, result evidence, marker publication, revision publication,
  record publication, retry/idempotency, cancellation, and error paths.
- Probe reentrancy and concurrency through journal, participant, release,
  prepare, apply, and recovery callbacks. Test two coordinators sharing a store,
  fresh coordinator restart, stale tail/CAS races, partial append failure,
  marker/evidence mismatch, participant disagreement, tick/schema mismatch,
  reservation rollback, terminal append retry, and duplicate full identities.
- Verify no public state/revision/raw-apply/ACK/marker API can fabricate commit
  authority; no reflection or compatibility shim must restore a bypass.
- Verify revision advancement is exactly once and only after complete durable
  proof; recovery must not guess, downgrade, or publish partial local state.
- Audit full journal proof identity, framing, checksum/hash links, append
  receipts, stage ordering, capacity/overflow, and cross-session isolation.
- Confirm Command lifecycle drains and fences inflight, concurrent, reentrant,
  stale, and default-composition applies without no-op success.
- Confirm all Voxel substitute types are non-public and fail closed. Separately
  verify whether a callable Architecture-owned generated Voxel projection now
  exists; an internal shape is not a public contract.
- Verify the ten durable JSON fixtures independently recompute the claimed
  identity/vector/evidence/hash-chain fields and contain no test-side oracle
  that merely repeats production code.
- Attempt both `.NET 10` forms:
  `dotnet test --project <csproj>` and `dotnet test <csproj>`. Report exact
  results and decide whether the repository's required canonical workflow is
  genuinely repaired; do not hide the positional-form failure or assume its
  severity.
- Run both suites, all six production TFM builds, format checks where
  applicable, dependency/SDK/generated gates, spec-lint/self-tests,
  `git diff --check`, 53-path scope, staged-index, LF, trailing whitespace, and
  final-newline checks.
- Re-run live Architecture validation and distinguish Runtime-local safety from
  missing portable process-loss/generated public contracts.

## Output

Write the complete report to:

`C:\Work\LumioGames\_codex-verification\runtime-command-authority-final-review-report.md`

Include package identity, commands with exit codes/key output, ordered findings
with file/line evidence, complete prior-finding reconciliation, local verdict,
D-005 verdict, and known gaps. Return only both verdicts, finding counts, a
one-line verification summary, and the report path.
