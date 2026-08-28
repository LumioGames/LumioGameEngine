# ADR-043: Loader Re-entry and Error Priority

- **Status**: Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (profile publisher), `LumioCoreEngine` (`loader` consumer)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; the ADR-038 machine set, its states and its transitions are unchanged)
- **Relation**: Completes the observable semantics of [ADR-019](ADR-019-loader-state-machine-package-identity.md); the machine itself stays frozen by [ADR-038](ADR-038-state-machine-descriptor.md).

## Context

ADR-019 froze the loader machine — `Uninitialized → Preflighting → Verified → Binding → ApiReady → Leased`, closing `Quiescing → Released`, with every pre-lease failure going to `FailedRolledBack` — and the process-level PackageIdentity latch. Three questions it left open are all questions a second implementation will answer differently:

1. **What happens after `FailedRolledBack`?** The state is terminal, so no transition leaves it. But a host that failed to load a package will obviously try again, and nothing says what "again" means.
2. **What happens after `Released`?** Same shape. V1 does not physically unload (No-Physical-Unload), so it is not obvious whether a later Acquire is a fresh load, a no-op, or an error.
3. **Which error is reported when a pre-lease failure rolls back?** A rollback produces at least two facts — the root cause (a missing symbol, an ABI mismatch) and the rollback itself (`PartialLoadRolledBack`). Two verifiers reporting different codes for the same failed load are both defensible today, which is exactly the problem.

## Decision

### 1. Terminal means terminal; retry is a new instance

`FailedRolledBack` and `Released` are terminal **for the loader instance**. No transition leaves them, and none is added — the ADR-038 descriptor is unchanged.

- **Retry after `FailedRolledBack`** begins a **new loader instance** at `Uninitialized`. It is not a transition, not a reset and not a resumption; the failed instance is finished and observably stays finished.
- **Acquire after `Released`** likewise begins a **new instance** at `Uninitialized`.

This is the whole answer to "re-entry", and it deliberately adds no state: a machine whose terminal states can be left is not a machine with terminal states. What survives across instances is the **process-level PackageIdentity latch** of ADR-019, not any part of the instance.

### 2. The latch is by identity, not by time

Once any instance reaches `Leased`, the process is latched to that PackageIdentity. For every later Acquire, in a new instance:

| Requested identity | Outcome |
| --- | --- |
| Equal to the latched identity | The **existing Lease** is returned. No second load, no second bind, no new `ApiReady`. Idempotent. |
| Different from the latched identity | `PackageIdentityConflict` (1023), including for a "compatible upgrade". |

**Concurrency resolves to the same rule.** Two concurrent Acquires do not race for an ordering: the first to reach `Leased` latches, and the other is then evaluated by the table above — same identity returns that same Lease, different identity is refused. First-success is therefore an outcome of the latch, not a separate rule, and no implementation needs to expose or agree on scheduling.

**Release does not clear the latch.** V1 does not physically unload, so a `Released` instance leaves the process still bound to its identity; an Acquire for a different identity after `Released` is still `PackageIdentityConflict`. A host that needs a different package needs a different process.

### 3. The reported error is the root cause, never the cleanup

When a pre-lease failure rolls back, the outward `ErrorCode` is the **root cause**. `PartialLoadRolledBack` (1028) reports that rollback happened; it is a **floor, not a winner**, and is reported only when no more specific cause is available.

Frozen total order — lower wins:

| Rank | `ErrorCode` | Numeric |
| --- | --- | --- |
| 1 | `PackageIdentityConflict` | 1023 |
| 2 | `NativeAbiMismatch` | 1004 |
| 3 | `SymbolMissing` | 1021 |
| 4 | `SymbolCollision` | 1022 |
| 5 | `CapabilityMissing` | 1020 |
| 6 | `TargetProfileMismatch` | 1019 |
| 7 | `LoaderOutOfMemory` | 1027 |
| 8 | `LoaderTimeout` | 1025 |
| 9 | `LoaderCancelled` | 1026 |
| 10 | `PartialLoadRolledBack` | 1028 |

The order is not a preference list. `PackageIdentityConflict` outranks everything because it is refused before the package is examined at all — there is nothing to diagnose. ABI and symbol causes precede capability and target causes because they describe the package as loaded rather than the host as configured, and a caller can act on the former. Resource and lifecycle outcomes (`OutOfMemory`, `Timeout`, `Cancelled`) rank below real defects because they describe the attempt, not the package: retrying can change them. `PartialLoadRolledBack` is last because "I cleaned up" is never the most useful thing to tell a caller.

The order is total, so two verifiers given the same set of simultaneous causes report the same code.

## Contract

`schemas/loader-profile.schema.json` (structural) plus `tools/lumio_contract.py` semantic rules:

- The re-entry rules equal the §1–§2 freeze, and the machine descriptor's terminal states still admit no outgoing transition (the existing ADR-038 rule already enforces this; §1 depends on it and must not silently diverge).
- `errorPriority` equals the §3 order exactly, and every entry names a registered `ErrorCode`.
- Every published re-entry and error-priority vector is **re-evaluated** by the gate: an acquire vector's outcome must be what §2 produces, and a failure vector's reported code must be the §3 minimum of its declared causes.

## Failure semantics

Unchanged from ADR-019: every failure maps to a registered stable `ErrorCode` and a pre-lease failure rolls back with the process still usable. This ADR only makes the reported code and the post-terminal behaviour deterministic.

## Alternatives

Adding `FailedRolledBack → Preflighting` and `Released → Preflighting` transitions was rejected. It would make the terminal states non-terminal, contradict the ADR-038 descriptor, and — worse — hide the fact that a retry re-runs preflight and trust verification from scratch. Modelling retry as a new instance keeps that visible.

Clearing the latch on `Release` was rejected: V1 does not physically unload, so clearing the latch would let a process claim an identity whose code is still mapped.

Ranking `PartialLoadRolledBack` first was rejected: it is the most common outcome of a failed load and the least informative, so ranking it first would make almost every failure report the same uninformative code.

Ordering by "first detected" was rejected: detection order is a property of an implementation's phase ordering, not of the contract, so two conformant loaders would disagree.

Writing the mutex, condition-variable or lock-ordering strategy into this ADR was rejected outright — the card forbids it and it would freeze an implementation detail as a public contract. §2 deliberately expresses concurrency as an identity rule with no scheduling content.

## Compatibility and migration

Additive. The ADR-038 machine set, its states, its transitions and `failure-bundle.coreEngine.loaderState` are all unchanged; no existing required field, enum or ID changes meaning, so the `LGE-V1.4-2026-08-27` baseline id and every repository mirror stay valid. `LumioCoreEngine` deletes any local decision about post-terminal behaviour or error precedence and consumes the published profile.

## Verification

Fixtures `loader/profile` (positive), `loader/error-priority-order` (an `errorPriority` that is not the §3 freeze), `loader/vector-outcome-mismatch` (an acquire vector whose outcome is not what §2 produces) and `loader/priority-vector-mismatch` (a failure vector whose reported code is not the §3 minimum of its causes). The vector set covers: first Acquire; concurrent Acquire with the same identity; concurrent Acquire with a different identity; Acquire after `Released` with the same identity; Acquire after `Released` with a different identity; retry after `FailedRolledBack`; a Binding failure whose root cause outranks the rollback marker; and a failure whose only fact is the rollback.
