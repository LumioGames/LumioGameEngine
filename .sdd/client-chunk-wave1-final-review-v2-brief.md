# Client Chunk Wave 1 Final Review V2 Brief

## Role

Perform a fresh independent adversarial review of the exact three-file Client
Replica candidate. This is review-only: do not edit the owner worktree,
candidate files, public/generated contracts, Workflow, or repository history.
Apply and build only in the isolated review worktree created for this task.

## Frozen Identity

- Base and required review-worktree HEAD:
  `08ffa587c55d03da05a847b3858a860824b41e76`
- Task-entry reference whose requested-revision change must remain preserved:
  `3c8b87190bf00a4fd89b3225948e5b331bab4f62`
- Canonical post-fix patch:
  `C:\Work\LumioGames\_codex-verification\client-chunk-wave1-final-v2.patch`
- Canonical post-fix SHA-256:
  `39248981337D3D2A11502D8992E1EFC4CD374A0DB9813F84ADB68FC124DD1AD3`
- Patch size: `70516` bytes
- Exact scope: three paths, `1292` additions and `128` deletions
- Implementer report:
  `C:\Work\LumioGames\_codex-verification\client-chunk-wave1-p1-fix-report.md`
- Prior review:
  `C:\Work\LumioGames\_codex-verification\client-chunk-wave1-final-review-report.md`
- Output report:
  `C:\Work\LumioGames\_codex-verification\client-chunk-wave1-final-review-v2-report.md`

The implementer report incorrectly calls the old returned patch
`client-chunk-wave1-final.patch` / SHA-256 `B1BA765E...9015` the frozen
candidate. That old package is input/review provenance only. The `v2` patch
and full hash above are the sole post-fix review identity. Verify its hash,
clean base, exact paths, apply check, reverse check, HEAD, and empty index
before substantive review.

## Verdict Contract

Return `PASS` only with zero P0/P1. Otherwise return `RETURN` and enumerate all
findings with severity, exact locations, deterministic counterexample, and
required fix. Report P2 separately. State a closure decision for every prior
P1-01 through P1-05; ordinary green tests do not waive a reproduced defect.

## Binding Behavior

1. The single-item path must perform generation, request identity, requested
   revision, state/revision, apply-byte, resident-byte, and item-count fences
   before any heap allocation or payload/hash span access.
2. A batch must preflight every member's generation, identity, state, revision,
   hash shape, and cumulative apply/resident/item budgets before any member's
   payload/hash materialization, hashing, or mutation. A later stale, malformed,
   throwing, or over-budget member must fail the whole batch with zero earlier
   payload/hash reads and zero partial state/counter changes.
3. Commit-time copy/hash/normalization/manager exceptions must not escape, must
   return the correct stable typed fail-stop result, and must restore all entry
   fields, completed/request tokens, counters, payloads, and hashes exactly.
4. Validation and commit must use one immutable owned capture. Caller mutation,
   disposal, or custom-memory behavior must not produce validation/commit
   TOCTOU, stored payload/hash mismatch, or untyped exceptions. Independently
   determine whether mutation/disposal after `ReplicaChunkUpdate` construction
   but before `Apply` can still change accepted semantics; the implementer
   report discusses only capture during Apply and does not settle this point.
5. Stale tokens for `Ready`, `Unrequested`, and every non-`InFlight` state must
   return typed `StaleRequest` before any payload/hash span access. Exact replay
   behavior must remain deterministic and must not let mismatched identities
   reuse completed state.
6. Preserve requested-revision identity, authoritative-generation success,
   replacement/duplicate/revision-collapse accounting, deterministic canonical
   coordinate ordering, maxChunks/maxBytes/maxApplyItems/maxApplyBytes, and the
   documented Client Owner Thread mutation contract.

## Required Adversarial Probes

- allocation-count probes for stale/invalid single updates and batches whose
  last member fails metadata; inspect the candidate/selected list allocations
  before all-member fences and classify them against the binding resource rule;
- early-valid/later-stale, later-invalid-hash-length, later-over-budget, and
  later-throwing-memory batches proving zero span access and zero commit;
- caller mutation/disposal between update construction and Apply, plus mutation
  between validation and commit, with independent stored-payload/hash checks;
- copy/normalize/hash and commit-snapshot exceptions at every member position,
  including rollback when snapshot creation itself fails partway;
- stale `Ready` and `Unrequested` requests using throwing memory managers;
- exact replay, changed same-token payload/hash, duplicate coordinates,
  replacement, revision collapse, signed-coordinate ordering, and capacity
  boundaries 0/1/max/overflow.

## Verification

Run serially in the isolated worktree:

- focused `ReplicaChunkStateStoreTests` and full Replica tests;
- Release Replica and full solution builds plus every discovered test assembly;
- generated/toolchain/dependency/contract mirror and relevant architecture
  filters;
- repeated allocation/throwing-manager/race probes;
- `git diff --check`, LF/trailing/final-newline scan, exact three-path scope,
  task-entry preservation, patch reverse check, HEAD, staged, and untracked
  checks.

Keep the absent pinned SDK `10.0.400`, Windows link/spec-lint, archive, Bash,
and non-propagating wrapper issues explicit. Installed SDK `10.0.111` evidence
does not waive the pin, while external gaps do not waive local P0/P1.

## Handoff

Write the full report to the output path above. Send exactly one `worker_done`
with verdict, P0/P1/P2 counts, prior-finding reconciliation, report path, and
confirmation that candidate source was not edited outside the isolated review
worktree.
