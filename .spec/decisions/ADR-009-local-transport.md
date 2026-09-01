# ADR-009: Local Transport Fidelity and Fault Injection

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioClient`/`LumioServer` adapters, `LumioGameRuntime` protocol semantics
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

LocalEmbedded is useful only if it exercises the same permissions, serialization, ordering and backpressure as a remote server. A direct method call would hide network bugs.

## Decision

LocalEmbedded uses the exact generated Envelope, Codec, size limits, ACK model, bounded queues and authorization path used by a split process. It may bypass OS sockets, TLS and kernel networking. A Fault Decorator can add delay, jitter, loss, duplication, reordering, disconnect, reconnect and QueueFull behavior with a deterministic seed.

## Contract

Transport adapters expose message batches and stable error classes: retryable, rejectable and fatal. Fault profiles are declared by Host Profile and recorded in Replay/Failure Bundle metadata.

## Failure semantics

Oversized or malformed messages are rejected before allocation. Queue exhaustion invokes the declared source policy; reliable messages are never silently dropped. A reconnect begins at Handshake/FullSnapshot unless a valid baseline is explicitly retained.

## Alternatives

Calling gameplay methods directly in LocalEmbedded was rejected. Reimplementing a second local protocol was rejected because it would drift from wire behavior.

## Compatibility and migration

Envelope and fault-profile changes require a protocol version. Transport implementation or TLS provider changes do not affect Gameplay when the adapter contract remains stable.

## Verification

Run LocalEmbedded and LocalSplitProcess with identical command streams and compare State Hash, ACK/Baseline behavior and Failure Bundles under every fault profile.
