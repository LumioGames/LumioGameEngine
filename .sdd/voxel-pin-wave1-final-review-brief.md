# Independent Voxel two-path final review brief

## Verdict

Review the complete two-path Voxel follow-up diff independently and read-only.
Return `PASS` only with zero P0/P1; otherwise return `RETURN`. Separately state
that the callable 11-method adapter/status/unknown-error mapping remains
`BLOCKED_UPSTREAM` unless accepted generated signatures/policy actually exist.

## Isolated package

- Clone: `C:\Work\LumioGames\_codex-verification\voxel-pin-wave1-final-review`
- Base/HEAD: `61cb864978dedfe9bdf7b687fea08660b31469f1`
- Patch: `C:\Work\LumioGames\_codex-verification\voxel-pin-wave1-final.patch`
- SHA-256: `8FAC5C8A68C88D779CD87A2B30D767868118346DD7F619CA0FF12CC6CA3AFCA0`
- Boundary: exactly two paths:
  - `crates/lumio-voxel-test-support/src/reference_harness.rs`
  - `crates/lumio-voxel-test-support/tests/reference_rust_differential.rs`

Do not edit candidate source/tests, stage, commit, push, or write Workflow.
Temporary probes belong outside the clone and must be removed or listed.

## Read first

1. Clone `AGENTS.md` and its three `.spec/` core documents.
2. `C:\Work\LumioGames\_codex-verification\voxel-pin-wave1-followup-report.md`
3. The full two-path patch before source sampling.
4. The published Architecture voxel-world-port schema and generated catalog
   evidence needed to distinguish local differential work from the upstream
   callable-contract blocker.

## Required review

- Prove the fixed 11-vector Reference model is genuinely independent of the
  real Rust implementation. Flag shared helper logic, copied transitions,
  expected values computed from actual output, self-comparison, or digest
  tautology.
- Independently recompute expected lifecycle, world/chunk revisions, canonical
  chunk presence, effects, known error IDs, observation ordering, and SHA-256
  traces for every vector. Validate duplicate/replay, cancellation, capture,
  durability ACK, and shutdown semantics against authoritative contracts.
- Prove the Rust leg executes a real `VoxelWorld` and real
  `GeneratedVoxelWorldPortAdapter`, seeds/publishes through legitimate paths,
  and does not bypass validation or replace missing operations with fake
  success.
- Check canonical sort/dedup, nondeterministic collections, endian/string
  framing, revision publication timing, stale handle/generation, error
  precedence, hash inputs, and cross-platform determinism.
- Judge whether the 873-line harness expansion is necessary and reviewable for
  11 vectors or contains avoidable feature/test-framework scope growth. Size
  alone is not a finding; identify concrete duplication/maintenance/correctness
  impact.
- Confirm no generated source/mirror, schema, ID, baseline, public contract,
  CI, shared manifest, or other module changed, and no generated bin/obj remains.
- Confirm no adapter signature, `status` shape, `quiesce`/`destroy` alias, or
  unknown-error fallback was invented. Verify all known published error IDs are
  mapped exhaustively without claiming the missing unknown policy.
- Reproduce focused differential, Voxel world/adapter tests, test-support
  all-features, workspace all-features, no-default-features check, fmt, clippy,
  crate DAG, generated-clean, architecture guards, managed generated dual-TFM
  builds, spec-lint/self-tests, `git diff --check`, reverse-check, exact scope,
  index, HEAD, LF, trailing whitespace, and final-newline checks.

## Output

Write:

`C:\Work\LumioGames\_codex-verification\voxel-pin-wave1-final-review-report.md`

Include package identity, commands/results, ordered findings, 11-vector
independence table, gate evidence, local verdict, upstream adapter verdict, and
known gaps. Return only verdicts, finding counts, one-line verification summary,
and report path.
