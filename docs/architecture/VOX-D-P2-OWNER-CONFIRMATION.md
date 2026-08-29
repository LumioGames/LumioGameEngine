# Architecture-owner confirmation — Voxel P2 gates (D-014 / VOX-D-005..008)

Confirmation id: `LGE-V1.4-VOX-D-P2-2026-08-29`

This record is the architecture-owner adjudication for LumioVoxelEngine P2 decision gates, issued on delegated authority (owner delegated the adjudication to the coordinating session on 2026-08-29; delegation recorded in the session ledger and R-00257). It does **not** change BaselineId, Schema, ID Registry, ABI, or generated Artifact five-tuples. Public numeric profile columns are **not** added to `config-table`.

Evidence basis: LumioVoxelEngine `origin/main` `d4ee9fc` (PR #4) — the four gate seams executed on linking hosts (`x86_64-apple-darwin` Rosetta + `aarch64-apple-darwin`, byte-identical legs, three-run process-level diff clean), with the prior defective-SHA-256 numbers forensically reproduced and superseded (VOX-D-006 §8.1). Measurement precondition of D-12 (2026-08-28 escalation record) is satisfied at the harness layer.

## Required fields

- Date: 2026-08-29
- Owner: LumioGameEngineArchitecture / Architecture owner (delegated)
- Baseline: `LGE-V1.4-2026-08-27`
- Affected ADR/Manifest: ADR-035/036 (unchanged wire semantics), `docs/architecture/DECISIONS_PENDING.md` D-014 (this confirmation). No new ADR. No Manifest numeric column.

## Selected value

LGE-V1.4 **does not generate a public Voxel P2 numeric profile, and no strategy candidate is selected**. What the harness evidence supports is confirmed as **binding invariants**; what it cannot rank is confirmed as **adapter-internal with a defined unlock condition**. Candidate sets in each gate's §3 stay open — none excluded, none preferred.

| Gate | Confirmed binding invariants (family level) | Deferred, adapter-internal (unlock condition) |
| --- | --- | --- |
| VOX-D-005 capture | Any landing capture candidate must satisfy same-cut three-run byte determinism; a `Ready` claim behind an expired pin publishes **nothing** (empty committed set — the measured stop-condition semantics). | Pin-vs-COW hold strategy, sub-chunk diff granularity, materialize rule, pin budget. Unlock: a production snapshot encoder exists and memory-amplification / encoded-bytes / write-tail axes are measured. |
| VOX-D-006 streaming | ADR-036 durability-ack fence and residency shapes reaffirmed; mapped faults are **unrecoverable after a visible write**; fence replay must be byte-deterministic. | Priority scoring, concurrency, queue capacity, backpressure thresholds, eviction scoring/hysteresis. Unlock: a production streaming coordinator exists and burst/latency/watermark axes are measured. |
| VOX-D-007 spatial | Cache keys must be **identity-complete**: World identity and Revision are mandatory key components (coordinate-only keys are rejected — measured stale-hit hazard); cancel/exception paths never populate the cache. | Kernel selection. `unaudited-oss-kernel` stays held out (license audit precondition). `nativecore-spatial-adapter` stays `pending`: requires a **published NativeCore kernel artifact** plus owner approval of its hash (dependency gap — no amount of re-running this seam closes it). |
| VOX-D-008 migration | Host owns DAG orchestration, fsync, and Active-pointer swap (never this crate); node crash leaves **only** an unconfirmed leftover candidate; faults after visible write are unrecoverable. | Node plan granularity, checkpoint rule, memory budget, production `toolVersion`. Unlock: a world migrator with a production corpus measures the §5.3 stop thresholds. |

Voxel `DecisionEvidence` for VOX-D-005..008 may record `approvalStatus=approved` **citing this confirmation id**. Approved means: the owner decision on the open fields is made as above — it does **not** mean any number is frozen. `numeric_policy_frozen()` stays `false`; proposals' numeric fields stay unset.

## Rejected alternatives

- Freezing any pin budget, priority weight, concurrency, capacity, backpressure threshold, or node budget from harness determinism hashes — the seams measure determinism and fault semantics, not cost.
- Ranking or selecting a capture/streaming/migration candidate without production-cost data.
- Coordinate-only cache key families (measured stale-hit hazard without World + Revision).
- Recording an unpublished / un-audited NativeCore kernel artifact hash as a production default.
- Treating seam trace digests as throughput or latency SLAs.
- Keeping the gates `blocked` on a measurement precondition that is now satisfied — status must track fact.
- Any handwritten Schema field, ID, error code, or BaselineId bump.

## What this does not freeze

- Every numeric axis named in the Deferred column above.
- D-009 protocol-dispatch and D-011 auth wire stay blocked; D-016 layout-Golden gating unchanged.
- The VOX-D-007 Reference-vs-NativeCore differential — blocked on a published NativeCore kernel artifact (tracked in the NativeCore S0 closeout ledger §4).
- Production memory/latency SLOs.

## Voxel follow-up

Voxel `docs/evidence/decision-gates/VOX-D-00{5,6,7,8}-*.md` §7 and front matter, the seams' `approval_status()`, and the `gate_remains_blocked`-class tests must be updated to cite this confirmation id (mirroring the P0 follow-up pattern). Public config, if ever generated, is produced by this architecture repository — not handwritten in Voxel.
