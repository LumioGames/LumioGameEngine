# ADR-046: Native Kernel Status Band

- **Status**: Historical · Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (`ErrorCode` namespace owner); `LumioNativeCore` and `LumioCoreEngine` (status producers and consumers)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; ID Registry values only — no schema, required field or enum changes)
- **Relation**: makes the `lumio_status_t` clause of [ADR-040](ADR-040-root-abi-generated-bundle.md) §3 satisfiable at the kernel boundary, and closes the "both map to stable Error Codes" / "buffer-too-small returns required size" clauses of [ADR-006](ADR-006-native-managed-abi.md) that had no registered value.

## Context

ADR-040 §3 froze `lumio_status_t` as `int32_t` carrying an ID Registry `ErrorCode` numeric, with `0` reserved for success and no other value reused. That makes the `ErrorCode` namespace the **entire** public status space of the Root ABI: any value a slot function returns is either `0` or a registered numeric.

Measured on `origin/main = f5ce0e3` (2026-08-28T12:23:28Z), the namespace held 43 values, all of them loader, manifest, session or voxel semantics. `InvalidHandle` (1029) was the only one a kernel slot call could plausibly return. Several behaviors that ADR-006 specifies in prose had no value at all:

- "Buffer-too-small returns required size" — no `BufferTooSmall`.
- "Rust catches panic; Managed entry points catch exceptions; both map to stable Error Codes" — no stable code for a caught panic.
- "Cancellation, timeout and completion after World destruction are terminal and cannot write state" — `LoaderCancelled` (1026) and `LoaderTimeout` (1025) are loader-scoped and say nothing about a kernel call.

`LumioNativeCore` (`origin/main` `03d6bd7`) independently froze 13 `ErrorCategory` values and could name only one of them publicly. This is not a downstream convenience request. ADR-006 composes NativeCore's provider API tables into the single root table that `LumioCoreEngine` exports; the caller invokes NativeCore's own function pointers and receives its `lumio_status_t` directly. There is no interposition layer that could translate a repository-private code into a registered one, so an unregistered return value would be an unregistered **public** numeric on the wire of the Root ABI.

## Decision

### 1. Ten new `ErrorCode` values — the kernel band

Allocated contiguously after the existing high-water mark (1043), `status: Active`, `since: V1`, owner `Architecture`:

| Numeric | Id | Condition |
| --- | --- | --- |
| 1044 | `InvalidArgument` | A parameter fails a documented precondition that is not a handle, capability or buffer-size problem. |
| 1045 | `WrongContext` | A handle is structurally valid but its `context` word does not match the context the call was made on. |
| 1046 | `BufferTooSmall` | A `buffer:out` / `buffer:inout` argument is smaller than the required size. The callee writes the required size into the buffer's `capacity` field and writes no payload — this is the "returns required size" convention of ADR-006. |
| 1047 | `CapacityExceeded` | A fixed-capacity structure (handle table, arena, slot table) is full. Distinct from `QueueFull` (1036, a bounded queue) and `BudgetExceeded` (1035, a policy budget). |
| 1048 | `Cancelled` | The operation was cancelled before completion. Terminal; the call wrote no observable state. |
| 1049 | `TimedOut` | The operation exceeded its deadline. Terminal; the call wrote no observable state. |
| 1050 | `ContextClosing` | The context is draining and refuses new work; outstanding work may still complete. |
| 1051 | `ContextDestroyed` | The context no longer exists; the call completed after destruction and cannot write state. |
| 1052 | `PanicBoundary` | A panic or exception was caught at the ABI boundary. The process survives; the slot result is unproven and defaults to `FaultClass` `SlotStateUnproven` unless the callee attests otherwise (ADR-006). |
| 1053 | `InternalInvariant` | The callee detected a violation of its own invariant. Always a defect in the callee, never a caller error. |

`ContextClosing` and `ContextDestroyed` stay separate because the caller's correct response differs: closing is a retry-elsewhere condition, destroyed is terminal.

### 2. Three kernel categories are served by existing values

They do not receive new numerics; a redundant public numeric is permanent and never recoverable.

| Kernel category | Registered value | Why |
| --- | --- | --- |
| Invalid handle | `InvalidHandle` (1029) | Already frozen; covers a structurally invalid handle and a generation mismatch. |
| Already released | `InvalidHandle` (1029) on any use path; `HandleDoubleRelease` (1030) on the release path | The Index+Generation encoding of ADR-006 makes "used after release" a generation mismatch, and 1030 is exactly "released twice". |
| Capability unavailable | `CapabilityMissing` (1020) | Same predicate — a required capability is not provided. The value is not loader-scoped by its definition. |

### 3. The status range is now a gate

Every registered `ErrorCode` numeric must fit `lumio_status_t`. The registry schema already forbids `0` and negatives (`minimum: 1`) but permitted up to `4294967295`, which `int32_t` cannot carry. `tools/lumio_contract.py` now rejects any `ErrorCode` numeric above `2147483647`. Other namespaces are unaffected — they do not cross the Root ABI as a status.

### 4. Allocation stays with the architecture source

The `ErrorCode` namespace owner remains `Architecture`. `LumioNativeCore` never allocates a numeric, never publishes a second table, and never returns an unregistered non-zero status from a slot function. Its internal category enum may keep any shape it likes as long as the value it converts to at the boundary is one of the above.

## Contract

`ids/index.json` (and its byte-identical positive fixture `fixtures/valid/id-registry.json`) — `schemas/id-registry.schema.json` is unchanged. Semantic rules in `tools/lumio_contract.py`:

- ids and numerics stay unique within a namespace (existing);
- an `ErrorCode` numeric must not exceed `2147483647` (new, this ADR).

The generated `ContractTypes` artifacts publish the id list (`StableErrorIds` / `STABLE_ERROR_IDS`) and therefore grow by ten entries. **The numerics are not published in any generated artifact** — `ids/index.json` remains their only authority. That is a pre-existing property of the artifact set, not something this ADR changes.

## Failure semantics

`0` is success. Every other value crossing `lumio_status_t` is a registered `ErrorCode` numeric. A callee that has no registered value for a condition must fail the build, not invent one — inventing is precisely the "private de-facto ABI" that ADR-017 and ADR-040 exist to prevent.

`BufferTooSmall` is the only value in this band with a mandatory side effect: the required size is written into the buffer's `capacity` field and no payload bytes are written. `Cancelled`, `TimedOut` and `ContextDestroyed` are terminal and guarantee no observable state was written. `PanicBoundary` guarantees the opposite — the state is *unproven*, which is why it maps to `SlotStateUnproven` rather than to a clean failure.

## Alternatives

Declaring the twelve categories repository-internal was rejected: ADR-040 §3 leaves no room for a private status value, and the composition model of ADR-006 gives no layer that could translate one. The result would be unregistered public numerics on the Root ABI — the exact defect the registry exists to prevent.

A separate `KernelStatus` namespace was rejected: `lumio_status_t` is one flat `int32` space with no discriminator field and no room to add one. Two namespaces would need a disjointness rule that is harder to enforce than a single contiguous namespace.

Allocating all thirteen categories was rejected: 1020, 1029 and 1030 already carry three of them. A numeric is never reused once published, so a redundant allocation is a permanent wart with no offsetting benefit.

Starting a new band at 2000 or 1100 was rejected: the namespace is contiguous from 1001, nothing reads a band prefix, and a gap would imply a banding rule that no consumer could rely on.

Widening `lumio_status_t` to `int64` so the schema's `uint32` range fits was rejected: it would change a frozen ABI layout for a range no value approaches.

## Compatibility and migration

Additive. No schema, required field, enum or existing numeric changes meaning, so `LGE-V1.4-2026-08-27` and every repository mirror stay valid. `ids/index.json` is inside the five projected prefixes that `LumioCoreEngine`'s `sync-architecture.sh` already mirrors, so this ADR needs no projection-rule change (it does not, on its own, close the `architecture.lock` upgrade described as D-2 in the 2026-08-28 gate escalations).

The generated `ContractTypes` artifacts gain ten id strings and therefore new `outputHash` values; `compilerHash` also moves because the validator source changed. Consumers that pin an `outputHash` re-pin. Consumers that read only the id list see ten additions and no removals.

`LumioNativeCore` replaces the placeholder in its `T-error-03` card: the public mapping now exists, and the card's wording becomes "map the frozen 13 `ErrorCategory` values onto the ten kernel-band numerics plus 1020/1029/1030 per §1–§2", not "establish a mapping to something that may not exist".

## Verification

- `ids/registry` (positive, `fixtures/valid/id-registry.json`) — the registry with 53 `ErrorCode` values validates, and `tools/lumio_contract.py` enforces that `ids/index.json` is byte-identical to it.
- `ids/duplicate` (negative, existing) — numerics stay unique within a namespace.
- `ids/status-range` (negative, new, `fixtures/invalid/id-registry-status-range.json`) — an `ErrorCode` numeric of `2147483648` is rejected because it cannot cross `lumio_status_t`.
