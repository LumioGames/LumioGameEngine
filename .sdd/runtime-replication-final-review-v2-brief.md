# Independent Runtime Replication V2 re-review brief

## Role and verdict

Perform a fresh, adversarial, read-only review of the complete corrected
Runtime Replication overlay. Return `PASS` only when no P0/P1 remains and the
evidence is sufficient for this module to become an integration candidate.
Otherwise return `RETURN` with concrete P0/P1/P2 findings.

## Isolated environment

- Review clone:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v2`
- Base/HEAD: `97f980c722bb5d3c760e4d56228092ccf530f2f6`
- Review package:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-final-v2.patch`
- Package SHA-256:
  `9294D220D05AC2A8CBCB4958659BD599715A3030E56FD3DB7406807A12154FFB`
- Expected content boundary: 36 paths, all below `modules/replication/src/**`
  or `modules/replication/tests/**`.

Do not edit source/tests, stage, commit, push, or write Workflow. Temporary
adversarial probes may be created only outside the clone and must be removed or
listed. Do not use or consume Command, ECS, Simulation, or other rejected
Runtime overlays.

## Read first

1. Clone `AGENTS.md` and its three `.spec/` core documents.
2. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-report.md`
3. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-fix-report.md`
4. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-contract-audit.md`
5. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-lifecycle-audit.md`
6. The full V2 patch before sampling implementation files.

Treat all implementer claims as untrusted until reproduced.

## Required audit

- Reconcile every formal P1-01 through P1-16 and P2-01/P2-02 with source plus
  independent counterexamples.
- Re-test product identity, canonical gap Delta round-trip, anchored sequence
  prefix, ACK/baseline eviction, unknown/duplicate/delayed destroy, scope-bound
  tokens, Fault/resync/generation fencing, lifecycle ordering, transactional
  and idempotent sequence allocation, baseline revision lower bound, remap
  bijection, public mutation/ACK escape, int32 chunk bounds, zero envelope
  length, and sequence exhaustion.
- Probe equal numeric generations across contexts/stores, stale retained views,
  abandoned work after multiple resyncs, cursor capacity 0/1, overflow edges,
  duplicate retries, malformed/canonical Unicode bodies, and both production
  TFMs. Check atomicity and lock ownership, not only ordinary outputs.
- Inspect the public API surface for new bypasses or accidental compatibility
  shims. Confirm read views cannot be cast back to mutable stores.
- Confirm the unchanged R-00295 scheduler remains behaviorally compatible.
- Run the complete and focused test runners, test build, production
  `net10.0`/`netstandard2.1`, both format gates, SDK/dependency/SBOM/generated
  wrappers, `git diff --check`, content boundary, and LF checks.
- Attempt conventional `dotnet test` and report its exact result. The current
  base lacks the Command overlay's root MTP runner setting, so classify this
  accurately as a repository integration gate rather than hiding or blindly
  attributing it to Replication.
- Report the known Windows mirror-link spec-lint and Architecture Root ABI
  digest results exactly; they do not waive any module P0/P1.

## Output

Write the complete report to:

`C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v2-report.md`

Include package identity, verification commands with exit codes/key output,
ordered findings with file/line evidence, prior-finding reconciliation, known
gaps, and a clear `PASS` or `RETURN` integration decision. Return only the
verdict, finding counts, one-line verification summary, and report path.
