# Independent Client Session / Headless Final Review Brief

## Verdict

Review the nine-path Client session/headless candidate independently and
read-only. Return `PASS` only with zero local P0/P1; otherwise return `RETURN`
with the complete finding set. Passing managed tests do not waive stale payload
access, cancellation leaks, reentrancy, trace-contract, or ownership defects.

The remote generated `DeltaAck` writer/envelope surface remains
`BLOCKED_UPSTREAM`; do not treat that publication gap as a local implementation
failure and do not invent a replacement.

## Isolated Package

- Source repository: `C:\Work\LumioGames\LumioClient`
- Base/HEAD: `380ce29c862b7c90c9e09a9d1b6b0c9a6b7185b0`
- Patch:
  `C:\Work\LumioGames\_codex-verification\client-session-headless-final.patch`
- SHA-256:
  `B7F935C58444FD10569A96DA3AC3072655C730EF070686AB2E38CA81022319B8`
- Expected boundary: exactly nine paths listed in the implementation report,
  483 additions and 36 deletions.

Materialize the patch in an isolated clone or read-only snapshot. Do not edit
the live candidate, stage, commit, push, write Workflow, or change generated
contracts. Remove or list every disposable probe.

## Read First

1. Clone `AGENTS.md` and its three `.spec/` core documents.
2. `C:\Work\LumioGames\_codex-verification\client-session-headless-followup-report.md`
3. The full patch before source sampling.
4. The published Client session/headless task and generated API-map evidence
   needed to separate local behavior from the upstream `DeltaAck` blocker.

## Required Audit

- Prove every queued frame/event/close callback carries the generation captured
  at enqueue and is rejected before payload, classifier, callback, or mutable
  state access after reconnect. Probe a payload/classifier that throws or counts
  access and verify stale work cannot enter the new lifecycle.
- Reproduce cancellation while connect/tick/driver/close is blocked, cancellation
  before first tick, callback/driver failure, repeated cancel/dispose races, and
  reconnect during cancellation. Check one terminal close, no leaked task/timer/
  transport, deterministic exception precedence, and idempotent cleanup.
- Audit `HeadlessBotHost` ownership. Confirm it disposes only resources it
  actually owns, in a deterministic dependency-safe order, and preserves the
  existing minimal public interface/constructors.
- Validate observable traces for success, cancellation, reconnect, failure, and
  cleanup. Determine whether `ConnectRequested` must use the post-connect
  snapshot rather than the pre-connect sentinel, including nonstandard session
  implementations and failed connect.
- Check close orchestration for completion races, synchronous callback
  reentrancy, generation advance between scheduling and callback, repeated
  shutdown, and exception masking from `finally`.
- Check session event priority/arbitration did not starve valid handshake work,
  change close/disconnect precedence, or admit a stale frame through another
  queue path.
- Confirm no generated/schema/ID/baseline/replica/CI/public-contract or unrelated
  module changed, index is empty, and the exact nine-path package matches the
  stated hash and statistics.
- Reproduce focused and full bot/session tests, integration and architecture
  tests, solution build/test, repeated cancellation, contract mirror, generated/
  dependency smoke, available format/lint/toolchain gates, `git diff --check`,
  reverse-check, exact scope, LF/trailing-whitespace/final-newline, and HEAD/index
  checks. Record SDK `10.0.400`, archive, and symlink blockers exactly without
  converting them into green gates.

## Output

Write:

`C:\Work\LumioGames\_codex-verification\client-session-headless-final-review-report.md`

Include package identity, exact commands/results, ordered findings, adversarial
scenario table, local verdict, upstream `DeltaAck` verdict, external gate
gaps, and integration decision. Return only verdicts, finding counts, one-line
verification summary, and report path.
