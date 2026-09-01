# ADR-033: Config Typed Column Validation

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGame` (table source), `LumioGameRuntime` (validation/activation)
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-010](ADR-010-persistence-config.md). The Accepted ADR-010 Decision text is unchanged.

## Context

`columns` and `activation` were optional and row values were unconstrained, so a `u32` column could hold a string and still pass the gate.

## Decision

`columns` and `activation` are required. The validator type-checks every cell against its column: `bool`, `i32`, `i64`, `u32`, `u64`, `f32`, `f64`, `string`, `enum` (closed `enumValues`), `ref` (`refTarget` table id). Optional `minimum`/`maximum` bound numbers. Unknown columns are rejected. Missing required columns are rejected. Production activation still requires a signature.

Default values, if declared on a column, fill a missing optional cell before type-check; they cannot invent a required cell that is absent.

## Contract

`config-table.schema.json` plus `lumio_contract.py` dynamic checks.

## Failure semantics

Type mismatch, range overflow, missing ref, unknown column, missing required column and unsigned production activation are invalid.

## Alternatives

Generating a per-table JSON Schema at compile time remains allowed; the Architecture Gate must still enforce the same rules.

## Compatibility and migration

Required-field tightening in `LGE-V1.3-2026-08-27`.

## Verification

Fixtures `config/typed-table`, `config/type-mismatch`, `config/range-overflow`, `config/missing-ref`, `config/unknown-column`, `config/production-unsigned`.
