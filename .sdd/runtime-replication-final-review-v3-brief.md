# Independent Runtime Replication V3 final review brief

## Verdict

Perform a fresh adversarial, read-only review of the corrected Runtime
Replication aggregate. Return `PASS` only if no local P0/P1 remains; otherwise
return `RETURN` with the complete finding set. Do not trust the 142/142 runner
without reproducing independent counterexamples.

## Isolated environment

- Review clone:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v3`
- Base/HEAD: `97f980c722bb5d3c760e4d56228092ccf530f2f6`
- Package:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-final-v3.patch`
- SHA-256:
  `92FA047248B70D1A1752880CC73CFF59714062D58F91E814A81DC0509215DD16`
- Expected boundary: 36 paths, all below Replication source/tests.

Review only. Do not edit candidate source/tests, stage, commit, push, or write
Workflow. Temporary probes belong outside the clone and must be removed or
listed.

## Read first

1. Clone `AGENTS.md` plus its three `.spec/` core documents.
2. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v2-report.md`
3. `C:\Work\LumioGames\_codex-verification\runtime-replication-v2-p1-fix-report.md`
4. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-report.md`
5. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-fix-report.md`
6. The full V3 patch before source sampling.

## Required audit

- Reproduce the V2 P1-01 FullSnapshot duplicate cases: older/non-pending,
  pending, acknowledged, active, direct projection, capacity 0/1, sequence
  exhaustion, lifecycle reset, cross-context/generation, and changed payload.
- Reproduce V2 P1-02 Delta duplicates before and after ACK, after eviction,
  same range with changed revision/mappings/body/gap/tombstones, capacity 0/1,
  QueueFull retry, release/reset/resync, collision, and sequence exhaustion.
- Verify idempotency identity is canonical, strict, collision-resistant and
  excludes only the allocated sequence. Confirm bounded caches cannot evict a
  still-retryable authoritative request or grow without budget accounting.
- Reproduce V2 P1-03 through every context tombstone path: add, destroy,
  duplicate/unknown/delayed destroy, bind fence, view, collect, release,
  generation reset, repeated resync, Fault, Close, and Dispose. Prove there is
  one canonical dictionary/authority, not synchronized mirrors.
- Re-run independent probes for all 16 prior P1 and two P2 families so the new
  cache/tombstone changes do not reopen token, history, lifecycle, canonical
  JSON, revision, remap, public API, or scheduler defects.
- Inspect locking/reentrancy and multi-context isolation for the new caches and
  shared tombstone state on both TFMs.
- Run locked restores, test build, full and focused runners, both production
  TFMs, both format gates, SDK/dependency/SBOM/generated wrappers,
  spec-lint/self-tests, conventional `dotnet test`, Architecture validation,
  `git diff --check`, reverse-check, 36-path boundary, index, LF, trailing
  whitespace, and final-newline checks.

## Output

Write:

`C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v3-report.md`

Include package identity, exact commands/results, ordered findings, complete
V2/prior reconciliation, known external gaps, and the integration decision.
Return only verdict, finding counts, one-line verification summary, and path.
