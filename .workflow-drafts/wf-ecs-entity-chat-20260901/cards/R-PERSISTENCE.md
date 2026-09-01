# [Runtime] ECS Snapshot and Restore for ChatComponent State

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-PERSISTENCE`
- Target repository: `C:/Work/LumioGames/LumioGameRuntime`
- Category: runtime / persistence
- Module: ECS Snapshot/Restore
- Wave: 3
- Priority: P0
- Risk: medium
- Readiness: conditional

## Background

ECS field attributes already distinguish persistence from replication. Chat must use that system rather than create a Chat-owned storage path.

## Goal

Persist and restore only each Entity's declared last-message text and logical Tick/Frame through the existing ECS snapshot pipeline.

## Preconditions

- ECS field persistence and Snapshot View: `R-00150`.
- Existing persistence adapter and recovery contract: `R-00231`.
- Canonical Snapshot/WAL architecture: `ADR-010`, `docs/specs/lumio-save-design-overview.md`.
- ChatComponent declarations: `R-CHAT-COMPONENT`.

## Requirements

- Mark only LastMessageText and LastMessageTick/Frame as persist-only ChatComponent fields.
- Use the existing canonical Snapshot/Restore and WAL/Command Log pipeline.
- Exclude ChatMessageEvent history and client chat-window contents.
- Restore fields deterministically and preserve AccountId/Entity identity rules.
- Reject malformed, stale or incompatible snapshots through existing recovery semantics.

## Acceptance

- A snapshot contains the last-message fields for eligible Entities.
- Restore reproduces those fields and no Chat event history.
- A process restart restores Room state through the existing recovery path; old connection bindings still require new login.

## Boundary

No per-message synchronous disk write, Chat history database, client cache persistence or parallel serializer.

\n