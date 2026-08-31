# Client Connection Terminal Queue Final Review Brief

## Role

Perform a fresh independent adversarial review. This is review-only work: do
not edit the owner candidate, commit, push, stage, update Workflow, or change
public/generated contracts. Build and probe only in the isolated review
worktree created for this task.

## Frozen Identity

- Base and required review-worktree HEAD:
  `380ce29c862b7c90c9e09a9d1b6b0c9a6b7185b0`
- Patch:
  `C:\Work\LumioGames\_codex-verification\client-connection-terminal-queue-final.patch`
- Patch SHA-256:
  `03371E2ACC3B3273FF6A097B3697C5C5EF241286542C90C3D45F9EA53E1A8F68`
- Patch size: `22771` bytes
- Exact scope: five paths, `385` additions and `46` deletions; the new
  `ConnectionTerminalReservationTests.cs` bytes are included in the patch.
- Implementer report:
  `C:\Work\LumioGames\_codex-verification\client-connection-terminal-queue-p1-fix-report.md`
- Prior finding authority: P1-11 in
  `C:\Work\LumioGames\_codex-verification\client-session-headless-final-review-report.md`
- Output report:
  `C:\Work\LumioGames\_codex-verification\client-connection-terminal-queue-final-review-report.md`

Before reviewing, verify the patch hash, clean base/HEAD, exact five-path
scope, and `git apply --check`; apply the patch only in this isolated review
worktree. Confirm reverse applicability after application. Treat implementer
test claims as inputs to verify, not acceptance evidence.

## Required Verdict

Return `PASS` only with zero P0/P1. Otherwise return `RETURN` and enumerate all
findings with severity, file/line evidence, counterexample, and required fix.
Report P2 findings separately. The review must state both local verdict and
whether the original full-queue terminal-loss P1 is closed.

## Adversarial Review Lens

1. Reproduce the original counterexample: a full ordinary event queue followed
   by Disconnect/Fault/Closed must never expose terminal success without an
   eventually drainable, immutable-generation terminal event.
2. Prove the terminal lane is bounded and one-shot under direct queue use,
   state-machine use, public factory use, WebSocket callbacks, and concurrent
   close races. A duplicate terminal must not replace the winner or reopen the
   queue.
3. Verify ordinary validated frames are never overwritten, FIFO ordering is
   retained, and repeated drains with destination sizes 0, 1, and smaller than
   the pending ordinary count eventually yield exactly one terminal event.
4. Scrutinize capacity 0 and 1. Confirm changing factory/WebSocket clamping from
   1 to 0 does not create an unreported Start success, invalid send-queue state,
   constructor inconsistency, or transport divergence.
5. Audit state-machine and queue locks for inversion, nested-lock deadlock,
   reentrancy, race windows in Count/Snapshot/Drain/close/inbound delivery, and
   publication ordering. Terminal state may become visible only after the
   reserved event is guaranteed.
6. Verify late generation, post-terminal inbound data, duplicate close, fault
   vs disconnect races, empty drains, disposal, and WebSocket rejection paths
   preserve the winning reason and immutable generation.
7. Confirm the two touched public-path files add or remove no public type/member
   or protocol surface and preserve established constructor/factory behavior.
8. Inspect test quality: deterministic assertions, real failure before the
   production fix, no tautological access to private implementation state, and
   enough repetitions to expose concurrency defects without flaky timing.
9. Confirm exact boundary, no generated/session/headless/replica/dependency
   changes, clean index, LF/text hygiene, and no temporary probe/build artifacts
   in the package.

## Verification

At minimum rerun serially in the isolated worktree:

- focused `ConnectionTerminalReservationTests`;
- the full Connection test project and Release production build;
- repeated close/full-queue probes for all three close reasons and capacities
  0/1/full;
- full solution Release build/test where the installed SDK permits;
- contract mirror, upstream smoke, generated/dependency/architecture filters;
- `git diff --check`, exact path/mode/text scans, patch reverse check, and final
  HEAD/index status.

Keep the unavailable pinned SDK `10.0.400`, archive, Windows link, Bash, and
cross-repository gates explicit. Installed SDK `10.0.111` evidence does not
waive those gaps, but external gaps also do not waive a local P0/P1.

## Handoff

Write the full report to the required output path. Send one `worker_done` with
the local verdict, finding counts, original-P1 closure decision, report path,
and confirmation that no candidate or review-source file was retained outside
the isolated review worktree.
