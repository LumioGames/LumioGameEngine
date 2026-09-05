# ADR-061: RM-00011 A2 Runtime Owner-Thread Query and Expiry Controls

- **Status**: Draft
Date: 2026-09-05
Owners: LumioGameEngineArchitecture, LumioGameRuntime
Supersedes: None

## Background

R5-02 and R5-03 already route admission, disconnect, and rebind intent through
`WorldManager.Enqueue`. The Server bridge is intentionally closed to the six
operations `boot`, `enqueue`, `tick`, `drain`, `snapshot`, and `restore`.
Runtime expiry and C-2 authoritative attribute/query resolution still exist only
as synchronous Replication calls, so the Server cannot complete the lifecycle
contract without a forbidden extra HostEntry operation or a second authority.

## Decision

Add Runtime-owned, in-process `WorldMessage` controls for expiry, binding
resolution, and attribute query. All three requests enter through
`WorldManager.Enqueue` and are executed on the owner thread during `Tick`.
Admission, disconnect, rebind, expiry, binding resolution, and attribute query
remain owned by the single `EntityBindingQuery` adapter; the ECS manager stores
no connection-keyed or query-result authority.

The Runtime emits query results through a separate `queries` collection in the
`DrainOutbox` host response. These results are internal bridge data and are not
C-1 WebSocket frames. `WireCodec` continues to encode only the frozen C-1 set:
`Welcome`, `WorldChange`, `InputCommand`, `ConnectionSuperseded`, and `Error`.

## Alternatives

1. Add a seventh HostEntry operation for synchronous `Expire`/query. Rejected:
   it violates the R5-03 bridge allowlist and puts lifecycle authority back in
   the host boundary.
2. Keep synchronous Replication calls behind the existing six operations.
   Rejected: it bypasses the Owner Thread and cannot be ordered with Tick.
3. Add typed Runtime messages and owner-thread result collection. Chosen: it
   preserves one ingress, deterministic ordering, and the closed network bridge.

## Contract Schema

`engine/wire/entity-binding-and-query-v1.json` gains an `ownerThreadControls`
section with these in-process messages:

- `ExpireEntityMessage(requestId, netEntityId, connection?)`
- `ResolveBindingMessage(requestId, roomId, netEntityId, connectionGeneration?, connection?)`
- `AttributeQueryMessage(requestId, callerScope, roomId, netEntityId, attributeId, connectionGeneration?, connection?)`

The corresponding result records are internal and are returned in `drain.queries`:

- `expire`: `accepted`, `tombstoned`, `non_existent`, or a request error
- `resolve`: `ok` with the Runtime binding five-tuple, or an explicit failure
- `attribute`: `ok` with value/revision/tick, or an explicit C-2 failure

`requestId` is a bridge correlation token only; it is not a game identity and
is not persisted. Optional `connectionGeneration` is validated as an unsigned
integer. Invalid types, malformed 128-bit IDs, undeclared attributes, stale
generations, invisible fields, unauthorized claims, and tombstoned IDs produce
explicit results and never alias another entity.

## Failure Semantics

Expiry queues destruction on the Runtime owner thread. The normal commit and
projection phases emit the authoritative destroy record, then the identity is
tombstoned. A repeated expiry is idempotent and reports `tombstoned`.
Query execution observes the same World revision and Tick as the owner-thread
operation. The Server must surface Runtime errors and must not synthesize local
bindings, values, or tombstones.

## Compatibility and Migration

No C-1 wire message is added or renamed. HostEntry keeps the six-operation
allowlist. Server CLR replaces the temporary unavailable responses with
enqueue/tick/drain correlation. Existing Runtime lifecycle messages remain
unchanged. The architecture contract and generated verification are updated
before Runtime consumers are changed.

## Verification Fixtures

- Enqueued expiry runs only at owner Tick, destroys a live entity, and leaves a
  tombstone that rejects subsequent resolution.
- Enqueued attribute queries return each C-2 outcome with observed Tick and
  revision, including cross-room, stale-generation, invisible, unauthorized,
  and tombstoned cases.
- A drain containing query results and C-1 frames preserves both collections;
  `WireCodec.EncodePack` rejects every internal control/result message.
- Server architecture tests prove the six HostEntry operations and no local
  query/binding authority; Client and Game tests consume only Runtime codec and
  manager APIs.
