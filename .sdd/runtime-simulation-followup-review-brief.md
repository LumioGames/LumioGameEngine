# Runtime Simulation Full Follow-up Deep Review

## Objective

Independently review the complete Runtime Simulation delivery from baseline
`ef822a76cd5586513ea6e52b3ea4f5497917bdc8` through candidate
`97f980c722bb5d3c760e4d56228092ccf530f2f6` plus the final uncommitted
follow-up overlay. Review the whole module, not only the most recent patch.

Review only: do not edit implementation/tests, stage, commit, push, write
Workflow, or consume the candidate for integration.

## Materials

- Authoritative review worktree:
  `C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-simulation-followup-review`.
- Follow-up fix brief:
  `C:\Work\LumioGames\_codex-verification\simulation-commit-followup-review-fix-brief.md`
- Implementer report:
  `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\runtime-simulation-fix-evidence.md`.
- Complete Simulation diff package relative to `ef822a`:
  `C:\Work\LumioGames\_codex-verification\runtime-simulation-full-review.patch`.
- Required reviewer report:
  `C:\Work\LumioGames\_codex-verification\runtime-simulation-followup-review-report.md`

Read target `AGENTS.md`, `.spec` core docs, `before-you-code`, reviewer rules,
repository/testing standards, Simulation README, phase contracts, and relevant
architecture/ADR mirrors before review.

## Mandatory Review Matrix

1. Session authority and lifecycle: the only public tick entry must enforce
   session state, owner thread, epoch, fault and disposal fences. Recheck public
   `Runner`/`TickRunner.Run`, non-owner `Dispose`/fault, reentrant handler-driven
   dispose, Created/Running/Faulted/Disposed paths, and configured seed versus
   request seed. No committed tick may bypass `SimulationSession.RunTick`.
2. Executor composition: installing thirteen delegates or test no-ops must not
   prove authoritative ingress/planning/native/ECS/Voxel/GAS/projection
   capability. Verify explicit executor capabilities, fail-closed incomplete or
   fake composition, phase ownership, and default/public construction paths.
3. Ingress and immutability: malformed/null ingress must return a stable result
   and cannot escape as an exception while leaving the runner usable. Captured
   `HostTickRequest.Inputs`, payloads, execution context, outputs and results
   must be immutable/defensively owned. Late/cross-thread output emission after
   Run must fail without mutating state. Canonical ordering must include
   `TargetTickId` and remain permutation deterministic.
4. Cancellation/deadline/budgets: verify actual cancel points and enforcement,
   not only exception remapping. A handler cannot run indefinitely past a
   supplied cancellation/deadline/logical or processor budget. Preserve stable
   `Cancelled`, `TimedOut`, and `BudgetExceeded` identities with zero
   uncommitted output; unrelated exceptions remain fail-stop.
5. Phase outcomes and commit semantics: contract-designated business rejection
   phases return `Rejected` without fault/commit. Infrastructure/processor
   failures fault. After `GasAndEventFinalize`, failures in projection,
   snapshot/hash/metrics, or egress preserve authoritative committed identity
   while surfacing post-commit failure. Duplicate replay of that exact committed
   attempt must obey `IdempotentSame` semantics.
6. Commit/cache/tick atomicity: `MarkCommitted` requires the actual finalize
   phase record to be completed. Check result cache ordering, fault transitions,
   next-tick arithmetic and `ulong.MaxValue`; a first attempt cannot report
   Faulted while caching a successful duplicate. Replays with same/different
   request hash must remain coherent before and after commit.
7. Determinism/state hash: seed, canonical ingress, phase results, revisions and
   committed output must feed deterministic identities as specified. Verify no
   mutable view or post-run action can change a cached/request/state hash.
8. Compatibility, tests and scope: review the complete module diff for public
   API compatibility, aliases, concurrency/reentrancy, exception/error mapping,
   false-success paths, overflow, and self-fulfilling tests. Run locked restore,
   dual-TFM Release builds, the complete Simulation test assembly, dependency,
   format and diff gates, and applicable contract checks from the independent
   snapshot. Record repository-wide MTP/spec-link environment gaps separately.

## Verdict

Write `PASS` or `RETURN`; any P0/P1 means `RETURN`. The report must include all
eight coverage dimensions, evidence-based findings with exact file/line and a
concrete failure scenario, fresh command exits/counts, reconciliation of every
known finding above and in the fix brief, known gaps, and an explicit
  integration-consumption decision.
