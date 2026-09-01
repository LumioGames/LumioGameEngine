# ADR-028: Replication Typed Bodies and MessageType Registry

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime`
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-005](ADR-005-replication-prediction.md). The Accepted ADR-005 Decision text is unchanged.

## Context

The envelope used an untyped `payload` and omitted TickId, SessionRevisionVector and MappingSetHash on FullSnapshot. Envelope `messageType` values `BaselineAck`, `DeltaAck` and `Error` were not in the ID Registry.

## Decision

The envelope is separated from the typed body. Each of Handshake, FullSnapshot, BaselineAck, Delta, DeltaAck, ResyncRequest, MaintenanceKick and Error has a required `body` object. FullSnapshot body requires `tickId`, a complete `SessionRevisionVector`, `schemaEpoch` and `mappingSetHash`. Delta body requires `baseSnapshotId`, `fromRevision`, `toRevision`, `mappingSetHash`, `confirmationSequence` and `tombstones`.

The envelope `transportPolicy` freezes `maxMessageBytes`, `maxFragmentBytes`, `antiReplayWindow`, `authBinding` and the three error classes `Retryable | Rejectable | Fatal`.

`MessageType` registry adds `BaselineAck`, `DeltaAck` and `Error`. `lumio_contract.py` requires the Schema enum, the ID Registry and fixture-used messageTypes to be the same set.

## Contract

`replication-envelope.schema.json` plus the MessageType namespace.

## Failure semantics

A FullSnapshot missing TickId, RevisionVector or MappingSetHash is invalid. A fixture or envelope whose `messageType` is not registered is invalid.

## Alternatives

Keeping a free-form payload was rejected because two implementations can pass the gate and disagree on Snapshot identity.

## Compatibility and migration

Breaking envelope shape in `LGE-V1.3-2026-08-27`; no deployed wire consumer.

## Verification

Fixtures `replication/full-snapshot`, `replication/delta`, `replication/missing-snapshot-identity`, `replication/unregistered-message-type`.
