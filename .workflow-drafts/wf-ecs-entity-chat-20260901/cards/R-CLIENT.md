# [Client] ReplicaWorld Entity Mapping and Chat Presentation

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-CLIENT`
- Target repository: `C:/Work/LumioGames/LumioClient`
- Category: client / replica
- Module: ReplicaWorld and Browser/Bot consumers
- Wave: 3
- Priority: P0
- Risk: high
- Readiness: conditional

## Background

Every client owns a separate ReplicaWorld. The Browser and each Bot do not share ECS objects with the server or with one another.

## Goal

Consume full snapshots and Chat events into a client ReplicaWorld, expose self Entity lookup and render live Room chat.

## Preconditions

- Common identity/query: `R-IDENTITY`.
- Formal Chat mapping: `R-CHAT-REPLICATION`.
- Existing ClientReplicaSession and full snapshot path: `R-00279`, `R-00281`.

## Requirements

- Maintain one independent ReplicaWorld per client connection.
- Apply FullSnapshot and Delta through the existing client authority-update transaction.
- Resolve the client's self NetEntityId and visible remote NetEntityIds locally.
- Query only replicated, visible AttributeIds from ReplicaWorld.
- Append accepted ChatMessageEvents to the Browser/Bot chat presentation in server order.
- Treat chat-window contents as client-only presentation state.

## Acceptance

- Browser displays messages from BotEntity senders with the correct sender NetEntityId and text.
- Two clients receive equivalent ordered events without sharing object references.
- A malformed or unauthorized event does not mutate ReplicaWorld or the chat window.

## Boundary

No direct server database query, shared World/Entity storage, Chat history restore or client authority over server Entity state.

\n