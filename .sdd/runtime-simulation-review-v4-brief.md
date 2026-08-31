# Runtime Simulation V4 Independent Deep Review

## Objective

Independently review the complete Simulation delivery from
`ef822a76cd5586513ea6e52b3ea4f5497917bdc8` through candidate
`97f980c722bb5d3c760e4d56228092ccf530f2f6` plus the final 30-path
uncommitted overlay. Reconcile all prior 2 P0/7 P1 findings and seek new
correctness failures in authority, lifecycle, execution controls, hashing,
durable replay and failure evidence.

Review only. Do not edit implementation/tests, stage, commit, push, write
Workflow, or consume the candidate.

## Materials

- Authoritative worktree:
  `C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-simulation-review-v4`
- Prior independent RETURN:
  `C:\Work\LumioGames\_codex-verification\runtime-simulation-followup-review-report.md`
- Implementer report:
  `C:\Work\LumioGames\_codex-verification\runtime-simulation-review-v3-fix-report.md`
- Complete module package relative to `ef822a`:
  `C:\Work\LumioGames\_codex-verification\runtime-simulation-review-v4.patch`
- Required report:
  `C:\Work\LumioGames\_codex-verification\runtime-simulation-review-v4-report.md`

Read target AGENTS/.spec core docs, reviewer/before-you-code/testing standards,
Simulation README/phase contracts and relevant architecture/ADR mirrors.

## Mandatory Matrix

1. Public authority/composition: compile an external assembly and enumerate all
   public constructors, factories, interfaces, properties and reflection-free
   injection paths. Arbitrary nominal/no-op executors must not receive
   authoritative capability or commit. Public/default composition fails closed;
   internal/test success paths cannot be reached by external callers.
2. Lifecycle/ownership: test Created/Running/Paused/Faulted/Disposed,
   non-owner Dispose/fault, reentrant Dispose from every phase, concurrent
   cleanup and commit-time revalidation. No lifecycle closure can race through a
   successful commit; cleanup remains deterministic.
3. Execution controls: verify cancellation, logical deadline, elapsed timeout,
   work/command/processor budgets and cooperative-check requirements are actual
   enforcement, not only exception mapping or self-attestation. Probe a slow or
   noncooperative executor, zero/overflow budgets and cancellation at each
   commit boundary; do not credit hard preemption the implementation lacks.
4. Ingress/seed/immutability: malformed/null/overflow/invalid UTF input stays in
   the stable result boundary, session remains coherent, and limits apply before
   copy/hash/publication. Configured seed is the sole authority across request,
   executor, hash and replay. All captured/output/result/control views are
   immutable after run.
5. State hash authority: a commit requires unforgeable complete contributors
   for session/world/release/config/manifest, revisions, ECS/Command/
   Coordination/Voxel/GAS/Replication, phases, tokens, input and committed
   output. Missing/duplicate/self-declared contributors fail closed. Different
   committed state/output cannot share a hash.
6. Post-commit/fail-stop: all post-finalize ProcessFault phases preserve
   committed identity/output and exact IdempotentSame replay while faulting the
   runner/session and rejecting subsequent ticks. First-failure evidence remains
   stable across retries and persistence failures.
7. Durable replay: test >256 ticks, restart/readback, cache eviction, exact and
   mismatched digest, release/config/world/manifest/session/epoch binding,
   corruption, bounded eviction and unavailable storage. Public/default
   authoritative composition cannot treat an in-memory-only store as
   process-loss durability.
8. Durable failure bundle: trace write/read ordering for pre/post-commit crash
   windows, persistence failure, snapshot versus NoSnapshotReason/bootstrap,
   revisions and prepared/participant tokens. Evidence must be immutable,
   content-addressed and reconstructable after process loss; missing/corrupt
   evidence fails closed without replacing the primary failure.
9. Commit/cache/determinism and compatibility: recheck business rejection,
   finalize guard, TargetTickId ordering, max TickId, cache atomicity, stable
   errors, concurrency/overflow, API visibility and self-fulfilling tests. Run
   locked restore, both TFMs Release, full 110+ tests, external probes,
   dependency/generated/format/diff gates and exact scope checks.

## Verdict

Write `PASS` or `RETURN`; any P0/P1 means RETURN. Report all nine coverage
dimensions, exact file/line and failure scenario, fresh commands/counts, prior
finding reconciliation, known gaps and integration-consumption decision.
