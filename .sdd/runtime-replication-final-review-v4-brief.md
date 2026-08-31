# Runtime Replication Final Review V4 Brief

## Role

Perform a fresh independent adversarial deep review of the exact Runtime
Replication V4 package. This is review-only work. Do not edit the owner
worktree, commit, push, stage, update Workflow, touch other Runtime modules, or
consume frozen Command/D-005 candidates. Apply/build/probe only in the isolated
review worktree created for this task.

## Frozen Identity

- Base and required review-worktree HEAD:
  `97f980c722bb5d3c760e4d56228092ccf530f2f6`
- Canonical V4 patch:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-final-v4.patch`
- SHA-256:
  `98C4487755354DCE5EF230591D5AE996061BA43669D85BECDB097B4747C0A380`
- Patch size: `514533` bytes
- Exact scope: `36` paths, `9070` additions and `350` deletions; path set is
  byte-for-byte identical to the V3 package path set.
- The owner worktree has 11 additional pre-existing tracked dirty paths. They
  are not in this patch and are protected by:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-preserved-11.sha256`
- Implementer report:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-v3-p1-fix-report.md`
- Prior V3 review:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v3-report.md`
- Output report:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-final-review-v4-report.md`

Verify hash, clean exact base, 36 paths/modes, apply check, empty index, and no
protected-path inclusion. Apply only in the isolated worktree and confirm
reverse applicability after application.

## Verdict Contract

Return `PASS` only with zero P0/P1. Otherwise return `RETURN` with every
finding's severity, exact locations, deterministic counterexample, and required
fix. Report P2 separately. Reconcile V3 P1-01 through P1-06 and all prior V1/V2
finding families. Ordinary `155/155` green evidence does not waive a reproduced
contract defect.

## Adversarial Review Lens

1. Complete replay budget: count and byte limits govern materialized payloads,
   keys, and the durable replay sequence/result together. There must be no
   unbudgeted second ledger, pinned-oldest starvation, hidden key growth, or
   successful outcome whose identity is later forgotten.
2. Transactional rejection: when a distinct or oversized identity cannot fit,
   return the existing typed capacity/Retryable result before sequence
   allocation, history/cache/cursor/lifecycle/publication mutation. Repeated
   rejected attempts consume no sequence and change no state.
3. Independently reproduce direct FullSnapshot and direct Delta A/B/C/retry-B
   at capacity one; B/C must reject while A remains retryable, retry-B must
   remain rejection, and the next legally accepted request after explicit
   release/reset must receive the next unconsumed sequence.
4. Reproduce oversized `HistoryBytes=1` first attempt/immediate retry and
   checked-size overflow for direct projection, BaselineStore, DeltaHistory,
   and context paths. No `retainIdempotency=false` success is allowed.
5. Context Baseline and Delta: test rotation, ACK, base eviction, changed same-
   key payload, exact replay before/after ACK/activation, resync, generation
   reset, release, Fault, Close, and Dispose. ACK alone must not forget a still-
   retryable outcome; legal reset/release must free capacity exactly once.
6. Byte/count cache accounting: every insertion/eviction/rejection/rollback
   keeps measured bytes within budget under overflow and concurrency. Probe
   capacity 0/1/max, item larger than capacity, sequence exhaustion, and
   exceptions during identity/framing computation.
7. Same-generation identity ordering: retain and compare `SourceRevision` for
   Alive/Destroy/tombstone/bind/collect/release; delayed older input cannot
   remove or resurrect newer state, and shared tombstone authority remains one
   scope-owned source.
8. Admission binding: authoritative admission session/product/release/message
   facts must bind the envelope and explicit expected identity in every
   overload; no caller-selectable second authority or malformed Unicode/ID
   bypass is permitted.
9. BaselineAck: duplicate after activation returns the original acknowledged
   result, changed/unknown ACK fails closed, and generation/work-epoch/reset
   transitions cannot reuse stale ACK authority.
10. Preserve lifecycle/token isolation, scheduler fairness/budgets, mapping
    bijection, canonical revision vectors/chunk IDs, projection immutability,
    integrity framing, QueueFull atomicity, sequence exhaustion, dual-TFM
    behavior, and all prior finding closures.
11. Public/scope integrity: no generated/public contract, other Runtime module,
    Command candidate `7952804`, Architecture, dependency, or protected-11
    path appears. Tests must be deterministic and fail against V3 before the
    repair rather than assert implementation internals.

## Verification

Run serially in the isolated worktree at minimum:

- new 42-case final reviewer probes and complete direct runner;
- every focused class listed by the report;
- locked restore, test Release build, production Release builds for net10.0
  and netstandard2.1;
- repeated capacity/replay/reset/concurrency/overflow probes;
- SDK, Bash/PowerShell dependency where executable, SBOM, generated wrappers,
  format, spec self-tests;
- exact 36-path/mode/text scan, `git diff --check`, patch reverse check,
  HEAD/index/untracked and no retained probe artifacts.

Run Architecture validation read-only and report the exact Root ABI gap. Keep
conventional MTP `dotnet test`, Windows junction lint, PowerShell JSON parser,
and Architecture digest issues separate. External gaps do not waive local
P0/P1.

## Handoff

Write the full report to the output path. Send exactly one `worker_done` with
verdict, P0/P1/P2 counts, full prior-finding reconciliation, P1-03 capacity/
sequence decision, report path, and confirmation that no candidate source was
edited outside the isolated review worktree.
