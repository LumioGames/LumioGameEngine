# Voxel Two-Path Differential Final Review V2 Brief

## Objective And Gate

Perform a fresh, independent, adversarial review of the complete two-path
Voxel local differential candidate. The implementer report and passing tests
are evidence to verify, not proof. The local candidate may receive `PASS` only
with zero P0 and zero P1 findings. Otherwise return `RETURN` with reproducible
findings.

The generated callable adapter is a separate upstream gate. Architecture V1.4
publishes the 11 method names but no callable signatures, public status shape,
or unknown-error disposition. Keep all 11 callable rows
`BLOCKED_UPSTREAM`. Do not invent or modify a public/generated contract and do
not treat internal routes as callable delivery.

## Frozen Identity

- Repository: `C:/Work/LumioGames/LumioVoxelEngine`
- Required clean base HEAD:
  `61cb864978dedfe9bdf7b687fea08660b31469f1`
- Frozen patch:
  `C:/Work/LumioGames/_codex-verification/voxel-pin-wave1-final-v2.patch`
- SHA-256:
  `30424CFFFE9F5A84B8B48C19A2E61BFEC11BCBAB56ACA1E28F359A8CBDB39F2B`
- Patch size: `194104` bytes
- Patch shape: `5574` additions and `1` deletion, LF-only with final LF.
- Implementer report:
  `C:/Work/LumioGames/_codex-verification/voxel-pin-wave1-local-review-fix-report.md`
- Prior independent review:
  `C:/Work/LumioGames/_codex-verification/voxel-pin-wave1-final-review-report.md`
- Required review report:
  `C:/Work/LumioGames/_codex-verification/voxel-pin-wave1-final-review-v2-report.md`

Before reviewing, verify clean HEAD/index/untracked state, exact patch hash and
size, `git apply --check`, exact paths and modes, and after application
`git apply --reverse --check`. Apply only in the isolated review worktree.

## Exact Boundary

Only these paths may be present in the candidate:

1. `crates/lumio-voxel-test-support/src/reference_harness.rs`
2. `crates/lumio-voxel-test-support/tests/reference_rust_differential.rs`

The second path is a new mode-100644 file and its bytes are included in the
patch. No generated source, schema, ID, fixture, manifest, dependency,
Architecture mirror, Workflow record, public API, or production Voxel source
may change. No commit, push, stage, acceptance, or contract invention.

## Required Adversarial Review

Reconcile every prior P1 and P2 against source and independent effects.

1. Oracle independence: prove that the reference leg does not call, copy,
   import, or mechanically transcribe the SUT transition implementation or its
   result helpers. Identify shared production helpers used for fingerprints,
   roots, receipts, ACKs, parsing, or framing and decide whether each destroys
   the claimed independence. A second transcription that can share the same
   bug is a P1.
2. Fixed golden evidence: independently derive the declared 11-row expected
   trace and golden digest from authoritative, recorded input/expected bytes,
   without calling either candidate digest function or copying SUT output.
   Record the exact bytes/framing and recomputed digest. Perform a
   mutation-negative by changing at least one expected identity, root, stamp,
   receipt, operation, or sequence field and prove the comparison/golden check
   fails. A golden updated from current SUT output is not independent evidence.
3. Complete observation projection: compare world/context identity,
   generation and stamp generation, internal lifecycle labels with explicit
   provenance, world and per-chunk revisions, exact signed-coordinate chunk
   set, page payload/digest evidence, published root, directory and dirty
   digests, dirty entries, config/provenance, publication epoch, capture,
   receipt/replay, ACK, restore, and error evidence. Reject synthetic labels
   that can mask authoritative result bytes.
4. Replay and receipt: test original, exact duplicate, retained duplicate
   bytes, and conflicting same-transaction replay. Independently validate
   receipt member set, identity, canonical fingerprint, disposition, retained
   hash/length, transaction evidence, root evidence, and receipt hash. Check
   that reference state is not just a boolean shortcut.
5. Durability ACK: effect-test stale, partial, replayed-old,
   newer-write-after-partial, current, duplicate, wrong-world, wrong-context,
   stale/future generation or cut, duplicate/malformed chunk, and wrong-kind
   ACKs. Verify exact covered revisions and that old ACKs cannot clear newer
   dirty state. Independently inspect roots and per-chunk frontier state.
6. Coordinate canonicalization: signed i32 numeric ordering, negative values,
   duplicates, `-0`, leading zeros, boundary and out-of-range values, malformed
   axes, and canonical output. Ensure the oracle is not raw string sorting or
   SUT parser reuse disguised as independence.
7. Lifecycle/public mapping honesty: internal `shutdown` must remain
   `shutdown`; no row, label, assertion, report, or coverage table may claim it
   is the frozen public `destroy` callable. Internal lifecycle labels must be
   explicitly distinguished from missing public status mapping.
8. Corpus and negatives: exactly 11 unique contiguous sequences and the fixed
   independent operation order. Audit abort, restore, lifecycle, budget,
   cancellation, identity/config mismatch, malformed coordinate, replay, and
   ACK negatives. Determine from actual local interfaces whether queue, fault,
   lost-result, cancellation-precedence, status, quiesce, createWorld, and
   destroy effects are locally testable or genuinely require the missing
   callable contract; report any local omissions at their earned severity.
9. Provenance/admission: verify fixed provenance/config values are meaningful,
   immutable evidence and mismatches fail. Empty/unvalidated hashes, tautology,
   self-generated expectations, or acceptance of synthetic provenance are not
   contract evidence.

Audit the full 5,574-line addition for maintainability and false-negative
risk: duplicated SUT algorithms, shared-bug channels, unchecked free-form
strings, fail-open error mapping, test-only assertions that cannot fail,
non-determinism, allocator normalization that hides identity defects, and
oversized hand transcription. Do not pre-judge findings as local P2 or
upstream; classify them from the available interfaces and authority.

## Verification

- Create independent temporary probes outside the candidate paths where
  useful; remove them before final identity checks.
- Run the focused differential repeatedly, the test-support and voxel-world
  suites, workspace/all-feature tests, no-default-feature check, fmt, clippy,
  crate DAG, generated-clean, architecture guards, generated C# builds, and
  metadata/scope/diff checks.
- Recompute golden and mutation-negative evidence with an implementation
  independent of both candidate legs.
- Treat the known Windows symlink spec-lint failure separately; it does not
  waive local P0/P1 findings.
- Final worktree must retain exact base HEAD, empty index, exactly the two
  patch paths, and no temporary artifacts.

## Report Contract

Write verdict and counts, frozen identity, local-versus-upstream disposition,
ordered findings with file/line and deterministic evidence, all prior-finding
reconciliations, exact 11-row observation and callable-coverage tables,
independent golden bytes/digest plus mutation-negative result, actual command
outputs, known gaps, and final VCS state. Send `worker_done` only after the
report is complete. Do not claim product acceptance.
