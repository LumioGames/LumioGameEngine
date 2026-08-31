# Runtime ECS Full Final Deep Review

## Objective

Independently review the complete ECS delivery from baseline
`ef822a76cd5586513ea6e52b3ea4f5497917bdc8` through candidate
`e1d2e803b7986122b94d196cbda4493055255b7f` plus the final uncommitted
follow-up overlay. This is a whole-module review, not a test-count confirmation.

Review only: do not edit implementation/tests, stage, commit, push, write
Workflow, or consume this candidate for integration.

## Materials

- Authoritative review worktree:
  `C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-ecs-final-review`
- Prior independent RETURN:
  `C:\Work\LumioGames\_codex-verification\ecs-followup-review-return.md`
- Final implementer report:
  `C:\Work\LumioGames\_codex-verification\ecs-final-followup-fix-report.md`
- Complete ECS package relative to `ef822a`:
  `C:\Work\LumioGames\_codex-verification\runtime-ecs-full-review.patch`
- Required report output:
  `C:\Work\LumioGames\_codex-verification\runtime-ecs-final-review-report.md`

Read target `AGENTS.md`, `.spec` core docs, `before-you-code`, reviewer rules,
repository/testing standards, ECS README, generated contract mirror, and
relevant architecture/ADR documents before review.

## Mandatory Review Matrix

1. Owner/lifecycle authority: prove every public authoritative transition after
   owner binding, including `Fault`, `Start`, cleanup and disposal, enforces the
   owner-thread and lifecycle contract. Exercise pre-start cross-thread fault,
   concurrent Start/Fault, reentrant adapter callbacks, running fault, draining,
   faulted and disposed paths. Internal fail-stop must not reopen a public
   non-owner state-transition bypass.
2. Capability provenance: try to forge component registration, world context,
   entity target, component/type/field handles and snapshot access from the
   Command friend assembly surface. Verify there is no friend-callable factory,
   constructible implementation, raw LocalEntityId rebinding path, value-only
   validation or cross-world/incarnation replay. Legitimate capabilities must be
   ECS-issued, reference/world/incarnation bound and opaque in normal code; test
   reflection is not production issuance.
3. Handle/error semantics: stale generations, foreign world/incarnation,
   destroyed entities, released snapshots, double release, invalid type/field
   and disposed/faulted accesses must return their exact frozen stable IDs.
   Tests must assert literal contract meanings rather than aliases that collapse
   distinct errors.
4. Storage/membership/query/snapshot: verify component membership is independent
   of field count and survives create, destroy, clone, snapshot and query for
   zero-field/multi-field components. Check generation reuse, required/excluded
   queries, defensive copies, deterministic ordering, capacity and overflow.
5. Snapshot lease lifecycle: public reads stay closed in Faulted/Disposed, but
   every already-issued pin can be released exactly once after fault. Verify
   release ordering, adapter exception behavior, double release, world cleanup,
   concurrent lifecycle/snapshot locking and no leaked or prematurely released
   pins.
6. Adapter/reentrancy/fatal evidence: all adapter exceptions and post-write
   failures must fail-stop, roll back caller-visible outputs/provisional state,
   avoid partial publication, and retain immutable evidence available at the
   exact failure site. Trace create/destroy/register/read/snapshot/query paths;
   validate tick, processor/evidence identity, entity/component/field,
   operation, and partial-change count are real rather than default/fabricated.
7. Contract/public boundary: no arbitrary public registration schema or local
   substitute for the absent generated Component/Field/LogicTransform metadata
   projection. The legitimate interim seam must remain internal/fail closed and
   not become a competing public contract. Check all exported types,
   constructors and friend visibility.
8. Compatibility/tests/scope: inspect the complete module diff for API
   compatibility, synchronization, aliasing/mutability, arithmetic boundaries,
   exception mapping, deterministic behavior, false-success paths and
   self-fulfilling reflection tests. Run locked restore, dual-TFM Release builds,
   complete ECS tests, dependency, scoped format, diff and applicable contract
   gates in this independent snapshot. Record repository-wide Windows
   link/MTP gaps separately.

## Verdict

Write `PASS` or `RETURN`; any P0/P1 requires `RETURN`. The report must declare
coverage of all eight dimensions, give exact file/line plus a concrete failure
scenario for each evidence-based finding, include fresh command exit/counts,
reconcile all eight prior P1s and known regressions, state known gaps, and give
an explicit integration-consumption decision.
