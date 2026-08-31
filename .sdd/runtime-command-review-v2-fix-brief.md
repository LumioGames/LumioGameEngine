# Runtime Command/Coordination Review V2 Fix Brief

## Context and Boundary

Owner worktree:
`C:\Users\g923\orca\workspaces\LumioGameRuntime\runtime-command-wave1`

Base/HEAD remains `1c53e2b7bc14d7f01a24a38ce6dc5d52448b2708`; the 32-path
follow-up overlay is uncommitted. Independent report:
`C:\Work\LumioGames\_codex-verification\runtime-command-review-v2-report.md`.

Fix every P1 and P2 below with TDD in the owner worktree. Preserve the already
fixed MTP runner, fail-closed defaults, token identity, and architecture
validation. Do not touch `.gitattributes`, Generated/**, ECS, Simulation,
Replication, AOI, NativeCore, Architecture, Server, or Workflow. Do not commit
or push. Do not spawn subagents.

## Required Fixes

### 1. Strict revision advance before authoritative commit

- Intent recovery must not treat participant `Applied(currentVector)` as a new
  successful transaction. Both participant vectors must exist, match, satisfy
  schema/epoch rules, and strictly advance the transaction's expected/current
  vector before a new commit can be published.
- Matching but regressing or wrong-schema vectors must be rejected before any
  durable `Committed` marker is appended.
- Preserve idempotent replay only when durable evidence proves this exact
  transaction already committed this exact vector; do not implement idempotence
  by globally treating equality as a new advance.
- Make the validation/reservation and post-marker local advance concurrency-safe
  so another coordinator cannot invalidate the precheck between those steps.

Required RED cases include stale-equal intent recovery, regressing vectors,
wrong schema/epoch, and a race/interleaving around marker append.

### 2. Marker-only durable result-vector recovery

A durable `Committed` marker may be authoritative only when its exact
`SessionRevisionVector` can be reconstructed after process loss without relying
on retained participant query receipts.

- Add or use an in-scope durable result-evidence abstraction with explicit
  write/read and transaction identity semantics, ordered so the result vector is
  durable before the `Committed` marker becomes authoritative.
- Recovery with a committed marker and both participant query ports unavailable
  must recover the exact vector from durable evidence, restore the revision
  store and record, and remain idempotent.
- Missing/mismatched/corrupt durable evidence must fail closed, never guess.
- An in-memory-only field on `TxnRecord`, encoding data into hash/length side
  channels, or a test stub that supplies the vector through participant queries
  does not satisfy this requirement.
- Keep the abstraction a Runtime coordination persistence seam, not a duplicate
  public wire/generated schema. Default composition must stay fail closed when
  durable evidence capability is absent.

Cover crash windows before evidence, after evidence/before marker, after
marker/before local-store advance, and replay after local-store restoration.

### 3. Remove the remaining public Voxel parallel contract

The following hand-authored seam in the main Coordination assembly remains a
public competitor to generated `voxel-world-port`: `IVoxelWorldPort`,
`VoxelPrepareRequest`, Voxel status/result records, and public constructors that
accept them.

- Make the entire hand-authored Voxel seam internal, including construction/
  injection paths, or replace it only with an actually generated callable
  projection if one now exists.
- Reflection tests must inspect both Coordination and VoxelAdapters assemblies,
  and must detect public Voxel request/result/interface/constructor surfaces, not
  only names prefixed `GeneratedVoxel`.
- Public/default composition remains explicitly `CapabilityMissing` until the
  architecture generator publishes the callable projection. Do not invent the
  missing public generated contract locally.

### 4. Make durable fixtures behaviorally complementary

- Replace the duplicated lost-result and partial-commit valid/invalid pairs with
  genuinely different replay behavior and assertions.
- Fixture JSON, not test-only synthesis, must carry the durable crash-boundary
  evidence needed by the scenario.
- Read and assert fields such as `expectedErrorId`; every fixture field must
  influence replay/assertion or be removed.
- Keep five discoverable positive/negative pairs for duplicate, timeout,
  lost-result, partial-commit, and crash boundary, with meaningful complementary
  outcomes.

## Verification and Delivery

Write the implementer report to:
`C:\Work\LumioGames\_codex-verification\runtime-command-review-v2-fix-report.md`.

Report:

- Real RED evidence for each group and final GREEN counts.
- Exact changed/untracked paths and proof all are within the allowed boundary.
- Locked restore; Command/Coordination/VoxelAdapters dual-TFM Release builds;
  official MTP tests; dependency/generated/SDK/format/diff gates; reviewed LF
  architecture validation; exact exit codes and key output.
- Known gaps and knowledge-sync decision.

Leave all changes uncommitted for a new independent deep review.
