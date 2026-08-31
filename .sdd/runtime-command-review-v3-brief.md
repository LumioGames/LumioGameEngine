# Runtime Command/Coordination V3 Deep Review

## Objective

Independently review the complete second follow-up overlay based on
`1c53e2b7bc14d7f01a24a38ce6dc5d52448b2708`. Reconcile every original and V2
finding and look for new correctness failures in revision reservation, durable
evidence, recovery, Voxel visibility, fixtures and default composition.

Review only. Do not edit implementation/tests, stage, commit, push, write
Workflow, or consume the candidate for D-005.

## Materials

- Authoritative review worktree:
  `C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-command-review-v3`
- Original D-005 rejection:
  `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\runtime-command-review-final-report.md`
- V2 independent RETURN:
  `C:\Work\LumioGames\_codex-verification\runtime-command-review-v2-report.md`
- V2 implementer report:
  `C:\Work\LumioGames\_codex-verification\runtime-command-review-v2-fix-report.md`
- Complete current overlay package:
  `C:\Work\LumioGames\_codex-verification\runtime-command-review-v3.patch`
- Required report:
  `C:\Work\LumioGames\_codex-verification\runtime-command-review-v3-report.md`

Read target AGENTS/.spec core docs, reviewer/before-you-code/testing standards,
Command and Coordination READMEs, architecture transaction/revision/journal
contracts, and generated Voxel schema metadata.

## Mandatory Matrix

1. Standard runner and scope: verify `global.json`/MTP commands from this clean
   snapshot, both production TFMs, complete Command/Coordination tests, package
   presence and all 35 paths. Resolve the reported 51-versus-52 Coordination
   count discrepancy from fresh output.
2. Strict revision reservation: adversarially test stale/equal, regressing,
   wrong schema/epoch, mismatched participant vectors, concurrent coordinators,
   reservation abandonment, exception/failure paths and reentrant recovery.
   No terminal marker may be written before an exact strict reservation; no
   leaked reservation may deadlock later transactions. Idempotent replay is
   exact transaction evidence, not global equality acceptance.
3. Durable result evidence: trace identity, request/release/expected/result
   vector, digest, schema and corruption checks. Evidence must be durably written
   before marker authority; marker-only recovery with both participants
   unavailable must reconstruct exactly after process loss. Missing/mismatched/
   corrupt evidence fails closed. Verify default/public composition cannot claim
   process-loss durability through an in-memory-only store and that bounded
   eviction cannot erase still-retained committed evidence.
4. Crash ordering and recovery: probe every window before/after reservation,
   evidence, marker, local store advance and record transition. Check evidence
   without marker, marker without evidence, duplicate replay, retry after failed
   writes, local store ahead/behind and recovery atomicity.
5. Voxel boundary: reflect both main Coordination and VoxelAdapters assemblies,
   including request/result/status/interface types, methods, properties,
   constructors, leases, coordinators and explicit interface implementations.
   No external assembly may implement/inject the hand-authored parallel Voxel
   contract. Public/default composition remains `CapabilityMissing` until the
   generated callable projection exists.
6. Fixtures/token/defaults: verify every JSON field drives replay/assertions;
   five pairs are behaviorally complementary and carry their own durable crash
   evidence. Recheck canonical token identity/framing and all public/default
   no-op false-success paths.
7. Compatibility/quality: inspect concurrency/locking, cleanup, aliasing,
   immutability, arithmetic/overflow, stable errors, API visibility, test
   quality and out-of-scope changes. Re-run dependency/generated/SDK/format/
   diff gates and LF architecture validation; record environment gaps exactly.

## Verdict

Write `PASS` or `RETURN`; any P0/P1 means RETURN. Include coverage for all seven
dimensions, exact file/line and failure scenario per finding, fresh commands/
counts, reconciliation tables for original and V2 findings, known gaps, and an
explicit D-005 consumption decision.
