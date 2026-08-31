# Client Session / Headless Final Review V3 Brief

Perform one fresh independent adversarial review of the exact nine-path
Session/Headless V2 fixer output. This is the only final review permitted for
this candidate during convergence closeout. Return `PASS` only with zero P0 and
zero P1; otherwise return `RETURN` and leave residual findings in backlog.

## Frozen identity

- Repository: `C:/Work/LumioGames/LumioClient`
- Required base HEAD: `380ce29c862b7c90c9e09a9d1b6b0c9a6b7185b0`
- Frozen patch:
  `C:/Work/LumioGames/_codex-verification/client-session-headless-final-v3.patch`
- SHA-256: `1D01174EEC4E62A4E3D98917E6103AC3AE2647FF34D8C98D185749786ABD13DC`
- Size: `173134` bytes; exact nine-path diff, LF-only, final LF.
- Implementer report:
  `C:/Work/LumioGames/_codex-verification/client-session-headless-v2-p1-fix-report.md`
- Prior independent review:
  `C:/Work/LumioGames/_codex-verification/client-session-headless-final-review-v2-report.md`
- Required report:
  `C:/Work/LumioGames/_codex-verification/client-session-headless-final-review-v3-report.md`

Verify clean base HEAD, empty index/untracked state, exact patch hash/size,
`git apply --check`, exact modes/paths, and reverse applicability after
application. Review only in a new isolated worktree; never edit the owner,
commit, push, stage, or write Workflow.

## Exact scope

Only these paths are allowed:

1. `modules/bot/src/Internal/HeadlessBotHost.cs`
2. `modules/bot/tests/Unit/BotCancellationRaceTests.cs`
3. `modules/session/src/Internal/ClientSession.cs`
4. `modules/session/src/Internal/Events/SessionEventInbox.cs`
5. `modules/session/src/Internal/Orchestration/CloseOrchestrator.cs`
6. `modules/session/tests/Fault/SessionRaceTests.cs`
7. `modules/session/tests/Support/SessionHarness.cs`
8. `modules/session/tests/Unit/CloseOrchestratorCallTests.cs`
9. `modules/session/tests/Unit/SessionEventArbiterTests.cs`

Connection/DeltaAck, HandshakeOrchestrator, Replica, generated/schema/ID,
public protocol, dependencies, toolchain, archive, and Architecture files are
outside scope. DeltaAck and Connection P1-11 remain `BLOCKED_UPSTREAM`.

## Review requirements

Independently reproduce and audit all prior V2 P1/P2 claims:

- same-drain Hello + FullSnapshot dispatch, control arbitration, stale
  generation and zero-payload fences;
- actual handshake Begin false/throw and ownership isolation from pre-existing
  Active/Negotiating work;
- authoritative pre/post callback fences around every host hook, driver,
  ingress, tick, scope, replica, prediction, runtime, presentation, local
  input, and connection-send callback, with and without trace;
- close trace truth, zero-tick encoding, already-terminal behavior, immutable
  trace reentry, and terminal generation revalidation;
- prepared-versus-activated scope ownership, activation false/throw rollback,
  incomplete `ValueTask` release under a captured SynchronizationContext, and
  no early terminal publication;
- transitive epoch cancellation and rollback across every staged authority;
- primary plus secondary cleanup failures, owned/shared connection/scope
  disposal, concurrent/reentrant close single-flight, and identical completion
  results;
- race-free PendingScope completion and all prior deterministic matrices.

### Additional mandatory static/effect check

Use an arbitrary third-party `IClientSession` implementation through the public
three-argument HeadlessBotHost constructor with `NullTickHook` and
`NullTraceSink`. It cannot expose the private/internal
`BeginHostFinalization` capability. Prove that strict lifecycle fences do not
silently disable in this path, that the host does not fabricate
`Connecting/generation 1`, invoke driver/ingress/tick after terminal or
generation changes, or treat a failed `TryGetSnapshot` as an authoritative
stale `lastSnapshot`. If this cannot be closed legally in the nine paths,
return a P1 finding; do not invent a public capability.

Inspect public constructors and reflection shape, resource ownership, exception
precedence, cancellation ordering, callback reentry from arbitrary threads,
and tests for tautological assertions or implementation-only visibility.

## Verification

Run focused Session/Headless tests repeatedly, full solution Release build and
all runnable tests, architecture/generated/dependency/contract filters,
contract mirror/upstream smoke, public API reflection, `git diff --check`,
exact nine-path/HEAD/index/untracked/mode/text checks, and at least five repeats
of close, callback, synchronization-context, and third-party-session probes.
Use installed SDK evidence only when the pinned SDK is unavailable and record
that limitation. Remove all temporary probes before final checks. Distinguish
known SDK, archive, Windows-link, Bash, and cross-repo compiler-digest gaps
from local findings.

## Report contract

Write verdict/counts, package identity, exact scope, deterministic findings,
prior reconciliation, callback/cleanup/trace/ownership matrix, actual command
outputs, external gaps, and final VCS state to the required report. Send one
`worker_done` only after cleanup and report completion. Do not claim product
acceptance; residual P0/P1 is backlog under the convergence hold.
