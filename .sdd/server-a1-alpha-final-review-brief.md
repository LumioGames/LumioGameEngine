# Server A1-alpha Final Independent Review Brief

## Role

Perform a fresh independent adversarial deep review of the complete Server
A1-alpha candidate. This is review-only work. Do not edit the owner worktree,
commit, push, stage, update Workflow, change public/generated contracts, or
consume frozen Runtime/D-005 candidates. Apply/build/probe only in the isolated
review worktree created for this task.

## Frozen Identity

- Base and required review-worktree HEAD:
  `5ec95ee269207c64281b6e3f9176ed4f7ab5952c`
- Complete final patch (the union of the original 60-path staged candidate and
  the 18-path working-tree overlay):
  `C:\Work\LumioGames\_codex-verification\server-a1-alpha-final.patch`
- Complete patch SHA-256:
  `4DBED0CC5D2A40528E0AAE3CC8EBC9FAB8A3942F4651DEA744DF59AC762D60E2`
- Patch size: `523865` bytes
- Union scope: `61` paths, `8419` additions and `800` deletions
- Frozen entry index patch:
  `C:\Work\LumioGames\_codex-verification\server-a1-alpha-entry-index.patch`
- Entry/current cached full-index SHA-256 and size:
  `A3373327146DE2C0066F4A8D838247F07A3757897B92E038ABA7D7880F18BC33`,
  `470352` bytes; coordinator independently confirmed exact equality.
- Implementer report:
  `C:\Work\LumioGames\_codex-verification\server-a1-alpha-final-fix-report.md`
- Finding authorities:
  `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\server-a1-alpha-review-findings.md`
  and
  `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\server-a1-alpha-final-return.md`
- Output report:
  `C:\Work\LumioGames\_codex-verification\server-a1-alpha-final-review-report.md`

The owner worktree intentionally retains its original staged index. The review
identity is the complete `git diff HEAD` patch above, not the cached patch alone
and not the 18-path unstaged overlay alone. Verify hash, clean base/HEAD, exact
61 paths/modes, `git apply --check`, and empty review index; apply only in the
isolated worktree and confirm reverse applicability.

## Verdict Contract

Give two explicit verdicts:

1. **Local candidate verdict:** `PASS` only with zero local P0/P1; otherwise
   `RETURN` with all findings, exact locations, deterministic counterexamples,
   and required fixes. Report P2 separately.
2. **A1-alpha acceptance verdict:** remains `BLOCKED_UPSTREAM` while step 16
   cannot be expressed by the accepted generated wire contract, even if the
   local candidate passes. Do not convert this into a local expected-failure
   success and do not invent a connection-generation field/writer.

Reconcile every prior finding and each item in the final return intake. Treat
the implementer test matrix as claims to challenge.

## Adversarial Review Lens

1. Authority revision: prove Session never manufactures WorldSlot/Simulation
   revisions, reconnect/enqueue failure cannot advance them, lower source
   revision is surfaced/fail-stopped, and `observedRevision` advances only on
   accepted owner evidence. Audit the `#if !MVP_HOST_FULL_GRAPH`
   `HostProtocolServer` path that still increments a local revision: determine
   whether any supported/default production composition can execute it and
   whether its provisional status is an actual P1 rather than a report caveat.
2. Public/friend surface: reflect exact six public Session commands and seven
   WorldSlot commands, no public `ReleaseAdmission`/`ConnectionTerminated`, no
   Session/WorldSlot proof-constructor IVT, and carrier-only immutable one-shot
   authentication evidence keyed to connection epoch.
3. Transport lifecycle: event overflow and service Dispose must close carrier,
   cancel timers, clear queues/metadata, reserve exactly one bounded terminal
   event before registry removal, reject stale generation afterwards, and
   fail-stop on reserve exhaustion. Probe throwing close/cancel and concurrent
   dispose/overflow/reentry without leaks or duplicate terminal events.
4. Delta ACK: bind confirmation sequence, base/from/to revisions, snapshot,
   generation/session identity, successful enqueue, exact replay, forged jump,
   changed duplicate, reconnect, and bounded pending retention. No ACK may skip
   unsent revisions or survive a generation reset incorrectly.
5. Owner lane: all transport/admin/auth/authority callbacks must enqueue
   immutable ingress only; no caller can run reducers or retries. Probe
   reentrant callbacks, wrong thread, disposal, and callback failure under the
   owner gate for deadlocks and off-lane mutation.
6. Reservation cleanup: queue-full/stale-epoch/throwing release must preserve
   ownership, bounded retry/dead-letter evidence, fail-stop on dead-letter
   exhaustion, and exactly-once success. Terminal session removal must not lose
   committed capacity.
7. Audit ordering: parallel and reentrant admin calls must receive unique,
   monotonic, authority-owned sequence IDs with no out-of-order publication.
8. Expiry/reconnect race: run truly simultaneous producers repeatedly; exactly
   one transition wins and the loser receives the schema-owned stable error.
   No connection may be closed without a client-visible registered error.
9. Signal/quiesce: inspect and, where host capability permits, exercise
   SIGTERM, SIGINT, console cancel, repeated signals, and signal during each
   stage. Required order is `AdmissionClosed -> Drained -> SnapshotCut ->
   Stopped`, with no early process return. Windows skips are evidence gaps, not
   proof; classify whether source/probe evidence is enough for local PASS.
10. Timer linearizability: Schedule/claim/cancel/Dispose races must never return
    an ID for an unfireable timer or execute after disposal; cover throwing
    callbacks and repeated disposal.
11. Secret/resource ownership: credential bytes must be zeroed on every early
    return/exception including MaxConnections; connection CTS/semaphore,
    sockets, timers, and carriers must be disposed exactly once; first-close
    deadline remains anchored through continuous backpressure.
12. Step 16 evidence: independently inspect Architecture/schema/generated
    package and smoke behavior. Confirm `passed:false`/exit 65 is truthful
    blocker evidence, not a green acceptance item, and that no private wire
    substitute exists in the 61 paths.
13. Gate/policy integrity: verify shell executable modes (`100755`) and
    PowerShell modes (`100644`), complete dependency/index inclusion, Bash and
    PowerShell gate parity, generated/mirror isolation, and no hidden
    Runtime `7952804` or R-00141 consumption.
14. Test quality: confirm new race/process/reflection/resource tests would fail
    on the returned candidate, assert observable behavior, are deterministic,
    and do not turn a known failure into an expected green result.

## Verification

Run serially in the isolated worktree at minimum:

- all focused Session, Transport, WebSocket, WorldSlot, Platform, App,
  Architecture, Auth, GeneratedContracts, Simulation.Reference, Wire, and
  Integration Release suites;
- repeated owner-lane, expiry/reconnect, DeltaAck, cleanup/dead-letter,
  overflow/dispose, timer, signal/quiesce, and credential/resource probes;
- `mvp-host/eng/verify-all.ps1` and Git Bash `verify-all.sh`, plus every
  dedicated PowerShell/Bash gate;
- Release `build.proj`, Server policy check, spec-lint/self-tests, mirror and
  generated checks;
- exact 61-path/mode/text scan, `git diff --check`, patch reverse check,
  HEAD/index/untracked status and no retained probe artifacts.

Run Architecture/contract gates read-only and record exact upstream failures.
Do not rewrite generated/schema artifacts to make them pass. Keep Windows POSIX
signal capability, compiler/artifact drift, and other external gaps explicit;
external gaps do not waive local P0/P1.

## Handoff

Write the full report to the output path. Send exactly one `worker_done` with
local verdict, P0/P1/P2 counts, A1-alpha upstream verdict, step-16 decision,
fallback/signal qualification, prior-finding reconciliation, report path, and
confirmation that no candidate source was edited outside the isolated review
worktree.
