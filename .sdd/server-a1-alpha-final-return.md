# Server A1-alpha Final Return Intake

Target worktree: `C:\Users\g923\orca\workspaces\LumioServer\server-a1-alpha-integration`

Candidate state at intake: 60 staged paths on
`codex/ms-00001-server-a1-alpha-integration`, HEAD `5ec95ee269207c64281b6e3f9176ed4f7ab5952c`.
This document consolidates review findings for a bounded implementation pass. It
does not accept the candidate and is not a substitute for a fresh ORCA review.

## Blocking Findings

1. A1-alpha step 16 is not implemented. `SmokeClientRunner` emits
   `passed:false` and exit code 65 for the stale connection-generation scenario,
   while `ProcessAndExitCodeTests` currently makes that failure a green test.
   Do not invent a generated connection-generation field or a private public
   contract. If the published contract still lacks the required field, preserve
   this item as `BLOCKED_UPSTREAM` with exact publication evidence.
2. A1-alpha step 15 does not exercise the expiry/reconnect race. The timer-win
   path can reject with `SessionMismatch` while closing the connection without a
   client-visible registered StableErrorId. Add an actual simultaneous race
   fixture and ensure the loser receives a schema-owned error envelope.
3. SIGTERM/SIGINT returns from `Program` without executing the required Quiesce
   progression: `AdmissionClosed -> Drained -> SnapshotCut -> Stopped`. Add a
   process-level signal test and route signals through the same lifecycle.
4. `MvpTimerService.Schedule` checks disposal outside the lock. A concurrent
   `Dispose` can stop the supervisor and clear the queue before `Schedule` adds a
   timer, returning an id for a timer that can never fire. Reproduce this race,
   then make schedule/dispose linearizable.
5. Session delta ACK handling is not correlated to the outstanding delta. A
   schema-valid forged jump can advance the cursor past unsent revisions. Track
   the bounded pending sequence/toRevision identity, define idempotent replay,
   and reject forged or post-reconnect ACKs.
6. Session owner-gate mutual exclusion is not an owner-lane guarantee. Transport
   and admin callers can execute reducers on their own threads through immediate
   `PumpOnce` calls. Make external paths enqueue-only or enforce an explicit
   owner identity with typed completion.
7. Release compensation drops tracking after `ReleaseAdmission` failure, which
   can leak committed capacity. Preserve a bounded retry/dead-letter record and
   escalate non-recoverable cleanup failures before deleting ownership state.
8. `WriteAdminAudit` advances `auditSequence` outside the owner lane. Parallel
   admin calls can duplicate or reorder IDs; make allocation atomic/owner-bound
   and add a concurrency regression.

The earlier detailed findings and intermediate-fix evidence remain authoritative
inputs in `.sdd/server-a1-alpha-review-findings.md`, including synthetic revision,
proof-constructor IVT, public command-surface expansion, terminal reserve bounds,
and Transport disposal semantics. Re-read and close every still-applicable P1,
not only the eight items above.

## Non-blocking Cleanup

- Zero decoded credential bytes on every early return, including MaxConnections.
- Dispose connection-level CTS/Semaphore resources on all close paths.
- Keep terminal close deadline anchored to the first close request; preserve the
  already-tested one-second continuous-backpressure behavior.

## Gate And Commit Hygiene

- Preserve all pre-existing staged candidate files. Do not reset, clean, or
  overwrite unrelated state.
- Keep `GateHelpers.sh`, `verify-gate-portability.sh`, and
  `verify-integration.sh` at mode `100755`; PowerShell files remain `100644`.
- Before handoff, verify the complete index contains all new gate dependencies.
- Run the focused RED/GREEN tests first, then the repository's Bash and
  PowerShell verification entry points serially. Do not run concurrent .NET
  builds in the same worktree.
- Do not commit, push, update Workflow, change generated public contracts, or
  claim A1-alpha 17/17. High-risk changes require a fresh independent reviewer.

## Fixed Cross-track Boundaries

- Runtime candidate `79528044f758d188844270bc7e55decce2a7b0cc` remains
  unaccepted.
- `R-00141` remains blocked until an executable upstream `LumioBinV1` codec is
  published.
