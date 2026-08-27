# ADR-011: Logging, Metrics, Trace, Audit and Failure Bundle

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioServer`/`LumioClient` Host adapters; all repositories emit domain events
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

Multi-threaded asynchronous logs can reorder or disappear, while audit, transaction recovery and deterministic replay require stronger durability than diagnostic output. The simulation thread must not block on a slow sink.

## Decision

Rust and C# use mature logging frameworks behind adapters and emit one Lumio Event Schema. Diagnostic events use bounded asynchronous queues and may be sampled. Audit, TxnJournal and CommandLog use independent durable queues with distinct owners: the Host observability component owns the Audit queue, its sinks and Failure Bundle assembly; the Host persistence component owns WAL, TxnJournal and CommandLog. The two owners may share low-level IO primitives but never share queue state or acknowledgment paths. Saturation of a durable queue stops admission or enters maintenance rather than silently losing records. Error/Fatal has synchronous emergency fallback; `EmergencySync` durability is reserved for Error/Fatal severity. Metrics, Trace and Failure Bundle remain separate products with shared correlation fields.

Durable categories return explicit durable acknowledgments. An orchestration step that requires persisted evidence — for example the maintenance persist step — must wait for the persistence commit acknowledgment and the Audit durable acknowledgment as two independent completions; neither implies the other.

Every event declares `correlation.scope`, the deepest identity tier the event legitimately possesses: `Process`, `Release`, `Session`, `World` or `Txn`. Base fields (ProductId, GameReleaseId, TraceId, ProducerId, per-producer EventSeq) are always required. SessionId, WorldId/TickId and TxnId are required only at their scope and must not be fabricated for events that occur before those objects exist, such as process startup, manifest validation or authentication rejects; Snapshot/Entity fields are added when applicable. Global cross-thread order is not promised; per-producer sequence plus Tick is the reconstruction contract.

Failure Bundle assembly has one owner, the Host observability component. Evidence providers continuously publish immutable evidence snapshots or references during normal operation; the assembler reads those published snapshots and never calls back into a faulted or destroyed module at assembly time. A provider that is missing or exceeds its budget yields a partial bundle that records the missing providers. The crash path writes a crash-safe minimal evidence set that is completed on next start.

## Contract

`logging-event.schema.json` defines category, severity, mandatory durability and scoped correlation. Failure Bundle references a verified Manifest, Snapshot and artifact hashes.

## Failure semantics

Diagnostic queue overflow is counted and sampled by policy. Durable queue overflow rejects new commands or transitions the target Pool to maintenance. Sink failures produce a Failure Bundle and never masquerade as successful persistence.

## Alternatives

One unbounded event bus was rejected for memory and failure-domain coupling. Treating logs as a transaction journal was rejected because diagnostic retention and recovery durability differ.

## Compatibility and migration

Adding optional event fields is compatible; changing correlation names, category meaning or durability requires a schema version and sink migration. Sensitive fields are redacted before enqueue.

## Verification

Run audit, queue-full, sink-failure, multi-thread ordering, emergency-fallback and Failure Bundle replay tests with the logging fixtures, including the startup-phase and auth-reject positive fixtures and the missing-durability and fabricated-scope failure fixtures.
