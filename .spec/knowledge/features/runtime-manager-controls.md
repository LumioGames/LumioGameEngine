---
name: runtime-manager-controls
description: Runtime-owned admission, disconnect, and rebind controls that enter the WorldManager owner-thread queue
metadata:
  type: doc
  status: 已交付
---

# Runtime Manager Controls

The Runtime needs one owner-thread entry for connection lifecycle mutations. Host
transport code must be able to enqueue admission, disconnect, and rebind intent
without owning a second binding table, a local Welcome path, or a second codec.

## Goal

Make the R5 host contract executable while keeping the ECS and Replication module
boundary one-directional. Network threads enqueue typed lifecycle intent; the
Simulation Owner Thread applies it during `WorldManager.Tick`; the existing
Replication binding query remains the only binding authority.

## Design

### Internal control messages

Add three typed `WorldMessage` variants in the ECS assembly:

- `AdmitConnectionMessage(connection, accountId, roomId, entityType)`
- `DisconnectConnectionMessage(connection)`
- `RebindConnectionMessage(connection, accountId, roomId, mode)`

`mode` is the string contract value `reconnect` or `takeover`. These messages are
in-process Runtime controls. They are not C-1 WebSocket messages and are never
encoded by `WireCodec`; the C-1 wire set remains `Welcome`, `WorldChange`,
`InputCommand`, `ConnectionSuperseded`, and `Error`.

### ECS-to-Replication boundary

ECS defines one narrow `IWorldControlAdapter` interface. The adapter accepts a
control message on the owner thread and returns either no response or an
`ErrorMessage` addressed to the original connection. It also resolves an
observer identity to its opaque connection reference for projection routing.

`WorldManager` owns at most one adapter reference. `EntityBindingQuery.Create`
registers its adapter and unregisters it on dispose. ECS never references
Replication types; the adapter implementation delegates to the existing
`Admit`, `Disconnect`, and `Rebind` methods.

### Tick ordering and routing

On a server tick, `WorldManager` dispatches lifecycle controls before ordinary
input commands, commits creates, then projects. A successful admission therefore
becomes a normal Runtime create plus `Welcome` in that tick. Rebind takeover
uses the existing Runtime supersession notice and the next projection emits the
new generation's `Welcome`. Rejected controls produce an `ErrorMessage` in the
outbox with the request connection; no synchronous entity ID is returned.

The adapter supplies connection routing as a callback during projection. The
Manager does not store a connection-keyed binding table. Connection and room
indexes remain owned by `EntityBindingQuery`; they are not persisted in snapshots.

### Contract surface

`entity-binding-and-query-v1.json` records the three internal control messages,
their in-process transport, required fields, accepted outcomes, and the rule that
they enter through `WorldManager.Enqueue`. The C-1 gameplay envelope is unchanged.

## Error handling

Malformed controls, unknown entity types, duplicate online accounts, stale
reconnects, and missing bindings use the existing C-2 outcome codes. The adapter
maps rejected outcomes to a Runtime `ErrorMessage` addressed to the supplied
connection. Host transport only forwards encoded outbox bytes and does not
reinterpret the result.

After the Host authenticates the account or Bot tool credential, both `player`
and `bot` are valid admission classifications. Runtime trusts that authenticated
classification and creates the matching entity template; any other value is an
invalid binding shape.

## Tests

- ECS tests verify all three control messages are `WorldMessage` values, are
  rejected by `WireCodec`, and are processed only during the owner tick.
- Replication tests enqueue admission from a non-owner thread, assert that the
  owner tick emits `Welcome` and `WorldChange`, and verify disconnect/rebind
  generation and takeover supersession behavior.
- Contract verification asserts the C-2 control-message table and preserves the
  five-message C-1 set.
- Existing Runtime ECS, Replication, sample, generator, and wire verification
  suites remain green.

## Non-goals

This design does not add a network admission envelope, a Manager-owned connection
dictionary, a local snapshot/delta format, or a synchronous `netEntityId` result.

## Delivered implementation

The Runtime ECS implementation is in `3bba165` and the Replication adapter is in
`a230a74`. `WorldManager.Enqueue` is the only cross-thread entry; lifecycle controls
are consumed before ordinary inputs on the owner tick. `EntityBindingQuery` registers
the single adapter and detaches it on disposal. The architecture contract and validator
are in `f07add7`; the handback report is `1bb6a32`.
