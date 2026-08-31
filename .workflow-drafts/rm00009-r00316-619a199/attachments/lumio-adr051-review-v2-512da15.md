# R-00316 / ADR-051 Independent Review v2

## Review Point

- Target commit: `512da155f6480a9989f6f01815bd1e0aa83f9b54`
- Parent-relative range: `23b9e019e3254ad3830b86b3fc6a8fd7cb22560e..512da155f6480a9989f6f01815bd1e0aa83f9b54`
- Authoritative brief: `C:/Users/g923/orca/reports/lumio-adr051-review-brief-f317b92.md`
- Repair evidence: `C:/Temp/R-00316-fix-report.md`
- Review package: `C:/Users/g923/orca/reports/lumio-adr051-review-package-512da15.diff`

## Verdict

**Approved.** The repaired implementation satisfies the four native acceptance criteria and independently closes all prior P1-1 through P1-6 and P2-1 findings. No remaining P0, P1, or P2 correctness, scope, or contract issue was found in the complete supplied range.

## Spec Compliance

1. **Namespace authorization:** `ids/index.json`, its valid fixture mirror, and ADR-051 register exactly one `LumioConfig` namespace with `LumioConfig` ownership, `NamespaceOwner` authority, `DomainAutonomousSerial`, inclusive `40000..49999`, `Never` reuse, and Architecture spot-check metadata. `tools/lumio_contract.py:2487-2515` rejects namespace/range/allocation metadata drift; the ID registry semantic checks reject duplicate namespace names and non-frozen LumioConfig metadata.
2. **Identity layers:** `schemas/config-id-registry.schema.json:56-175` and the ADR define source name/aliases, permanent stable numeric ID, ephemeral `revisionOrdinal`, and column ordinals. Persistent declarations are closed to the six stable fields and sole ephemeral revision field; `tools/lumio_contract.py:2578-2581`, `2609-2615`, and `2634-2658` reject row/tombstone ordinals, row persistence flags, inverted declarations, and non-empty persisted ordinals.
3. **Allocation and compatibility:** `tools/lumio_contract.py:2793-2842` enforces patch merge order, strictly increasing unique assignments, authorized range, and tombstone exclusion. `:2861-2900` enforces one-to-one active-row/AddRow/assignment atoms; `:2986-3032` enforces bidirectional rename/delete/column-rename history links. `:2902-2926` checks replacement IDs against the complete consumed set, independent of tombstone list order.
4. **Stable failures and evidence:** the five ErrorCode values are registered in both ID registries and generated catalogs. Negative fixtures cover out-of-range, duplicate/reused IDs, persisted ordinals, preallocation, missing/duplicate history, unknown column endpoints, malformed persistence, replacement ordering, and duplicate tombstone identity. Full validation reports all registered fixtures passing.

## Strengths

- The repair directly addresses every prior review finding with focused tests and fail-closed collection handling.
- Column rename resolution is alias-aware and requires both published endpoints while preserving the ordinal (`tools/lumio_contract.py:2660-2719`).
- Tombstone replacement validation is deterministic regardless of record order and distinguishes duplicate logical tombstone identity from numeric reuse in `config_source_errors` (`tools/lumio_contract.py:880-932`).
- The valid tombstone-only snapshot is structurally representable (`rows` has no minimum) and semantically accepted.
- Changes remain scoped to ADR-051 docs, indexes, schemas, fixtures/tests, validator semantics, and regenerated package catalogs/bindings. No allocator/runtime implementation or Workflow access was added.

## Findings

### P0

None.

### P1

None. Prior P1-1 (direct/unlinked identities), P1-2 (unknown column endpoints), P1-3 (persistence inversion/row flag), P1-4 (order-dependent replacement), P1-5 (empty-row snapshot), and P1-6 (malformed-input crash) were reproduced as fixed by the new focused tests and independent probes.

### P2

None. Prior P2-1 stable-code confusion is fixed: duplicate tombstone row identity emits `ConfigDuplicateStableId`, while duplicate/reused tombstone numeric IDs emit `ConfigTombstoneReuse` (`tools/lumio_contract.py:923-932`).

## Focused Checks Run

- `python3 -m unittest tools.config_id_test tools.config_patch_test` -> **54 tests, 54 passed**.
- `python3 -m py_compile tools/lumio_contract.py tools/config_id_test.py tools/config_patch_test.py` -> exit 0.
- `python3 tools/lumio_contract.py validate` -> **Validated 240 fixture(s), 0 failure(s)**.
- `git diff --check 23b9e019..512da155` -> exit 0.
- `git merge-base --is-ancestor 23b9e019 512da155` -> pass; prerequisite `398a2dd5` ancestry -> pass.
- Accepted parent blobs for ADR-007, ADR-010, ADR-033, ADR-034, and ADR-050 are byte-identical between parent and target.
- Direct adversarial probes reject direct rows, unassigned AddRow/DeleteRow/RenameRow/RenameColumn records, both unknown column-rename endpoints, malformed persistence collections, and later-entry tombstone replacement reuse.

The repair evidence additionally reports clean-materialization `spec-lint`, generator parity (70/70), Rust checks, six .NET builds, and cross-language KAT agreement. Those reported commands were not rerun here because the focused contract gates and supplied evidence were sufficient for this read-only re-review.

## Evidence and Residual Risk

The implementer’s historical RED transcript is documented in `C:/Temp/R-00316-fix-report.md` but is not a separately captured artifact that can be independently replayed from this review checkout. This is an evidence limitation only; final focused tests, full fixture validation, ancestry, scope, and parent-byte checks are independently reproducible and pass. The ordinary Windows checkout’s tracked-link materialization limitation remains environmental and is covered by the supplied junction-enabled clean-materialization evidence, not a source defect.

## Task Quality

**Approved.** The implementation is complete, scoped, reproducible at the contract-gate level, and suitable for coordinator acceptance; no fixes are required by this review.
