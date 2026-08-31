# Server A1-alpha Review P1/P2 Bounded Fix Brief

## Objective

Fix every locally owned finding from the final independent review with
deterministic TDD, while preserving the original staged index byte-for-byte.
This is one bounded fix wave for all seven P1 and two P2 findings. It does not
remove the independent A1 step-16 upstream block.

Read first:

- target repository `AGENTS.md` and its three `.spec` core documents;
- `C:/Work/LumioGames/_codex-verification/server-a1-alpha-final-review-report.md`;
- `C:/Work/LumioGames/_codex-verification/server-a1-alpha-final-fix-report.md`;
- `C:/Work/LumioGames/LumioGameEngineArchitecture/.sdd/server-a1-alpha-final-review-brief.md`.

Do not spawn subagents.

## Entry Identity And Index Safety

- Work only in
  `C:/Users/g923/orca/workspaces/LumioServer/server-a1-alpha-integration`.
- Required HEAD:
  `5ec95ee269207c64281b6e3f9176ed4f7ab5952c`.
- Entry index: exactly 60 staged paths and no untracked paths.
- Frozen cached index patch:
  `C:/Work/LumioGames/_codex-verification/server-a1-alpha-entry-index.patch`.
- Required cached patch SHA-256:
  `A3373327146DE2C0066F4A8D838247F07A3757897B92E038ABA7D7880F18BC33`.
- Complete pre-fix union patch:
  `C:/Work/LumioGames/_codex-verification/server-a1-alpha-final.patch`.
- Complete pre-fix union SHA-256:
  `4DBED0CC5D2A40528E0AAE3CC8EBC9FAB8A3942F4651DEA744DF59AC762D60E2`.

Before editing, verify HEAD, cached-patch hash, staged path count, exact current
overlay, and no untracked files. Preserve the index byte-for-byte throughout.
Do not use `git add`, reset, restore, checkout, clean, commit, push, unstage, or
Workflow writes. Add fixes only as unstaged working-tree changes. Stop and
report if entry identity differs.

## Exact Editable Boundary

Only these 14 paths may receive new working-tree changes:

1. `mvp-host/src/Lumio.Server.MvpHost.Transport/TransportService.cs`
2. `mvp-host/src/Lumio.Server.MvpHost.Session/SessionRegistry.cs`
3. `mvp-host/src/Lumio.Server.MvpHost.Transport.WebSocket/WebSocketByteCarrier.cs`
4. `mvp-host/src/Lumio.Server.MvpHost.App/FullGraphComposition.cs`
5. `mvp-host/src/Lumio.Server.MvpHost.Auth/MvpAuthorizationService.cs`
6. `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/ConnectionLifecycleTest.cs`
7. `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/TransportEventOrderingAndDiagnosticsTests.cs`
8. `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/TransportHarness.cs`
9. `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/BoundedQueueTest.cs`
10. `mvp-host/tests/Lumio.Server.MvpHost.Session.Tests/SessionBehaviorTests.cs`
11. `mvp-host/tests/Lumio.Server.MvpHost.Transport.WebSocket.Tests/CarrierBudgetAndFaultTests.cs`
12. `mvp-host/tests/Lumio.Server.MvpHost.App.Tests/ReadinessAndTraceTests.cs`
13. `mvp-host/tests/Lumio.Server.MvpHost.Auth.Tests/AuditAndErrorSemanticsTest.cs`
14. `mvp-host/tests/Lumio.Server.MvpHost.WorldSlot.Tests/WorldSlotFocusedTests.cs`

Do not edit HostContracts, generated/schema/ID/fixtures, project/dependency,
gate, Architecture, Runtime, Workflow, or public contract files. Do not change
script modes. If a finding cannot legally close within this boundary and the
existing frozen command/event vocabulary, return `BLOCKED` with exact missing
authority rather than expanding or inventing a contract.

## Required TDD Fixes

Add a deterministic behavioral RED for each item before its production edit,
then record the RED and GREEN result.

### P1-1 Transport service disposal terminal publication

Unify explicit close, overflow retirement, and service disposal through one
idempotent terminal path. For every live registry entry, close carrier, timer,
queues, and metadata; reserve and publish exactly one typed terminal event
before registry removal; preserve the winning reason and stale-generation
rejection. Full ordinary event capacity must not drop the terminal. Bound the
reserve and use the existing fail-stop/diagnostic behavior on exhaustion.
Test duplicate/concurrent dispose, throwing cleanup, full/zero-capacity lanes,
and no double terminal/resource disposal.

### P1-2 Immutable handshake ingress

Own a defensive byte snapshot at both normal `HandshakeEnvelope` and
authenticated-handshake ingress before any queue or callback retains it.
Header/body metadata must describe the owned bytes. Mutating or disposing the
caller buffer after enqueue must not change parsed identity, trace, release,
message, or authorization bytes. Cover normal and authenticated paths plus a
throwing/custom memory source where the existing type permits it.

### P1-3 Enqueue-only dependency callbacks

Every external transport/auth/authority callback, including same-thread
reentry from `Bind`, must enqueue immutable owner ingress. Never invoke a
reducer inline from a dependency callback while the owner gate/saga is active.
Drain after the admission saga reaches a safe boundary and revalidate epoch and
mapping identity. Reproduce synchronous `Closed` from `TrySend(Bind)` and prove
the connection cannot finish with a live Syncing binding.

### P1-4 Durable Unbind retry ownership

Do not remove `connectionSessions` ownership when `Unbind` returns Full,
Closed, stale, or throws. Retain the exact connection/epoch/session unbind
intent, retry only on the owner lane with a finite budget, retire it only after
accepted or proven stale, and emit stable diagnostic/dead-letter fail-stop
evidence on exhaustion. Test queue-full then success, persistent failure,
stale replacement, duplicate compensation, and exactly-once convergence.

### P1-5 Disposal of committed slot reservations

`SessionRegistry.Dispose` must not clear committed or pending release authority
before every reservation is released or represented by durable bounded
retry/dead-letter evidence that survives the shutdown boundary. Exercise active
committed sessions, pending cleanup, throwing/full/stale releases, multiple
reservations, and repeated dispose. No committed capacity may be silently
leaked or released twice.

### P1-6 Force-convergent WebSocket retirement

Repeated close/dispose must converge even when `CloseRequested` was already
true and no send loop can finish it. Idempotently mark closed, cancel and join
receive/send work within the bounded budget, dispose socket and synchronization
resources exactly once, remove the dictionary entry, and complete the state
task. Test the exact stuck `CloseRequested=true` race, active send/receive,
throwing cancel/dispose, and repeated concurrent disposal.

### P1-7 Session-local fault isolation

Do not call process `MarkFatalFault` for
`FaultAdjudication(SessionLocalProven, SlotMustFailStop=false,
SessionMustIsolate=true)`. Use only the existing typed Session/WorldSlot
vocabulary to route isolation to the affected local session where identity is
available, preserving a stable diagnostic/fail-closed result if delivery is
impossible. Slot/process/unproven critical faults still terminate the process.
Tests must prove one session-local fault does not complete the process fault
task or take unrelated sessions down, while slot/process faults remain fatal.
If the frozen event lacks sufficient affected-session identity, report this
specific local contract blocker instead of inventing a public field.

### P2-1 Throwing timer cancellation in WebSocket Touch

Catch and diagnose idle-timer cancel failure. Re-arm safely or fail-close under
the carrier lifecycle contract; never let an infrastructure timer exception
escape an otherwise valid receive. Test cancel throw, schedule throw, repeated
touch, and close/dispose races with exact resource/terminal outcomes.

### P2-2 Bounded authentication success reserve

Replace the unbounded success reserve with an explicitly bounded lane tied to
the declared auth event budget. Preserve accepted-result delivery priority,
and on reserve exhaustion take the documented diagnostic/admission-stop or
fail-stop path without silent success/drop. Test exact boundary, overflow,
drain/retry, repeated close, and sustained success bursts with bounded memory.

## Preserve

Preserve all previously closed findings: authority-revision evidence, frozen
public shape, one-shot auth proof, DeltaAck correlation, owner-lane admin and
revision paths, slot release retry/dead-letter, audit sequencing,
expiry/reconnect race, signal/quiesce ordering, timer Schedule/Dispose
linearizability, credential zeroing, first-close deadline anchoring, gate
parity/modes, and no Runtime consumption.

Step 16 remains `BLOCKED_UPSTREAM`: the published wire contract has no
connection-generation field. Keep the truthful `passed:false`/exit-65 smoke
evidence. Do not invent a body field, writer, adapter, or generated artifact.
Do not consume Runtime `79528044f758d188844270bc7e55decce2a7b0cc` or
R-00141.

## Verification And Handoff

Run focused RED/GREEN suites first, with repeated race and failure-injection
cases. Then run all affected Server suites and the complete serial Release
matrix from the final review, including PowerShell/Git Bash entry points,
build, policy, architecture, mirror/generated/isolation/portability/integration
gates, `git diff --check`, exact 14-path overlay audit, no untracked files,
HEAD, cached index SHA-256/path count, and script modes. External Windows link,
Root ABI compiler digest, upstream artifact, and POSIX signal gaps stay explicit
and do not waive local failures.

Write
`C:/Work/LumioGames/_codex-verification/server-a1-alpha-review-p1-fix-report.md`
with finding-by-finding RED/GREEN, exact changes, commands/exits/counts,
terminal/cleanup/retry/fault matrices, step-16 blocker, VCS/index identity, and
known gaps. Return `DONE`, `DONE_WITH_CONCERNS`, `NEEDS_CONTEXT`, or `BLOCKED`
via `worker_done`. Do not claim PASS or acceptance; the coordinator will freeze
a new complete union patch and start another fresh isolated review.
