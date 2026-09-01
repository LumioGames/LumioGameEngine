# [NativeCore] Fixed Tick/Frame Timer Manager

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-TIMER`
- Target repository: `C:/Work/LumioGames/LumioNativeCore`
- Category: native infrastructure / contract
- Module: Timer Manager
- Wave: 1
- Priority: P1
- Risk: high
- Readiness: conditional until the Native timer ABI is frozen

## Background

The reconnect deadline belongs to Host monotonic time, while gameplay timers need a shared deterministic Tick/Frame foundation for Server and Client.

## Goal

Provide the first bounded Native Timer Manager surface without allowing arbitrary hot Gameplay callbacks.

## Preconditions

- Existing monotonic/timer host primitives: `R-00272`, `R-00212`.
- Native ABI handle and callback constraints: `ADR-006`, `ADR-017`.

## Requirements

- Support fixed Tick/Frame one-shot and repeating timers, cancellation and scope/generation validation.
- Use opaque generation-safe TimerHandle values.
- Permit only controlled CallbackSlot callbacks with lifecycle scope and explicit failure behavior.
- Reject late completions after cancellation, scope close or generation change.
- Expose shared Server/Client infrastructure through adapters; do not add a second timer truth in gameplay.

## Acceptance

- One-shot, repeating, cancel and stale-handle cases have deterministic fixtures.
- CallbackSlot lifecycle and failure boundaries are observable and safe.
- Server reconnect deadline is demonstrably separate from gameplay Tick/Frame TimerManager scheduling.

## Boundary

No full GameTime/RealTime/Scaled/Unscaled matrix, arbitrary function pointers, C# delegate ABI or direct hot Gameplay callbacks.

\n