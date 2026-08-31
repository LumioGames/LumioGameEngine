# Client Chunk Wave 1 Final Review V3 Brief

## Objective

Perform a fresh, independent, adversarial review of the complete three-path
Client Replica Chunk candidate. The implementer evidence is input, not proof.
Return `PASS` only with zero P0 and zero P1 findings. Otherwise return
`RETURN` with exact findings and reproducible evidence.

## Frozen Identity

- Repository: `C:/Work/LumioGames/LumioClient`
- Required clean base HEAD:
  `08ffa587c55d03da05a847b3858a860824b41e76`
- Frozen patch:
  `C:/Work/LumioGames/_codex-verification/client-chunk-wave1-final-v3.patch`
- SHA-256:
  `3AA2E021B4D59994A9E39FD38BDB4DAAA7D25C2F0D1CDB3166E39E40973304D3`
- Patch size: `106002` bytes
- Patch shape: `2239` additions and `181` deletions across exactly three
  paths; LF-only with a final LF.
- Implementer report:
  `C:/Work/LumioGames/_codex-verification/client-chunk-wave1-v2-p1-fix-report.md`
- Prior independent V2 review:
  `C:/Work/LumioGames/_codex-verification/client-chunk-wave1-final-review-v2-report.md`
- Required review report:
  `C:/Work/LumioGames/_codex-verification/client-chunk-wave1-final-review-v3-report.md`

Before review, verify clean HEAD/index/untracked state, exact patch hash and
size, `git apply --check`, exact path/mode set, and after application
`git apply --reverse --check`. Apply the patch only in the isolated review
worktree. Do not edit the owner candidate.

## Exact Scope

The patch may contain only:

1. `modules/replica/src/Internal/ReplicaChunkState.cs`
2. `modules/replica/src/Internal/ReplicaChunkStateStore.cs`
3. `modules/replica/tests/Unit/ReplicaChunkStateStoreTests.cs`

No public/generated contract, schema, ID, fixture, project, dependency,
toolchain, archive, Session, Headless, Connection, or Architecture source may
change. Preserve `RequestedRevision` identity and the public API shape. No
commit, push, stage, Workflow write, or acceptance action is allowed.

## Required Adversarial Review

Independently inspect and effect-test all five prior P1 families. Do not rely
only on supplied unit tests or their assertions.

1. Batch preflight resource and atomicity: last-member stale, malformed raw
   hex, over-item, over-apply-byte, over-resident-byte, conflicting collapse,
   throwing source, and duplicate/replacement cases must perform no forbidden
   pre-fence heap allocation, no earlier payload/hash source reads where the
   fence is knowable from metadata, and no partial state/counter mutation.
   Check overflow and exact-boundary arithmetic and deterministic tie-breaking.
2. Construction ownership: mutate and dispose caller payload/hash storage after
   every constructor form, including custom/throwing memory managers. Accepted
   semantics must remain the value captured at construction, getters must not
   expose mutable backing storage, and malformed input must fail with a stable
   typed result rather than an escaping exception.
3. Reentrancy and commit identity: exercise `Fail`, `Reset`, `Request`, and
   nested `Apply` during construction/capture and during the memory-container
   path, including generation/token/entry replacement. Confirm the outer
   transaction cannot overwrite a winner, leak resident/ready counters, or
   roll back a valid nested winner. Inspect same-thread, callback, and exception
   paths for balanced active/epoch state.
4. Raw 64-byte hash validation: uppercase, non-hex, mixed malformed bytes,
   32-byte digest, valid lowercase hex, empty/short/long inputs, and throwing
   custom storage. A later malformed member must be rejected before earlier
   payload/hash reads during Apply.
5. `ReadOnlyMemory<ReplicaChunkUpdate>` container: enforce Length before Span,
   reject over-limit with zero Span access, map throwing/inconsistent/custom
   Span materialization to a typed fail-stop result, and prevent reentrant
   mutation from reaching outer commit.

Also audit the complete candidate for new P0/P1 defects: integer overflow,
unbounded resource use inside declared limits, accidental quadratic behavior
outside bounded `MaxApplyItems`, public memory aliasing, fail-open exception
paths, incomplete rollback, stale token/revision admission, noncanonical signed
coordinate ordering, state/count divergence, and tests that are tautological,
reflection-only, race-prone, or coupled to implementation details.

## Verification

- Add independent temporary probes where needed, run them, and remove them
  before final identity checks.
- Run focused `ReplicaChunkStateStoreTests`, full Replica tests, full solution
  Release build and all runnable tests using the installed SDK when the pinned
  SDK is unavailable.
- Repeat allocation/reentrancy/custom-memory probes enough times to establish
  determinism.
- Run Architecture generated/dependency/toolchain filters, contract mirror,
  upstream smoke where available, public API/scope checks, and
  `git diff --check`.
- Distinguish local candidate failures from known external SDK pin, archive,
  Bash, Windows symlink, mirror drift, and Architecture compiler-digest gaps.
  A wrapper that prints failing subprocesses but exits zero is not green.
- Final worktree must return to exact base HEAD, empty index, and exactly the
  three patch paths, with no temporary probe files.

## Report Contract

Write the required report with verdict, P0/P1/P2 counts, package identity,
exact scope, findings with file/line and deterministic reproduction, prior
finding reconciliation, adversarial matrix, actual command output/counts,
environmental gaps, and final VCS state. Send `worker_done` only after the
report is complete. Do not claim product acceptance or modify the candidate.
