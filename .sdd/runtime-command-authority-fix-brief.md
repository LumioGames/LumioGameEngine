# Runtime Command/Coordination authority-kernel fix brief

## Objective

Repair the current uncommitted Command/Coordination owner overlay in
`C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-command-wave1`.
The base commit is `1c53e2b7bc14d7f01a24a38ce6dc5d52448b2708` and is explicitly rejected for
D-005 consumption. The overlay also failed the later V3 review. Produce a new
uncommitted candidate; do not commit, push, stage, or write Workflow.

## Read first

1. Target repository `AGENTS.md` and its three `.spec/` core documents.
2. `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\runtime-command-review-final-report.md`
3. `C:\Work\LumioGames\_codex-verification\runtime-command-review-v3-report.md`
4. `C:\Work\LumioGames\_codex-verification\runtime-command-v3-targeted-revision-audit.md`
5. `C:\Work\LumioGames\_codex-verification\runtime-command-v3-architecture-triage.md`
6. `C:\Work\LumioGames\_codex-verification\runtime-command-dependency-feasibility.md`
7. `C:\Work\LumioGames\_codex-verification\runtime-txn-durability-contract-blueprint.md`
8. `C:\Work\LumioGames\_codex-verification\runtime-voxel-generated-port-blueprint.md`

## Scope

- `global.json`
- `modules/command/**`
- `modules/coordination/**`

Do not touch Replication, ECS, Simulation, GAS, Architecture source, Git
index/HEAD, or Workflow. Do not consume rejected implementation slices from
other modules. Do not spawn subagents.

## Required behavior

- Follow TDD: add focused failing adversarial tests before each production
  change and retain RED/GREEN evidence.
- Treat the V3 11 P1 findings plus the D-005 final report P1/P2 findings as one
  union. Fix every locally repairable item, not only the four original P1s.
- Implement the session-scoped transaction authority kernel described by the
  architecture triage. One authority must own state transitions, leases,
  durable-intent ordering, participant apply, revision evidence, marker
  publication, recovery, and retry/idempotency decisions.
- Fail closed when real participant ports are absent. Public/default
  composition must never report commit through no-op ECS or Voxel participants.
- Enforce full participant/result revision equality and one-time revision
  advancement. Marker-only and restart recovery must not fabricate or skip the
  committed revision.
- Remove any public Runtime duplication of the generated Voxel contract. A
  temporary private/internal seam may remain only when clearly non-public and
  fail-closed; do not invent a generated projection or claim the upstream
  contract exists.
- Preserve full deferred-token identity in canonical bytes and maintain
  durable failure fixtures as actual data artifacts.
- Make standard `dotnet test <project.csproj>` work through checked-in runner
  configuration and run both Command and Coordination tests that way.
- Add adversarial coverage for every V3 P1, the targeted revision/recovery
  findings, duplicate/retry paths, restart journal continuity, public API
  authority bypasses, stale/mismatched markers/evidence, lease fencing, and
  default composition.
- Do not represent Architecture-owned generated Voxel or portable process-loss
  journal contracts as closed. Report those as upstream blockers after local
  safety is complete.

## Verification and report

Run focused tests during implementation, then the two standard `dotnet test`
commands, test-project build, production dual-TFM builds, `git diff --check`,
and the target repository's applicable dependency/SDK/contract gates. Avoid
running builds in another agent's worktree.

Write the full handoff to
`C:\Work\LumioGames\_codex-verification\runtime-command-authority-fix-report.md`
with changed paths, RED/GREEN evidence, final command outputs, finding-by-finding
closure, and known upstream gaps. Return only `DONE`, `DONE_WITH_CONCERNS`,
`NEEDS_CONTEXT`, or `BLOCKED` plus a concise summary.
