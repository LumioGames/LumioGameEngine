# Server A1-alpha Final Review V2 Brief

Perform the one and only final independent review of the current Server
A1-alpha union candidate during convergence closeout. Return `PASS` only with
zero local P0/P1 findings and complete post-correction evidence. Otherwise
return `RETURN` and leave residual findings in backlog. A1 step 16 remains a
separate `BLOCKED_UPSTREAM` acceptance condition.

## Frozen Identity

- Repository: `C:/Work/LumioGames/LumioServer`
- Review worktree must start from clean base HEAD:
  `5ec95ee269207c64281b6e3f9176ed4f7ab5952c`
- Complete union patch (staged candidate plus current unstaged overlay):
  `C:/Work/LumioGames/_codex-verification/server-a1-alpha-final-v2.patch`
- Patch SHA-256:
  `B81DE9AA21ABD5BC65BE6D15417FCEB5114FDD431B4A819FED6ADB8AB5195AA3`
- Patch size: `581324` bytes; exact 63 paths; LF-only with final LF.
- Protected entry index patch:
  `C:/Work/LumioGames/_codex-verification/server-a1-alpha-entry-index.patch`
- Protected cached SHA-256/size:
  `A3373327146DE2C0066F4A8D838247F07A3757897B92E038ABA7D7880F18BC33`,
  `470352` bytes, exactly 60 staged paths.
- Implementer report:
  `C:/Work/LumioGames/_codex-verification/server-a1-alpha-review-p1-fix-report.md`
- Prior review:
  `C:/Work/LumioGames/_codex-verification/server-a1-alpha-final-review-report.md`
- Required report:
  `C:/Work/LumioGames/_codex-verification/server-a1-alpha-final-review-v2-report.md`

Before application verify patch hash/size, clean base HEAD, forward
`git apply --check`, exact 63 path/mode set, and after application
`git apply --reverse --check`. Review-only: no owner edits, commit, push,
stage, Workflow write, generated/schema/public-contract change, or Runtime
consumption.

## Scope And Provenance

The 63-path union intentionally includes the original 60 staged candidate and
its pre-existing overlay plus the current bounded Server changes. The current
bounded fix wave's newly owned paths are:

- `mvp-host/src/Lumio.Server.MvpHost.Transport/TransportService.cs`
- `mvp-host/src/Lumio.Server.MvpHost.Session/SessionRegistry.cs`
- `mvp-host/src/Lumio.Server.MvpHost.Transport.WebSocket/WebSocketByteCarrier.cs`
- `mvp-host/src/Lumio.Server.MvpHost.App/FullGraphComposition.cs`
- `mvp-host/src/Lumio.Server.MvpHost.Auth/MvpAuthorizationService.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/ConnectionLifecycleTest.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/TransportEventOrderingAndDiagnosticsTests.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/TransportHarness.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.Transport.Tests/BoundedQueueTest.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.Session.Tests/SessionBehaviorTests.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.Transport.WebSocket.Tests/CarrierBudgetAndFaultTests.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.App.Tests/ReadinessAndTraceTests.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.Auth.Tests/AuditAndErrorSemanticsTest.cs`
- `mvp-host/tests/Lumio.Server.MvpHost.WorldSlot.Tests/WorldSlotFocusedTests.cs`

Audit all 63 paths for accidental out-of-scope contract or generated changes,
but distinguish pre-existing union provenance from this wave. Step 16,
Runtime `79528044f758d188844270bc7e55decce2a7b0cc`, and R-00141 are hard holds.
The accepted wire contract still lacks `connectionGeneration`; do not invent a
field, writer, adapter, or expected-failure success.

## Required Adversarial Review

Re-run and independently inspect every final-review P1/P2 family:

1. `TransportService.Dispose` must close each carrier/resources, reserve and
   publish one terminal event before registry removal, preserve reason and
   stale-generation fencing, and converge under full/zero capacity,
   duplicate/concurrent dispose, throwing cleanup, and reserve exhaustion.
2. Normal and authenticated handshake ingress must own immutable bytes before
   any queue/callback retention; mutate/dispose producer buffers after enqueue
   and verify parsed identity/trace/auth bytes do not change.
3. All dependency callbacks, including same-thread `Bind` reentry, must be
   enqueue-only; synchronous Closed/Faulted during admission cannot be lost or
   leave a live Syncing binding.
4. Failed Unbind must retain ownership through Full/Closed/stale/throwing
   enqueue outcomes, retry only on owner lane with bounded diagnostic/dead-letter
   convergence, and never silently diverge transport/session ownership.
5. `SessionRegistry.Dispose` must release every committed slot reservation or
   preserve bounded durable retry/dead-letter evidence across shutdown, without
   double release or capacity leak.
6. WebSocket retirement must force-converge an already `CloseRequested` state:
   mark closed, cancel/join work, dispose socket/signals once, remove registry,
   and complete state. Exercise concurrent/repeated disposal and throwing timer
   or socket operations.
7. A `SessionLocalProven` adjudication must isolate the affected session and
   never call process-fatal handling when `SlotMustFailStop=false`; slot,
   process, and unproven critical faults remain fatal. Do not invent missing
   affected-session identity; classify an unexpressible route explicitly.
8. Throwing timer cancellation in `Touch` must become typed/diagnosed
   lifecycle behavior, not an escaping exception. Auth success reserve must be
   bounded to its declared budget with deterministic exhaustion behavior.
9. **Mandatory post-correction reconnect:** run
   `ReconnectScenarioCompletesAfterAnOutOfBandMutation` and its child smoke
   path after the final WebSocket cancellation correction. The implementer
   reported a prior exit `70` before correction and explicitly did not rerun;
   this is not green evidence. Repeat enough times to distinguish deterministic
   lifecycle failure from environment, and inspect close-frame/receive-token
   ordering.
10. Recheck authority revision, public shape/IVT, DeltaAck correlation,
    owner-lane execution, reservation cleanup, audit sequence, expiry/reconnect
    loser errors, signal/quiesce order, timer linearizability, credential
    zeroing, close-deadline anchoring, script modes, and generated/mirror
    isolation. POSIX signal skips on Windows and upstream contract gaps remain
    evidence qualifications, not waivers.

## Verification

Run all focused Server suites and complete serial Release build/test matrix,
including App, Auth, Session, Transport, WebSocket, WorldSlot, Platform,
Architecture, GeneratedContracts, Simulation.Reference, Wire, and Integration.
Run PowerShell/Bash verification entry points and dedicated gates where the
host supports them. Record exact exits/counts; wrappers that swallow failures
are not green. Run architecture contract validation read-only. Remove all
temporary probes before final checks.

Final review worktree must return to clean base HEAD, empty index, no untracked
files, exact applied 63-path patch before reverse cleanup, and `git diff
--check` clean. Confirm the owner worktree's cached 60-path index still hashes
to the protected value, but never alter it.

## Report Contract

Write verdict and P0/P1/P2 counts, package identity, union/new-wave scope,
deterministic findings, mandatory reconnect result, prior-finding
reconciliation, terminal/resource/fault matrix, exact command outputs, known
environmental gaps, step-16 blocker, and final VCS state to the required
report. Send one `worker_done` after cleanup. Do not claim A1 acceptance or
consume Runtime/D-005 candidates.
