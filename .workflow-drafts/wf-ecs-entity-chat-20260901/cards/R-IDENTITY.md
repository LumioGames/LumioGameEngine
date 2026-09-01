# [Runtime] Common Connection Binding and NetEntityId Attribute Query

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-IDENTITY`
- Target repository: `C:/Work/LumioGames/LumioGameRuntime`
- Category: runtime / contract
- Module: ECS identity and query
- Wave: 1
- Priority: P0
- Risk: high
- Readiness: conditional until the public query contract is frozen

## Background

Chat, targeting, ownership, UI inspection and future gameplay all need the same way to map a connection to its Entity and query Entity attributes. This is a shared ECS capability, not Chat-specific sender logic.

## Goal

Provide generation-safe connection binding, self Entity lookup and typed Attribute Query by NetEntityId for server and client worlds.

## Preconditions

- ECS World/LocalEntityId/Generation: `R-00149`.
- ECS Query and Snapshot Views: `R-00150`.
- Owner-thread fail-stop: `R-00152`.
- Replication Mapping and Net/Local identity context: `R-00172`.

## Requirements

- Maintain `AccountId + RoomId + NetEntityId + EntityType + ConnectionGeneration` binding after admission.
- Expose the current self NetEntityId to a client and resolve admitted connections/NetEntityIds on the server.
- Address Attribute Query with generated stable AttributeId values and return typed values plus observed Revision/Tick.
- Enforce World/Room, visibility, claims and stale-generation checks.
- Return explicit non-existent, stale, invisible and unauthorized outcomes; never alias a destroyed or tombstoned ID.
- Keep server reads on the Simulation Owner Thread and client reads local to ReplicaWorld.

## Acceptance

- Every admitted connection resolves exactly one current NetEntityId.
- Server lookup maps each ID to the correct authoritative Entity and AccountId.
- Client lookup reads only replicated/visible fields; server-only and persist-only fields are rejected.
- Delayed references to destroyed A do not resolve to replacement B.

## Boundary

No SQL, arbitrary property-name lookup, direct storage access, cross-Room query or public AccountEntity object reference.

\n