---
name: runtime-manager-query-expiry
description: Runtime owner-thread expiry, binding resolution, and attribute query controls for the R5 host bridge
metadata:
  type: doc
  status: implementation planned
---

# Runtime Manager Query and Expiry

## Purpose

R5 host integration needs authoritative expiry and C-2 query behavior without
opening another HostEntry operation or maintaining a second Server-side
authority. Runtime owns the world, binding query, and owner-thread ordering.

## Design

`ExpireEntityMessage`, `ResolveBindingMessage`, and `AttributeQueryMessage` are
in-process `WorldMessage` values. Their required fields are defined in
`engine/wire/entity-binding-and-query-v1.json`; they are not WebSocket frames.
Network or host code submits them through `WorldManager.Enqueue`. `WorldManager`
dispatches them on the owner thread before normal input and appends explicit
result records to the internal `drain.queries` collection.

`EntityBindingQuery` remains the only adapter that can mutate bindings or read
declared attributes. It maps its existing C-2 result outcomes into the result
records, including observed revision and tick. Query requests carry an optional
connection and generation for owner/claim visibility and stale-generation
checks. A request correlation id is transient and is never persisted.

Expiry queues entity destruction on the Runtime owner thread. The normal commit
and projection pipeline emits a destroy record and derives the tombstone.
Repeating expiry is idempotent. Resolution and attribute query never replace a
tombstoned identity with another entity.

## Host Boundary

HostEntry keeps exactly `boot`, `enqueue`, `tick`, `drain`, `snapshot`, and
`restore`. Its `drain` response has two independent collections: encoded C-1
`frames` and internal `queries`. The Server consumes query metadata only to
complete its RuntimeSurface; it never decodes C-1 bytes, builds snapshots, or
stores binding/query state.

## Error Semantics

Malformed request fields, invalid 128-bit identities, unknown attributes,
cross-room references, stale generations, invisible fields, unauthorized
claims, non-existent identities, and tombstoned identities return explicit
outcomes. Missing or malformed result records are bridge failures, never
silently dropped.

## Verification

Runtime tests cover owner-thread ordering, all query outcomes, expiry and
tombstone behavior, internal result collection, and rejection by the C-1 wire
codec. Server tests cover strict result parsing, six-operation HostEntry
allowlisting, and end-to-end RuntimeSurface behavior. The C-1 message set stays
`Welcome`, `WorldChange`, `InputCommand`, `ConnectionSuperseded`, and `Error`.
