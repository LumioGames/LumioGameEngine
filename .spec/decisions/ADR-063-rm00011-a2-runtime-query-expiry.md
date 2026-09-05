# ADR-063: RM-00011 Runtime Owner-Thread Query and Expiry Controls

- **Status**: Draft
Date: 2026-09-05
Owners: LumioGameEngineArchitecture, LumioGameRuntime
Supersedes: None

## Background

The R5 host bridge has a closed six-operation surface: `boot`, `enqueue`,
`tick`, `drain`, `snapshot`, and `restore`. Binding resolution, attribute
query, and entity expiry must therefore not become synchronous HostEntry
operations or a second Server-side authority.

## Decision

Expiry, binding resolution, and attribute query are Runtime-owned,
in-process `WorldMessage` controls. Each request enters through
`WorldManager.Enqueue` and executes on the owner thread during `Tick`.
`EntityBindingQuery` remains the only binding/query adapter; the ECS manager
does not store connection-keyed or query-result authority.

The Runtime returns explicit result records in the internal `drain.queries`
collection. These records are bridge data, not C-1 WebSocket frames. The
frozen C-1 message set remains `Welcome`, `WorldChange`, `InputCommand`,
`ConnectionSuperseded`, and `Error`.

## Contract Schema

`engine/wire/entity-binding-and-query-v1.json` defines three request types:

- `ExpireEntityMessage(requestId, netEntityId, connection?)`
- `ResolveBindingMessage(requestId, roomId, netEntityId, connectionGeneration?, connection?)`
- `AttributeQueryMessage(requestId, callerScope, roomId, netEntityId, attributeId, connectionGeneration?, connection?)`

Their corresponding internal result records are `expire`, `resolve`, and
`attribute` in `drain.queries`. `requestId` is only a transient correlation
token; it is not a game identity and is not persisted. Optional connection
generation values are unsigned and are checked by Runtime.

Each result record has a closed `fields` map whose keys are exactly the union
of its `required` and `optional` lists, with field type expressions fixed by the
contract. `outcomeShapes` closes the result union: terminal and C-2 failure
outcomes carry only `requestId` and `outcome`; `request_error` additionally
requires `code` and `detail`; resolve `ok` additionally requires `binding` and
`observedRevision`; attribute `ok` additionally requires the returned identity,
value, observed revision, and observed tick.

## Failure Semantics

Expiry queues destruction on the Runtime owner thread. The normal commit and
projection path emits the destroy record, after which the identity is derived
as tombstoned; repeated expiry is idempotent. Resolve and attribute requests
return explicit C-2 outcomes for malformed or cross-room references,
non-existent, stale-generation, invisible, unauthorized, and tombstoned
identities. They never alias a replacement identity or synthesize local
values.

## Compatibility

No C-1 frame is added or renamed, and HostEntry keeps exactly its six
operations. Runtime controls are rejected by the C-1 wire codec and are
available only through the in-process enqueue/tick/drain bridge.

## Verification Fixtures

- Owner-thread expiry is not applied before `Tick`, and a repeated expiry
  reports `tombstoned`.
- Queries preserve their request correlation and return the observed revision
  and tick for successful attribute reads.
- A drain may carry independent C-1 frames and internal `queries`; the latter
  never enter network encoding.
