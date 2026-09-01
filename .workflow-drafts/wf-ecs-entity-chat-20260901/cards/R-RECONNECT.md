# [Server] Five-minute Reconnect and Expiry Lifecycle

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-RECONNECT`
- Target repository: `C:/Work/LumioGames/LumioServer`
- Category: server / session lifecycle
- Module: reconnect retention
- Wave: 4
- Priority: P0
- Risk: high
- Readiness: conditional

## Background

Reconnect is a client connection event. The server Room continues normally; the client rebuilds its local view as if it logged in again.

## Goal

Keep a disconnected server Entity for five minutes, reject its input, rebind it after a fresh login, and destroy it after expiry without corrupting Room state.

## Preconditions

- Room admission/entity lifecycle: `R-ADMISSION`.
- Client ReplicaWorld: `R-CLIENT`.
- Host monotonic clock: `R-00212`.
- Existing reconnect/session orchestration: `R-00240`, `R-00279`.
- ECS persistence recovery: `R-PERSISTENCE`.

## Requirements

- Retain Entity A and keep the Room/server simulation running normally after disconnect.
- Reject only that account's input and expose explicit disconnected state to Room observers.
- Measure five minutes with the process-local monotonic Host clock; record Tick for audit only.
- Reconnect through fresh login and full handshake, then rebind A if within the window.
- Rebuild only the reconnecting client's ReplicaWorld from a full snapshot, clear its chat window and re-enable input after completion.
- Do not replay Chat history and do not roll back/rebuild the Room.
- On expiry destroy and tombstone A; later login creates B with the same AccountId and a different NetEntityId.

## Acceptance

- Input from the disconnected connection is rejected while other Room clients and entities continue.
- Reconnect within five minutes reuses A and produces a clean client ReplicaWorld.
- Expiry produces a tombstone; a later login creates B and stale A references never alias B.
- Process restart requires a new login and does not claim to preserve the old connection window.

## Boundary

No implicit Resume Token, server Room rollback, live migration or cross-process retention guarantee.

\n