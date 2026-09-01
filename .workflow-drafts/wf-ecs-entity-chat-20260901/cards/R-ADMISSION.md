# [Game Server] Room Admission and Player/Bot Entity Lifecycle

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-ADMISSION`
- Target repository: `C:/Work/LumioGames/LumioServer`
- Category: server / world
- Module: Room admission and ECS lifecycle
- Wave: 2
- Priority: P0
- Risk: high
- Readiness: conditional

## Background

Game Server must create Game ECS Entities only after Account Server admission and Room entry. Entity kind is derived from the authenticated login name, not from a client-controlled arbitrary type field.

## Goal

Create isolated Room Worlds and enforce the account-to-Entity lifecycle for `PlayerEntity` and `BotEntity`.

## Preconditions

- Account login/admission: `R-ACCOUNT`.
- ECS identity/storage: `R-00149`.
- WorldSlot/admission primitives: `R-00235`, `R-00277`.

## Requirements

- Accept only a valid Account Server admission credential.
- Enforce one active Room per AccountId.
- Classify login names matching `Bot` plus decimal digits as BotEntity; other normal names create PlayerEntity.
- Require the authenticated Bot-tool context when admitting a Bot-numbered account; do not allow a normal client to claim BotEntity by naming alone.
- Store AccountId on the Game Entity as an identity attribute, never as an AccountEntity object reference.
- Create exactly 100 BotEntity instances for Bot01-Bot100 and one PlayerEntity for the Browser in the main Room.
- Keep Room Worlds isolated; no cross-Room Entity resolution or Chat delivery.

## Acceptance

- Main admission trace proves 100 BotEntity plus 1 PlayerEntity in one Room.
- Duplicate Room admission for one AccountId is rejected or idempotently returns its existing binding according to the frozen session contract.
- A second Room cannot see or address the first Room's Entities.
- A normal Browser/client cannot enter as BotEntity using a Bot-numbered account without the Bot-tool admission context.

## Boundary

Do not add multi-World authority transfer, cross-server migration or client-selected EntityType.
\n