# Architecture-owner confirmation — Voxel P0 gates (D-013 / VOX-D-001..004)

Confirmation id: `LGE-V1.4-VOX-D-P0-2026-08-28`

This record is the architecture-owner freeze for LumioVoxelEngine P0 decision gates. It does **not** change BaselineId, Schema, ID Registry, ABI, or generated Artifact five-tuples. Public numeric profile columns are **not** added to `config-table`.

## Required fields

- Date: 2026-08-28
- Owner: LumioGameEngineArchitecture / Architecture owner
- Baseline: `LGE-V1.4-2026-08-27`
- Affected ADR/Manifest: ADR-024 (unchanged wire contract), ADR-025 (unchanged receipt protocol), `docs/architecture/DECISIONS_PENDING.md` D-013 (this confirmation). No new ADR. No Manifest numeric column.

## Selected value

LGE-V1.4 **does not generate a public Voxel numeric profile**. ADR-024 remains the frozen wire contract (i32 chunk coordinates, canonical ChunkId `c:x:y:z`, u32 block value, versioned hashed pages, `Dense`/`Sparse` envelopes, `None`/`Zstd`/`Lz4` codec *names*). Consumers must not depend on concrete chunk extent, world bound, page size, query batch limit, or receipt-table capacity.

Voxel implementation may proceed with these **adapter-internal families** (not Schema defaults, not BaselineId changes):

| Gate | Selected internal family | Meaning |
| --- | --- | --- |
| VOX-D-001 | `IsolatedCubicExtentFamily` | Chunk extent is adapter-internal. Port/ABI/generated config must not expose a concrete dimension. |
| VOX-D-002 | `DenseUncompressedAdapter` | V1 default *adapter* is uncompressed dense pages. Codec envelope names already exist in ADR-024; V1 default codec identity is `None`. No new crate. |
| VOX-D-003 | `StrictAdmissionBudgetFamily` | Hard admission. Full-load *action* stays the generated error `QueueFull` / `BudgetExceeded` (ADR-0005 matrix in Voxel). Batch/cost *numbers* stay adapter-internal and are not Schema columns. |
| VOX-D-004 | `GenerationBoundLeaseFamily` | Reservation lease is bound to instance generation / revision / tick (`leaseDeadlineTick` already in `voxel-mutation-receipt`), never wall clock. Receipt table capacity stays adapter-internal. |

`VoxelConfigSnapshot::from_generated` may treat DecisionEvidence for VOX-D-001..004 with `approvalStatus=approved` citing this confirmation. The snapshot still must not grow public fields named `chunk_size`, `page_size`, `batch_limit`, or `lease_ms`.

## Rejected alternatives

- `CoupledPageAxisExtentFamily` as a public Schema coupling of page size to per-axis extent.
- `ExternalLz4PageAdapter` / `ExternalZstdPageAdapter` as V1 default (license audit pending; unaudited crates forbidden). `PaletteRleAdapter` as V1 default.
- `ContinuationFirstBudgetFamily` / `ExplicitMissingQuotaFamily` as generated public defaults.
- `WallClockLeaseFamily` (non-deterministic across replays).
- Copying V1.3 `DECISION_GATES.md` unapproved numbers into production defaults.
- Any handwritten Schema field, ID, error code, or BaselineId bump.
- Treating unexecuted three-repeat `link.exe` traces as a numeric SLA.

## What this does not freeze

- VOX-D-005..008 (D-014 and later) remain unapproved.
- D-009 protocol-dispatch and D-011 auth wire stay blocked.
- Concrete memory/latency SLOs. Those still require a host that can link measurement binaries.

## Voxel follow-up

Voxel `docs/evidence/decision-gates/VOX-D-00{1,2,3,4}-*.md` and `benchmarks/decision_gates/*.rs` `approval_status()` must cite this confirmation id. Public config, if ever generated, is produced by this architecture repository — not handwritten in Voxel.
