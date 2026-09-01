# [Replication] ChatInput and ChatMessageEvent Typed Mapping

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-CHAT-REPLICATION`
- Target repository: `C:/Work/LumioGames/LumioGameRuntime` plus Server/Client adapters
- Category: protocol / replication
- Module: Chat typed mapping
- Wave: 2
- Priority: P0
- Risk: high
- Readiness: conditional until the formal generated mapping surface is frozen

## Background

The old Hello wire contract is a development artifact and cannot become a second protocol truth. Formal Chat must use the typed mapping path already planned for InputCommand and replication state blocks.

## Goal

Define the minimum formal Chat input/event mapping and reliable ordered Room delivery.

## Preconditions

- Typed replication bodies and registry: `ADR-028`, `ADR-045`.
- Draft state/input carriage: `ADR-049`.
- Runtime identity mapping: `R-IDENTITY`.

## Requirements

- Gameplay ChatInput contains message text only; no client input sequence or client frame.
- Transport/session sequencing and connection-generation validation remain protocol-layer concerns.
- ChatMessageEvent contains server MessageId, strict Room order, sender NetEntityId, text and authoritative applied Tick.
- The first channel is the current Room public channel and all permitted Room members receive the event reliably and in order.
- Duplicate events are suppressed by server identity/order; malformed or unauthorized input produces no ECS write.
- State uses the formal typed mapping path: InputCommand upstream and stateBlocks/changedBlocks downstream. Do not extend hello-wire-v1.

## Acceptance

- A Bot or Browser sends only text and all permitted clients display one ordered event.
- Out-of-order, duplicate, stale-generation and unauthorized inputs are rejected or suppressed without state corruption.
- The same mapping validates in two identical runs and maps the event sender to the correct NetEntityId.

## Boundary

No Chat-specific parallel envelope, free-form Ack payload, client-selected sender, Chat history or compression policy.

\n