# Independent Client chunk three-file final review brief

## Verdict

Review the complete three-file Client chunk candidate independently and
read-only. Return `PASS` only with zero P0/P1; otherwise `RETURN`. Ordinary
41/41 tests do not waive allocation, stale-generation, or batch-atomicity
failures.

## Isolated package

- Clone: `C:\Work\LumioGames\_codex-verification\client-chunk-wave1-final-review`
- Base/HEAD: `08ffa587c55d03da05a847b3858a860824b41e76`
- Task-entry Store reference: commit
  `3c8b87190bf00a4fd89b3225948e5b331bab4f62`
- Patch: `C:\Work\LumioGames\_codex-verification\client-chunk-wave1-final.patch`
- SHA-256: `B1BA765EEB1755E3084637DFD7A56A1F95D31FE6C296EE795150A3D4E9E39015`
- Boundary: exactly the State, Store, and StoreTests paths named in the patch.

Review only. Do not edit candidate files, stage, commit, push, or write
Workflow. Temporary probes belong outside the clone and must be removed/listed.

## Read first

1. Clone `AGENTS.md` and its three `.spec/` core documents.
2. `C:\Work\LumioGames\_codex-verification\client-chunk-wave1-final-fence-report.md`
3. The full patch before source sampling.
4. The three task-entry file versions, especially the seven-line Store delta at
   `3c8b871`, so review distinguishes preserved user work from this fix.

## Required audit

- Prove generation, token/request identity, requested revision, monotonic
  revision, `maxChunks`, per-update bytes, resident bytes, and checked-overflow
  fences all occur before payload/hash Span access, hash parsing/computation,
  `ToArray`, allocation, callback, queue/cursor change, or store mutation.
- Reproduce stale+over-budget precedence, stale request vs stale generation,
  malformed hash, throwing/tracking memory managers, oversized length without
  materialization, valid authoritative generation, and no-side-effect failures.
- Audit the single-update overload for allocations before fencing. Determine
  whether its one-element array violates the strict requirement and blast
  radius; do not dismiss allocation solely because payload bytes were not read.
- Audit batch atomicity: every member's metadata/budget must preflight before
  any member's expensive work or mutation; a later stale/invalid/throwing item
  must not hash/materialize/partially apply an earlier valid item. Probe commit
  exceptions and verify stable typed failure/fail-stop behavior.
- Check stored `ReadOnlyMemory` lifetime/immutability: caller mutation after
  construction must not change accepted hash/payload semantics or bypass the
  eventual validation. Check lazy validation does not create TOCTOU behavior.
- Check resident-byte/chunk accounting on replacement, duplicate, eviction,
  overflow, zero/capacity edges, and partial batch collapse.
- Check owner/thread/concurrency behavior and generation advance/reset against
  late queued work.
- Judge the 390+/42- three-file diff for concrete unnecessary redesign,
  duplicated logic, or lost existing behavior; size alone is not a finding.
- Reproduce focused/full Replica and solution tests/build, contract mirror,
  generated/toolchain/dependency filters, available format/lint/SDK/archive
  gates, `git diff --check`, reverse-check, exact three-file boundary, index,
  HEAD, LF, trailing whitespace, and final-newline checks. Report external SDK
  10.0.400/symlink/archive blockers exactly without waiving local P1s.

## Output

Write:

`C:\Work\LumioGames\_codex-verification\client-chunk-wave1-final-review-report.md`

Include package identity, commands/results, ordered findings, requirement and
task-entry reconciliation, local verdict, and known external gaps. Return only
verdict, finding counts, one-line verification summary, and report path.
