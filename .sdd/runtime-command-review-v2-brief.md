# Runtime Command/Coordination Follow-up Deep Review

## Objective

Independently review the complete uncommitted follow-up overlay for the rejected
Runtime Command/Coordination candidate based on
`1c53e2b7bc14d7f01a24a38ce6dc5d52448b2708`. Decide whether every P1/P2 item in
the original audit is actually fixed without introducing a new correctness,
contract, recovery, or test-quality defect.

This is review-only work. Do not edit source or tests, stage new changes, commit,
push, write Workflow, or consume the candidate for D-005.

## Required Materials

- Review worktree:
  `C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-command-final-review`
- Original independent rejection:
  `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\runtime-command-review-final-report.md`
- Implementer evidence:
  `C:\Work\LumioGames\_codex-verification\runtime-command-review-fix-evidence-task_d16030a47fe1.md`
- Complete review package (tracked and originally untracked files):
  `C:\Work\LumioGames\_codex-verification\runtime-command-review-v2.patch`
- Reviewer report output:
  `C:\Work\LumioGames\_codex-verification\runtime-command-review-v2-report.md`

Before review, read the target repository `AGENTS.md`, its three `.spec` core
documents, `before-you-code`, `reviewer.agent.md`, repository architecture,
testing, and the Command/Coordination architecture and module documents cited by
the original report.

## Mandatory Review Matrix

Perform a deep, adversarial review of the entire overlay, not a confirmation of
the implementer's summary. Cover every repository reviewer dimension and report
file/line evidence for each finding.

1. Standard test runner: independently run the repository-supported
   Microsoft.Testing.Platform `dotnet test` commands for both Command and
   Coordination from this clean snapshot. Verify `global.json` is the intended
   checked-in runner configuration and that a normal checkout does not require
   direct DLL execution or an undocumented workaround.
2. Revision consistency: trace normal commit, manual facade, participant apply,
   crash recovery, committed-marker recovery, and idempotent replay. Try to
   falsify the requirements that Voxel/ECS result vectors both exist, agree,
   advance the current session vector where required, are durably represented,
   and cannot publish `Committed` from stale, fabricated, partially applied, or
   mismatched evidence. Check failure atomicity and retry behavior, not only
   happy paths.
3. Voxel contract boundary: verify no hand-authored generated-looking Voxel
   contract remains publicly exported or usable as a competing public contract.
   If the architecture package lacks a callable generated projection, the
   production/default surface must remain explicitly fail closed.
4. Default composition: prove all public/default Command and Coordination entry
   points cannot report a successful commit through no-op ECS/Voxel
   participants. Check overloads and indirect construction paths.
5. Token identity: verify canonical encoding contains every equality/identity
   field, has unambiguous escaping/framing, and cannot collide across WorldId,
   BufferGeneration, or delimiter-containing values. Verify command canonical
   bytes use the corrected identity.
6. Durable fixtures: inspect all ten JSON artifacts and the replay test. Verify
   each positive/negative pair models duplicate, timeout, lost-result,
   partial-commit, and crash-boundary behavior rather than merely satisfying a
   self-defined parser. Confirm fixture discovery works from clean test output.
7. Regression and scope: inspect the complete diff for API compatibility,
   aliasing/mutability, arithmetic/monotonicity boundaries, exception mapping,
   concurrency/reentrancy, false-success paths, and out-of-scope changes. Re-run
   the most relevant Release builds/tests and `git diff --check`; reproduce the
   architecture validation evidence against the reviewed architecture candidate
   when feasible, otherwise state the exact limitation without converting it
   into a pass.

## Verdict Contract

Write the full report to the required output path and return only a concise
status. The report must include:

- Verdict: `PASS` or `RETURN`; any P0/P1 means `RETURN`.
- Coverage statement for all seven reviewer dimensions.
- Findings ordered P0, P1, P2 with exact file and line evidence plus a concrete
  failure scenario. Do not report speculative findings without evidence.
- Exact commands, exit codes, and key counts from fresh verification.
- Reconciliation of every original P1/P2 item.
- Known gaps and whether the candidate may be consumed for D-005.

Do not rely on the prior test counts or prior reviewer conclusion. Evidence must
come from this independent snapshot.
