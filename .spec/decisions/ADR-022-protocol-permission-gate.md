# ADR-022: Generated Protocol/Permission Gate

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.2-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameEngineArchitecture` (generated validator), `LumioGameRuntime` (MessageType namespace)
- **Baseline**: `LGE-V1.2-2026-08-27`
- **Relation**: Uses the V1 replication `MessageType` surface from [ADR-005](ADR-005-replication-prediction.md). Does not freeze D-009（旧制度 `DECISIONS_PENDING` 台账，已随架构源制度废止；见 git 历史） RPC/Message dispatch.

## Context

Active-session messages must be rejected before replica/prediction semantics if Session, Release, Role, Claims, Message identity or Connection Generation do not match admission. Connection-layer anti-replay is already a transport concern. Leaving the Active-message field set to each repository would drift Server and Client gates. D-009 remains unfrozen, so this ADR must not invent an RPC envelope.

## Decision

The architecture toolchain generates the Protocol/Permission Validator from this repository's Schema and ID Registry. V1 Active messages use the existing `MessageType` namespace as `messageId`. The generated gate checks, and only checks, this field set:

- `sessionId` equals the admitted `ClientReplicaSession` identity.
- `productId` + `gameReleaseId` equal the admitted exact Release.
- `messageId` is registered and permitted for the admitted Role.
- `role` equals the admitted Role (`Server` / `Client` / `Replay`).
- `claims` is a subset of the claims granted at Handshake admission.
- `connectionGeneration` equals the current connection generation.

Anti-replay ownership is split:

- Connection-scope anti-replay (frame/channel sequence) belongs to the Connection layer.
- Session-scope anti-replay (session message sequence / window after admission) belongs to the `ClientReplicaSession` owner. This names the public session aggregate, not a client implementation module.

Role and Claims are admission context, not per-message inventions. Handshake remains the admission owner. Rejected messages never enter replica or prediction semantics. LocalEmbedded uses the same generated gate.

This ADR does not define Session Resume Token, RPC dispatch, or an auth ticket wire format (D-011).

## Contract

`protocol-permission-gate.schema.json` is the generatable validator record: message fields, admitted context, verdict and reject reason. Error codes `MessagePermissionDenied` and `StaleConnectionGeneration` are registered for stable rejects.

## Failure semantics

Any field mismatch or session-scope replay is `Reject`. A Connection Generation mismatch uses `StaleConnectionGeneration`. Other gate failures use `MessagePermissionDenied`. Accept with a mismatched generation or a claim outside admission is invalid.

## Alternatives

Hand-written per-repo validators were rejected for drift. Embedding Role/Claims on every wire envelope was rejected; they are admission context checked by the generated gate. Freezing D-009 in this ADR was rejected so Game RPC remains blocked until its own design lands.

## Compatibility and migration

Additive in `LGE-V1.2-2026-08-27`. When D-009 is later accepted, new MessageIds join the same gate field set; they do not replace it.

## Verification

Fixtures `gate/accept` (positive) and `gate/stale-generation` (failure: Accept despite generation mismatch).
