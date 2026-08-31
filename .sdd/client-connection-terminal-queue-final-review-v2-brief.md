# Client Connection Terminal Queue Final Review V2 Brief

## Role

Perform a fresh independent adversarial review of the complete seven-path
Connection candidate. This is review-only work: do not edit the owner candidate,
commit, push, stage, update Workflow, or change public/generated contracts.
Apply/build/probe only in the isolated review worktree created for this task.

## Frozen Identity

- Base and required review-worktree HEAD:
  `380ce29c862b7c90c9e09a9d1b6b0c9a6b7185b0`
- Canonical V2 patch:
  `C:\Work\LumioGames\_codex-verification\client-connection-terminal-queue-final-v2.patch`
- SHA-256:
  `C73CF0DC0D621394C28E5D3C87D6069E8C7E4CDB73B776FFD5A7C6E090029869`
- Patch size: `66090` bytes
- Exact scope: seven paths, `1289` additions and `82` deletions; the complete
  untracked `ConnectionTerminalReservationTests.cs` is included.
- Implementer report:
  `C:\Work\LumioGames\_codex-verification\client-connection-terminal-queue-review-fix-report.md`
- Prior review:
  `C:\Work\LumioGames\_codex-verification\client-connection-terminal-queue-final-review-report.md`
- Output report:
  `C:\Work\LumioGames\_codex-verification\client-connection-terminal-queue-final-review-v2-report.md`

Verify the patch hash, clean exact base, seven paths/modes, apply check, empty
index, and untracked test inclusion before substantive review. Apply only in
the isolated worktree and confirm reverse applicability afterwards.

## Verdict Contract

Return `PASS` only with zero P0/P1. Otherwise return `RETURN` with severity,
exact locations, deterministic counterexamples, and required fixes. Report P2
separately. Reconcile the original full-queue terminal-loss P1, all three V1
review P1s, and both V1 P2s one by one.

## Adversarial Review Lens

1. Terminal lane: ordinary-full/zero/one capacity, destination sizes 0/1/small,
   all close reasons, duplicate/mixed races, late generation, inbound after
   terminal, drain/dispose/reentry must retain exactly one bounded immutable-
   generation terminal without overwriting ordinary FIFO.
2. Capacity-zero Start: Local and syntactically valid/rejected WebSocket paths
   must never return successful Start without exactly one observable Started.
   If Start fails, `_started` remains false, no pump/dial/send begins, repeated
   Start remains deterministic, and later explicit close/fault still publishes
   one terminal through the reserved lane.
3. Policy reentrancy: `ITransportFaultPolicy.Decide` is arbitrary synchronous
   public callback code. Verify it runs outside lifecycle locks, cannot deadlock
   through snapshot/close/dispose/send reentry, and every generation/terminal/
   disposal/send-epoch token is revalidated before dequeue, send, delay move,
   or success return. Probe multiple concurrent senders and callbacks changing
   head/tail/epoch while a decision is in flight.
4. Post-terminal send: Local and WebSocket must emit no selected or new bytes
   after any close/fault/disconnect/dispose wins. The selected head must be
   retained/discarded consistently without capacity leak or tail overtaking.
5. `TransportFaultAction.Disconnect`: non-full/full/delayed egress, Local and
   WebSocket, duplicate/mixed close races and callback reentry must atomically
   publish one `Disconnected` terminal before any target send while preserving
   an earlier winning reason.
6. Delay: prove the existing deterministic queue is actually bounded by the
   configured combined egress capacity; held frames release exactly once in
   FIFO order, no tail overtakes, transport-full retry is lossless, repeated
   Delay cannot grow memory, and close/fault/dispose clears or fences held
   frames with no post-terminal release. Verify policy invocation and release
   triggers do not spin or starve.
7. Lock/epoch ordering: audit ConnectionStateMachine, event/send/delay queues,
   Local factory and WebSocket sender for inversion, stale epoch acceptance,
   Count/Snapshot races, missed wakeups, and false success on callback/queue
   failure. Run deterministic high-contention probes.
8. Public/test shape: no public type/member/enum/protocol/dependency change;
   the WebSocket handshake gate is test-only and cannot alter ordinary server
   behavior. Ensure tests assert external effects and would fail on the V1
   returned patch rather than mirror private implementation.
9. Boundary: exactly seven Connection paths, no Session/Headless/Replica/
   generated/project/dependency drift, no missing untracked bytes, no retained
   probes/build artifacts, clean text and mode metadata.

## Verification

Run serially at minimum:

- focused terminal reservation/effect suite and at least five repetitions;
- full Connection tests and production Release build;
- additional concurrent policy/send/close/Delay/terminal probes for Local and
  valid WebSocket paths;
- full solution Release build/test, Architecture tests and generated/
  dependency/upstream filters;
- contract mirror and upstream smoke;
- exact scope/text/hash checks, `git diff --check`, patch reverse check,
  HEAD/index/untracked status.

Keep unavailable pinned SDK `10.0.400`, archive, Windows link/spec-lint, Bash,
cross-repository, DS/device gaps explicit. Installed `10.0.111` evidence does
not waive the pin; external gaps do not waive local P0/P1.

## Handoff

Write the full report to the output path. Send exactly one `worker_done` with
verdict, counts, complete prior-finding reconciliation, report path, and
confirmation that no candidate source was edited outside the isolated review
worktree.
