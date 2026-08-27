# ADR-016: Benchmark Workload, Tick Budget and Hardware Profile

- **Status**: Draft for Architecture Gate
- **Owner**: `LumioGameEngineArchitecture` (result contract), each implementation repository (measurements)
- **Baseline**: `LGE-V1.0-2026-08-27`

## Context

Capacity claims in the reviews were not tied to a repeatable workload. A player count without hardware, entity/chunk distribution, network rate and retention policy is not an engineering limit.

## Decision

Every benchmark records Release/Schema hash, Host Profile, OS/architecture, compiler, CPU/memory, TickRate, entity/chunk/AOI distribution, command/build frequency, network fault profile and duration. The first shared workload runs 1/10/25/50/100/150/200 Bots; 100 is a milestone target, not a capacity promise. Report Tick p50/p95/p99/max, CPU, RSS, GC, Native heap, queue depth, replication bytes/retransmits, FFI batch size, log throughput and persistence latency.

Budgets are declared per Processor, Queue, Session and Pool. Regression thresholds are relative to a versioned baseline and must distinguish deterministic simulation time from diagnostic/IO time.

## Contract

Benchmark result JSON references `ProductId`, `GameReleaseId`, `ManifestHash`, Scenario, WorkloadId, hardware profile, sample counts and Failure Bundle on abort. Results are comparable only when the required fields match.

## Failure semantics

Missing hardware/workload metadata invalidates a result. A budget breach fails the gate or marks the profile unsupported; it cannot be hidden by averaging. OOM, queue overflow, dropped durable log or missed Tick is a named failure.

## Alternatives

Using one developer laptop as a universal target was rejected. Reporting only average frame time was rejected because tail latency and queue pressure drive server failure. Treating 100 players as guaranteed capacity was rejected.

## Compatibility and migration

Workload schema changes create a new WorkloadId/version. Historical results remain immutable and are compared only within compatible hardware/profile groups.

## Verification

Add deterministic smoke and long soak runs to `lumio test perf`; store raw samples and a summarized Failure Bundle. The first implementation gate must publish a baseline before optimization work.
