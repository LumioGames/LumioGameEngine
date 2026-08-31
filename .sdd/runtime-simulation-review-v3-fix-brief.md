# Runtime Simulation Deep Review V3 Fix Brief

## Context and Boundary

Owner worktree:
`C:\Work\LumioGames\LumioGameRuntime-simulation-commit-fix`

Base/HEAD remains `97f980c722bb5d3c760e4d56228092ccf530f2f6`; the 16-path
overlay is uncommitted. Independent report:
`C:\Work\LumioGames\_codex-verification\runtime-simulation-followup-review-report.md`.

Fix all 2 P0 and 7 P1 findings below with TDD. Preserve previously fixed
internal Runner surface, context closure, typed business rejection,
TargetTickId ordering, finalize guard, and committed duplicate identity. Only
modify `modules/simulation/src/**` and `modules/simulation/tests/**`. Do not
touch global/shared policy, Generated, ECS, Command, Coordination, Replication,
AOI, Architecture, NativeCore, Server, or Workflow. Do not commit/push or spawn
subagents.

## Required Fixes

### P0-1: No public nominal capability self-attestation

Public callers can implement the ten executor interfaces with no-op methods,
self-report `ExecutorId`/`IsAvailable`, receive `TickExecutorCapability.All`, and
commit.

- Remove public construction/injection of hand-authored nominal executor ports,
  or require unforgeable subsystem-issued capabilities whose provenance is
  validated by Simulation and cannot be constructed/self-attested by an
  external assembly.
- Until actual ECS/Voxel/GAS/Replication/etc owners provide those capabilities,
  public/default composition must fail closed and cannot execute a committed
  tick.
- Keep any reference/test success composition internal and explicitly marked as
  non-production; tests must compile an external no-op implementation and prove
  it cannot construct/inject an authoritative session.
- Do not invent public generated contracts or treat interface method presence as
  proof work occurred.

### P0-2: Dispose must obey owner and in-flight lifecycle fences

- A non-owner thread cannot dispose a bound Running session.
- An executor callback cannot reentrantly dispose the session during RunTick and
  then let that tick commit.
- Track in-flight execution explicitly; validate owner/state before disposal and
  revalidate lifecycle before/at commit. Preserve deterministic cleanup and
  avoid Monitor reentrancy becoming an authority bypass.
- Add Created/Running/Faulted/Disposed, non-owner, reentrant, concurrent and
  cleanup tests with explicit final states.

### P1-1: Enforce cancellation, deadline and budgets

- Introduce an explicit execution-control contract carrying cancellation,
  deadline and logical work limits into the public RunTick path and every
  authoritative executor.
- Enforce checkpoints before/after phases and cooperative checks within
  processor/executor work. Connect processor descriptor budgets, including
  MaxCommands; zero/invalid budgets cannot silently mean unlimited.
- A phase cannot commit after cancellation, timeout or budget exhaustion.
  Preserve stable `Cancelled`, `TimedOut`, `BudgetExceeded` with zero
  uncommitted output.
- Do not claim hard preemption that the implementation cannot provide. If a
  trusted synchronous executor must be cooperative, make that capability and
  check contract explicit and fail closed for executors that cannot satisfy it.

### P1-2: Malformed ingress stays inside the stable result boundary

- Validate all captured ingress fields and payloads before canonical hashing.
- Move canonicalization into the protected boundary and map malformed/null/
  invalid UTF-8/overflow cases to a stable rejected/fault result as specified.
- No exception may escape while leaving the session Running; add null SessionId,
  malformed payload, invalid lengths and follow-up tick tests.

### P1-3: Session seed is authoritative

- A configured session seed cannot be changed per request. Reject a mismatched
  request seed or remove request authority and consistently use the session
  seed, following the frozen Simulation contract.
- The short request constructor must not silently force seed 0 for a differently
  configured session.
- Hashing, executor context and replay identity must use the same authoritative
  seed; add mismatch and replay tests.

### P1-4: State hash covers authoritative committed identity

- A successful commit requires all contract-required hash contributors, not an
  arbitrary self-declared provider set.
- Include committed outputs and authoritative session/world/release/config,
  revision, ECS/Command/Coordination/Voxel/GAS/Replication/phase identities as
  required by the architecture. Missing contributors fail closed.
- Different committed output/state must not produce the same state hash; inputs
  and contributor views remain immutable after run.
- Coordinate this with internal capability provenance; do not fabricate hashes
  for unavailable integrations.

### P1-5: Post-commit process faults are fail-stop

- Preserve `IsCommitted=true`, committed result identity, outputs and
  `IdempotentSame` duplicate replay for a failure after finalize.
- Also transition the runner/session to Faulted for a phase with
  `ProcessFault + FailStop`; the next authoritative tick must be rejected.
- Test all three post-commit phases, first failure evidence, duplicate replay and
  next-tick behavior.

### P1-6: Committed replay survives bounded cache eviction

- An exact replay of any contract-retained committed Tick must return
  `IdempotentSame`, not `RevisionConflict`, after the 256-entry memory cache
  evicts it.
- Add a durable/bounded replay lookup seam with transaction identity and stable
  result evidence, or implement the architecture-defined retention mechanism.
  Do not solve this with unbounded process memory or a test-only store.
- Default composition fails closed when required durable replay capability is
  absent; cover 257+ ticks, restart/recovery, same/different digest and max TickId.

### P1-7: Persist a complete durable failure bundle

- Add/use an explicit durable failure-bundle port required by authoritative
  composition. Before returning a fail-stop result, persist the first failure
  evidence required for recovery: phase/error/fault action, tick/session/epoch,
  revisions, prepared/participant tokens where available, snapshot identity or
  explicit NoSnapshotReason, and deterministic evidence identity.
- Missing/corrupt persistence capability fails closed and cannot be reported as
  durable success. Preserve first-failure semantics and idempotent replay.
- Cover restart/readback, pre-commit and post-commit cases, persistence failure,
  snapshot/no-snapshot paths, and immutable evidence.

## Verification and Delivery

Write the report to:
`C:\Work\LumioGames\_codex-verification\runtime-simulation-review-v3-fix-report.md`.

Report real RED per finding, final GREEN counts, exact paths, locked restores,
dual-TFM Release builds, complete in-process tests, dependency/scoped format/
diff/generated-contract gates, known environment gaps, strict boundary proof,
and knowledge-sync decision. Leave all changes uncommitted for a fresh full
independent review.
