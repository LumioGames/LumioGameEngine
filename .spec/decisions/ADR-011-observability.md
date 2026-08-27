# ADR-011: Logging, Metrics, Trace, Audit and Failure Bundle

- **Status**: Draft for Architecture Gate
- **Owner**: `LumioServer`/`LumioClient` Host adapters; all repositories emit domain events
- **Baseline**: `LGE-V1.0-2026-08-27`

## Context

Multi-threaded asynchronous logs can reorder or disappear, while audit, transaction recovery and deterministic replay require stronger durability than diagnostic output. The simulation thread must not block on a slow sink.

## Decision

Rust and C# use mature logging frameworks behind adapters and emit one Lumio Event Schema. Diagnostic events use bounded asynchronous queues and may be sampled. Audit, TxnJournal and CommandLog use independent durable queues; saturation stops admission or enters maintenance rather than silently losing records. Error/Fatal has synchronous emergency fallback. Metrics, Trace and Failure Bundle remain separate products with shared correlation fields.

Every event carries ProductId, GameReleaseId, ReleasePoolId when known, SessionId, WorldId, TickId, TraceId, ProducerId and per-producer EventSeq; Txn/Snapshot/Entity fields are added when applicable. Global cross-thread order is not promised; per-producer sequence plus Tick is the reconstruction contract.

## Contract

`logging-event.schema.json` defines category, severity, durability and correlation. Failure Bundle references a verified Manifest, Snapshot and artifact hashes.

## Failure semantics

Diagnostic queue overflow is counted and sampled by policy. Durable queue overflow rejects new commands or transitions the target Pool to maintenance. Sink failures produce a Failure Bundle and never masquerade as successful persistence.

## Alternatives

One unbounded event bus was rejected for memory and failure-domain coupling. Treating logs as a transaction journal was rejected because diagnostic retention and recovery durability differ.

## Compatibility and migration

Adding optional event fields is compatible; changing correlation names, category meaning or durability requires a schema version and sink migration. Sensitive fields are redacted before enqueue.

## Verification

Run audit, queue-full, sink-failure, multi-thread ordering, emergency-fallback and Failure Bundle replay tests with the logging fixture pair.
