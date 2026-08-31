# Runtime Replication Full Follow-up Deep Review

## Objective

Independently review the complete Runtime Replication delivery from repository
baseline `ef822a76cd5586513ea6e52b3ea4f5497917bdc8` through candidate
`97f980c722bb5d3c760e4d56228092ccf530f2f6` plus the uncommitted review-fix
overlay. This is a whole-module review, not an overlay-only confirmation.

This is review-only work. Do not edit source/tests, stage additional changes,
commit, push, write Workflow, or consume the candidate for integration.

## Required Materials

- Independent worktree:
  `C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-replication-followup-review`
- Original verified-finding/fix brief:
  `C:\Work\LumioGames\_codex-verification\replication-97f980c-review-fix-brief.md`
- Implementer handoff evidence:
  `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\runtime-replication-review-fix-evidence.md`
- Complete Replication review package relative to `ef822a`:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-full-review.patch`
- Required report output:
  `C:\Work\LumioGames\_codex-verification\runtime-replication-followup-review-report.md`

Read the target repository `AGENTS.md`, its three `.spec` core documents,
`before-you-code`, `reviewer.agent.md`, repository architecture, testing, the
Replication README, relevant generated-contract/ADR mirrors, and the scheduler
reference cited by the fix brief before drawing conclusions.

## Mandatory Review Matrix

Perform a deep adversarial review of the complete module and independently run
verification. Do not accept implementer test counts as evidence.

1. Permission/admission: prove every pre-queue path invokes the generated
   `ProtocolGate` with authoritative role, claims, session/connection generation,
   schema, and capability context before parsing, applying, or queueing payloads.
   Try stale-generation, missing-role/claim, malformed-body, and partial-state
   cases. Confirm no hand-coded permission matrix competes with the generated
   contract and stable generated errors survive mapping.
2. Cross-TFM envelope parsing and integrity: trace the net10.0 and
   netstandard2.1 paths. Falsify duplicate/extra member, wrong type, numeric
   overflow, malformed escape/Unicode, invalid revision order, schema epoch,
   tombstone/gap, and trailing-data handling. Verify SHA-256/CRC recomputation is
   over the contract-defined canonical body bytes and a tampered body cannot pass
   a shape-only or self-declared digest check.
3. Lifecycle/history: exercise Created, Snapshotting, AwaitingBaselineAck,
   Active, resync, closure, and fault paths. Delta must require acknowledged
   baseline and Active state. Verify ACK cursor movement, jump/replay behavior,
   partial baseline/delta eviction, bounded capacity reclamation, same-connection
   resync, and failure atomicity cannot dead-end or silently lose required state.
4. Identity/tombstones/revisions: verify namespace aliases and structural
   identity are canonical and collision resistant; stale generation remains the
   stable error. Removed NetEntityIds must retain reuse memory through the exact
   logical horizon and reject delayed resurrection/rebind. Check horizon and
   revision arithmetic boundaries, ordering, wraparound, and collection rules.
5. Projection: verify all payload/hash data are defensively owned, exposed views
   cannot mutate stored batches, every byte counted by the admission budget is
   bounded with overflow-safe arithmetic, and schema epoch/revision validation is
   consistent between full snapshots and deltas.
6. Scheduler: compare the integrated scheduler against the reviewed `f0584a6`
   semantics. Cover bounded queue/identity, generated permit checks before send,
   denied-head progress, deterministic jitter/frequency, truncation returning the
   original revision, starvation limits under sustained higher priority arrivals,
   fixed clocks, requeue, `cap=0`, and all Normal/Congested/Slow threshold edges.
7. Compatibility, tests, and scope: inspect public API compatibility, aliases and
   mutability, exception/error mapping, concurrency/reentrancy, deterministic
   ordering, false-success paths, test assertion quality, and out-of-scope
   changes. Run locked restore, both production TFMs in Release, the complete
   Replication test assembly, dependency/format/diff gates, and applicable
   contract checks from this independent worktree. Record standard `dotnet test`
   behavior separately if repository-wide MTP configuration is absent at this
   candidate base.

## Verdict Contract

Write the full report to the required output path and return only concise status.
The report must contain:

- Verdict `PASS` or `RETURN`; any P0/P1 requires `RETURN`.
- Coverage declaration for all seven dimensions.
- Findings ordered P0, P1, P2, each with exact file/line evidence and a concrete
  failing scenario; discard speculative findings.
- Fresh command evidence with exit codes and key counts.
- Reconciliation of every item in the original fix brief, including R-00295.
- Known gaps and an explicit integration-consumption decision.
