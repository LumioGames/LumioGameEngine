# Runtime Replication V2 three-P1 fix brief

## Objective

Fix the complete finding set from the independent V2 re-review in the existing
uncommitted owner worktree:

`C:\Work\LumioGames\LumioGameRuntime-replication-aggregate-v1`

Read first:

1. Target repository `AGENTS.md` and its three `.spec/` core documents.
2. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v2-report.md`
3. `C:\Work\LumioGames\_codex-verification\runtime-replication-final-fix-report.md`
4. `C:\Work\LumioGames\_codex-verification\runtime-replication-token-fix-design.md`

The re-review verdict is `RETURN`, with exactly three P1 findings and no P0/P2:

1. FullSnapshot duplicate retry is not idempotent, including older/non-pending
   snapshots and direct public `ReplicationProjection` duplicate calls.
2. Delta duplicate retry is not idempotent before or after acknowledgement and
   creates a second sequence/gap.
3. `ReplicationContext` splits tombstone authority between the mapping table
   and registry, so context tombstones do not fence binds and destroys are not
   visible through the context tombstone view.

## Scope and constraints

- Only `modules/replication/src/**` and `modules/replication/tests/**`.
- Do not touch Command, Coordination, ECS, Simulation, generated contracts,
  Architecture source, Git index/HEAD, or Workflow.
- Do not commit, push, stage, or spawn subagents.
- Preserve every previously closed P1/P2 and the unchanged R-00295 scheduler.
- Follow TDD: add deterministic failing tests for all independent review
  counterexamples before production changes, then retain RED/GREEN evidence.

## Required behavior

- Introduce a stable, canonical FullSnapshot request/payload idempotency
  identity. A duplicate must return its original sequence without consuming a
  new one or replacing pending state, regardless of whether it is current,
  older, acknowledged, or invoked directly through the projection API.
- Introduce a stable, canonical Delta request/payload idempotency identity that
  excludes the allocated sequence but includes every authoritative input. A
  duplicate before or after ACK must return the original sequence/record and
  never append a second stream element or create a gap. QueueFull retry must
  keep the already-correct transactional sequence behavior.
- Bound any idempotency retention according to existing budget/history
  semantics. Do not add unbounded process-lifetime caches or admit cross-base,
  cross-session, cross-generation, or hash-collision reuse.
- Make one canonical tombstone authority under the shared context gate. Bind,
  destroy, add, collect, release, views, generation reset, resync, Fault, Close,
  and Dispose must observe the same horizon atomically. Do not mirror two
  independently mutable dictionaries.
- Add adversarial coverage for old/non-pending/acknowledged FullSnapshot
  duplicates, direct projection duplicates, Delta duplicates before/after ACK,
  same range with changed authoritative payload, capacity 0/1, sequence
  exhaustion, cross-context/generation identity, context add-tombstone then
  bind, context destroy visibility, collect/release, and lifecycle reset.

## Verification and report

Run focused RED/GREEN suites, the complete Replication runner, test build,
production `net10.0` and `netstandard2.1`, both format gates, SDK, dependency,
SBOM, generated wrappers, spec-lint/self-tests, `git diff --check`, content
boundary, staged-index, LF, trailing whitespace, and final-newline checks.

Write the complete handoff to:

`C:\Work\LumioGames\_codex-verification\runtime-replication-v2-p1-fix-report.md`

Return only `DONE`, `DONE_WITH_CONCERNS`, `NEEDS_CONTEXT`, or `BLOCKED`, plus a
concise test summary and report path.
