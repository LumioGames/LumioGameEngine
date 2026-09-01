# [ECS] ChatComponent Field Declarations and SetMessage

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-CHAT-COMPONENT`
- Target repository: `C:/Work/LumioGames/LumioGame`
- Category: gameplay / ECS component
- Module: ChatComponent
- Wave: 2
- Priority: P0
- Risk: high
- Readiness: conditional until the component mapping contract is frozen

## Background

The first gameplay component is a ChatComponent attached to PlayerEntity and BotEntity. Its job is to update last-message state and produce a live event; it does not own transport or persistence infrastructure.

## Goal

Declare ChatComponent in ECS and make `SetMessage` a deterministic authoritative Tick operation.

## Preconditions

- ECS lifecycle/query/field markers: `R-00149`, `R-00150`, `R-00152`.
- Input capture and commit barrier: `R-00178`, `R-00189`.

## Requirements

- `SetMessage(text)` runs only on the Simulation Owner Thread through the normal command/commit path.
- Update `LastMessageText` and `LastMessageTick/Frame` in the same committed Tick.
- Mark those fields persist-only on the authoritative ECS component; they are not a client property-sync stream.
- Emit one authoritative ChatMessageEvent after the component update is committed.
- Perform no network I/O, file I/O or direct Account Server access inside the component.

## Acceptance

- A valid ChatInput updates exactly one sender component at the next fixed Tick.
- The resulting event and component state carry the same applied Tick.
- Direct writes from a network thread or after Entity destruction fail safely.
- Snapshot/Restore can observe the declared last-message fields without restoring Chat history.

## Boundary

No Chat history list, moderation, private channel, client-side state mutation or custom persistence subsystem.

\n