# [Original Requirement] Formal ECS Entity and Chat Vertical Slice

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1`
- Category: product integration
- Module: ECS / Account / Room / Chat
- Wave: 0
- Priority: P0
- Risk: high
- Readiness: ready (source record)

## Background

Hello World proved process connectivity and message display, but it did not create formal Game ECS Entities from authenticated accounts or exercise Entity identity, Attribute Query, component state, reconnect lifecycle or ECS persistence.

## Goal

Define and deliver the first formal vertical slice with one central Account Server, multiple Room boundaries, 100 BotEntity instances, one PlayerEntity Browser observer, generic NetEntityId binding/query and a ChatComponent live event path.

## Acceptance

- The slice has an executable requirement source, decision log, dependency DAG and explicit non-goals.
- The main scenario is unambiguously 100 BotEntity plus 1 PlayerEntity = 101 Game ECS Entities.
- Existing Hello World and module-room objects remain unchanged and are referenced as prerequisites.

## Boundary

This is a planning/source record. It does not itself implement code, move existing requirements or create a new milestone.

\n