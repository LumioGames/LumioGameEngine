# ADR-029: Entity Namespace Required and Domain Constraints

- **Status**: Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime`
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-004](ADR-004-entity-identity.md). The Accepted ADR-004 Decision text is unchanged.

## Context

`namespace` existed on the entity schema but was optional, so Provisional/Authoritative/Replay checks could be skipped.

## Decision

`namespace` is required. `Provisional` must use a client authority domain (`client-` prefix). `Authoritative` must not use a provisional domain. `Replay` (and migration-retained IDs) must carry `sourceRevision` and `sourceReleaseId` when they keep the original NetEntityId.

## Contract

`entity-identity.schema.json`.

## Failure semantics

Missing namespace, Provisional without a client domain, Authoritative with a client provisional domain, or Replay without source Revision/Release are invalid.

## Alternatives

Inferring namespace from authorityDomain was rejected because the two fields would drift.

## Compatibility and migration

Additive required field in `LGE-V1.3-2026-08-27`. Existing fixtures already set namespace.

## Verification

Fixtures `entity/tombstone`, `entity/provisional-remap`, `entity/missing-namespace`, `entity/authoritative-provisional-domain`, `entity/replay-missing-source`.
