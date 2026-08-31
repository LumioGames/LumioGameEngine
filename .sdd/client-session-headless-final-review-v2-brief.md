# Client Session / Headless Final Review V2 Brief

## Role

Perform a fresh independent adversarial review of the exact nine-path Client
Session/Headless candidate. This is review-only work. Do not edit the owner
worktree, commit, push, stage, update Workflow, or change generated/public
contracts. Apply/build/probe only in the isolated review worktree created for
this task.

## Frozen Identity

- Base and required review-worktree HEAD:
  `380ce29c862b7c90c9e09a9d1b6b0c9a6b7185b0`
- Canonical patch:
  `C:\Work\LumioGames\_codex-verification\client-session-headless-p1-fix.patch`
- Patch SHA-256:
  `AE43520247E70955DB99582AF3A2FA26E29E356619C8EB02C152A46A9D5B4EB7`
- Patch size: `99827` bytes
- Exact scope: nine paths, `2068` additions and `198` deletions
- Implementer report:
  `C:\Work\LumioGames\_codex-verification\client-session-headless-p1-fix-report.md`
- Prior review:
  `C:\Work\LumioGames\_codex-verification\client-session-headless-final-review-report.md`
- Output report:
  `C:\Work\LumioGames\_codex-verification\client-session-headless-final-review-v2-report.md`

Verify the patch hash, clean base/HEAD, exact nine paths, `git apply --check`,
and empty index before review. Apply only in this isolated worktree and confirm
reverse applicability after application. Treat the implementer report as
claims to challenge, not acceptance evidence.

## Separate Gates

- The prior P1-11 Connection terminal-event loss is owned by a disjoint patch
  and independent review. Do not assume it is fixed and do not edit or include
  `modules/connection/**` here. Assess this nine-path candidate against the
  base Connection behavior and state any resulting integration qualification.
- DeltaAck remains `BLOCKED_UPSTREAM`; no local writer/envelope/generated alias
  is authorized. Confirm this patch does not invent one.

## Verdict Contract

Return `PASS` only with zero local P0/P1. Otherwise return `RETURN` with every
finding's severity, exact locations, deterministic counterexample, and required
fix. Report P2 separately. Reconcile P1-01 through P1-10 and P2-01 one by one.

## Adversarial Review Lens

1. Public shape: reflection/source checks must show only the original public
   three- and four-argument `HeadlessBotHost` constructors. Internal/test
   composition must not widen IVT or expose a resource/trace API indirectly.
2. Stale frame ordering: generation/state/control-event arbitration must occur
   before mapper/priority/payload access. Exercise earlier/later stale frames,
   same-drain disconnect/fault, throwing memory/mapper, and mapper side effects.
3. Callback isolation: injected mapper, trace, hook, driver, ingress, scope,
   handshake, and connection callbacks may synchronously reenter public
   Session/Host operations. Prove terminal monotonicity, generation fencing,
   and no deadlock or resurrection under every callback position.
4. Connect failure: a false or throwing connection `Start()` must return the
   correct typed failure, freeze/fault consistently, release every created
   resource exactly once, and never publish successful Negotiating/Active.
5. Host close/cancellation: false, throw, cancellation before/during/after
   close, terminal completion races, driver/session exceptions, and repeated
   disposal must produce one legal close attempt and deterministic result/
   exception precedence. No callback may run after a terminal snapshot.
6. Trace authority: every entry must use the actual post-operation snapshot,
   actual generation, and actual executed tick. Connect/close failure and
   canceled/zero-tick paths must not emit synthetic or misleading success.
   A throwing trace sink must not alter lifecycle/cleanup semantics.
7. Cleanup atomicity: inspect synchronous completion of `ValueTask` releases,
   lock ownership, synchronization-context deadlock risk, partial completion,
   fault aggregation, false results, and continuation after each fault.
   Explicit owned/shared flags must cover connection, scope, handshake,
   replica, prediction, runtime, and any ledger entry without double-dispose.
8. Reentrant/single-flight close: nested direct and session-level close,
   callbacks during release, concurrent close/fault/cancel, and disposal must
   preserve exactly-once ledger and connection actions without hiding failure.
9. Event inbox and mapper caching must not admit fabricated future-generation
   tokens, stale priorities, duplicate payload access, unbounded growth, or
   cross-tick ordering changes.
10. Test quality: confirm every new regression would fail on the returned V1
    patch, asserts observable behavior rather than private implementation, and
    uses deterministic synchronization instead of timing-sensitive sleeps.

## Verification

Run serially in the isolated worktree at minimum:

- focused bot/session regression classes and full bot/session projects;
- repeated cancellation, mapper, cleanup, and close-reentrancy races;
- Integration and Architecture projects, full Release solution build, and all
  discovered runnable test assemblies;
- contract mirror, upstream smoke, generated/dependency/contract filters;
- public API reflection checks, exact nine-path boundary, LF/text hygiene,
  `git diff --check`, patch reverse check, HEAD/index/untracked status.

Keep the unavailable pinned SDK `10.0.400`, archive, Windows link/spec-lint,
Bash, cross-repository, real DS, and Unity/device gaps explicit. Installed SDK
`10.0.111` evidence does not waive the pin; external gaps do not waive a local
P0/P1.

## Handoff

Write the full report to the output path. Send exactly one `worker_done` with
verdict, P0/P1/P2 counts, prior-finding reconciliation, separate Connection and
DeltaAck qualifications, report path, and confirmation that no candidate source
was edited outside the isolated review worktree.
