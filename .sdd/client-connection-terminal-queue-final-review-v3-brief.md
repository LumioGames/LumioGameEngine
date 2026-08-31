# Client Connection Terminal Queue Final Review V3 Brief

Perform the one authorized final independent review of the current seven-path
Connection terminal-queue candidate during convergence closeout. Return `PASS`
only with zero P0/P1 findings; otherwise return `RETURN` and leave residual
findings in backlog. Do not create another fixer after this review.

## Frozen identity

- Repository: `C:/Work/LumioGames/LumioClient`
- Required base HEAD: `380ce29c862b7c90c9e09a9d1b6b0c9a6b7185b0`
- Frozen complete patch:
  `C:/Work/LumioGames/_codex-verification/client-connection-terminal-queue-final-v3.patch`
- SHA-256: `3E1698C25105E6F4E146A4435BE169A3D359D8763EB1A298DD2C71C266D68715`
- Size: `95763` bytes; LF-only with final LF.
- Implementer report:
  `C:/Work/LumioGames/_codex-verification/client-connection-terminal-queue-v2-p1-fix-report.md`
- Prior review authority:
  `C:/Work/LumioGames/_codex-verification/client-connection-terminal-queue-final-review-v2-report.md`
- Required report:
  `C:/Work/LumioGames/_codex-verification/client-connection-terminal-queue-final-review-v3-report.md`

The patch contains exactly seven paths, including the new mode-100644
`modules/connection/tests/Fault/ConnectionTerminalReservationTests.cs`. Verify
clean base, empty index/untracked state, hash/size, forward apply, exact path
and mode set, and reverse applicability after application. Review only in a
new isolated worktree; do not edit owner, commit, push, stage, or write
Workflow.

## Exact scope

1. `modules/connection/src/Internal/Faults/DeterministicDelayQueue.cs`
2. `modules/connection/src/Internal/Queues/ConnectionEventQueue.cs`
3. `modules/connection/src/Internal/State/ConnectionStateMachine.cs`
4. `modules/connection/src/Internal/Transport/WebSocket/WebSocketClientConnection.cs`
5. `modules/connection/src/Public/IClientConnectionFactory.cs`
6. `modules/connection/tests/Transport/LoopbackWebSocketServer.cs`
7. `modules/connection/tests/Fault/ConnectionTerminalReservationTests.cs`

No Session/Headless, Replica, generated/schema/ID, public contract shape,
project/dependency/toolchain/archive, or Architecture file may change. DeltaAck
and any broader public protocol remain `BLOCKED_UPSTREAM`.

## Required adversarial review

Independently inspect and effect-test the two prior P1 families and all stated
preservation claims:

1. **Send-stage terminal fence:** deterministically dequeue/select a valid large
   frame, win close/fault/disconnect/dispose before actual `SendAsync`, and
   verify no application bytes arrive after the terminal event. Cover first
   send, duplicate send, delayed-release send, full-queue retry, zero-capacity
   Start followed by explicit close, and repeated mixed close races. Verify a
   generation/terminal/disposal/send reservation is revalidated immediately
   before every actual send and that in-flight frames have an explicit,
   one-shot retain/discard outcome with no capacity leak.
2. **Callback-safe disposal:** invoke direct and `Task.Run`/cross-thread
   `Dispose` synchronously from `ITransportFaultPolicy.Decide`; assert bounded
   completion, no sender/callback wait cycle, no disposed semaphore/socket while
   workers can touch it, and exactly-once eventual cleanup. Exercise active
   decision, active send/pump, no-worker immediate Start/Dispose, repeated and
   concurrent disposal, and throwing cancel/dispose implementations. Thread-id
   heuristics must not be the sole safety mechanism.
3. **Terminal lane and FIFO:** verify one-shot terminal precedence and reserved
   delivery at capacities 0/1/2, destination-size and empty-drain behavior,
   ordinary-full and terminal-full diagnostics, Disconnect action, rejected
   generation, and no post-terminal Local/WebSocket bytes.
4. **Delay semantics:** repeat Delay/full/terminal-fence sequences and verify
   deterministic FIFO, reservation release, no duplicate delayed sends, and no
   epoch invalidation from tail enqueue.
5. Audit for new P0/P1 issues: races between terminal publication and send
   completion, lost in-flight head, cancellation swallowing real faults,
   semaphore/task disposal races, reservation accounting overflow, stale epoch
   acceptance, duplicate terminal events, unbounded queue growth, and tests
   that are tautological or depend on forbidden production test hooks.

## Verification

Run focused `ConnectionTerminalReservationTests` and at least five serial
repetitions of all new race cases, full Connection tests, full solution Release
build/tests, Architecture generated/dependency/contract filters, contract
mirror and upstream smoke, public API reflection, `git diff --check`, exact
seven-path/hash/mode/untracked checks, and any available SDK/toolchain gates.
Run only serially where shared .NET artifacts require it. Record pinned SDK,
Bash, Windows-link, archive, and compiler-digest gaps separately; wrappers
that swallow failures are not green. Remove temporary probes before final
identity checks.

## Report contract

Write verdict/counts, exact package identity, scope, deterministic probe output,
prior-finding reconciliation, send/terminal/delay/disposal matrices, command
outputs, known gaps, and final clean review VCS state to the required report.
Send one `worker_done` after cleanup. Do not claim product acceptance. Under
the convergence hold any P0/P1 is backlog with no further fixer wave.
