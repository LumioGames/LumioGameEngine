# [Integration] 100 Bot plus Browser 101 Entity Acceptance

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-E2E`
- Target repository: `C:/Work/LumioGames/LumioGame` integration harness
- Category: integration / acceptance
- Module: formal ECS entity-chat slice
- Wave: 5
- Priority: P0
- Risk: high
- Readiness: conditional until all consumed contracts and implementations are available

## Background

The existing Hello World run demonstrated connectivity only. This acceptance must prove the formal account, entity, binding, query, ChatComponent, ReplicaWorld, reconnect and persistence path.

## Goal

Run the complete deterministic scenario with 100 Bot clients and one Browser client in one Room, plus smaller Room isolation and lifecycle failure cases.

## Preconditions

- Account Server: `R-ACCOUNT`.
- Room admission: `R-ADMISSION`.
- Entity binding/query: `R-IDENTITY`.
- Chat component and mapping: `R-CHAT-COMPONENT`, `R-CHAT-REPLICATION`.
- Client ReplicaWorld: `R-CLIENT`.
- Reconnect/expiry: `R-RECONNECT`.
- Snapshot/Restore: `R-PERSISTENCE`.
- E2E shell and fixture wiring: `R-00247`.

## Requirements

- Generate Bot01-Bot100 in a loop, login-or-register each using `123456`, and enter one Room.
- Login a normal Browser account through Account Server and enter the same Room.
- Verify 100 BotEntity plus 1 PlayerEntity = 101 Game ECS Entities.
- Verify self binding, server NetEntityId resolution and permitted Attribute Query.
- Send ChatInput text from Bots and Browser; verify reliable ordered event display in Browser.
- Exercise reconnect within five minutes, expiry/new Entity B and stale reference rejection.
- Exercise Snapshot/Restore of last-message fields without Chat history restore.
- Repeat the run twice and compare entity counts, event order and applied Tick evidence.

## Acceptance

- All 11 scenarios in the requirements source have executable evidence or an explicit blocked prerequisite.
- No client or server crosses Room boundaries, resurrects tombstoned IDs or bypasses Account Server.
- Evidence includes command lines, key output, lifecycle traces and failure cases; no claim is based on plan text alone.

## Boundary

No performance capacity claim beyond this measured 101-Entity scenario, no production auth claim and no changes to archived Hello World objects.

\n