# ADR-007: Contract Toolchain, ID Namespace and Dependency DAG

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameEngineArchitecture` (registry/tool contract), owning implementation repositories (source schemas)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

Handwritten duplicate MessageIds, serializers and bindings create drift across seven repositories. The architecture source must be authoritative without becoming a runtime dependency.

## Decision

The architecture source publishes versioned Schema, ID Namespace, compiler input/output hash, CLI result and Failure Bundle contracts. `schemas/index.json` is the registry. Generated artifacts flow from Native/Voxel schemas to CoreEngine, Runtime/Host and Game release packages; reverse source dependencies are rejected. Generated files are read-only outputs and are reproducible in a clean checkout.

The bootstrap validator is `tools/lumio_contract.py`; mature upstream validators are preferred in implementation builds and are isolated behind the tool adapter.

## Contract

Every generated artifact records BaselineId, schema/compiler version, input hash, output hash and owner. IDs are namespaced by domain and release; collisions fail the build. CLI commands return stable machine-readable exit codes and eventually a result JSON schema.

## Failure semantics

Missing schema, unresolved reference, duplicate id, dependency cycle, stale baseline or non-reproducible output blocks publication. A local fallback validator may report only its supported subset and must not claim cryptographic signing or code generation.

## Alternatives

Keeping a copy in every implementation repository was rejected because drift is undetectable. Runtime reflection as the contract source was rejected for startup, determinism and AOT reasons. Embedding generators in CoreEngine/Game was rejected because it creates ownership cycles.

## Compatibility and migration

Schema breaking changes increment SchemaEpoch and require new fixtures and generated artifacts. Old generated packages remain consumable only under their declared ReleaseManifest.

## Verification

Run the registry, all fixture validation, duplicate-id and dependency-cycle checks in a clean environment; compare canonical output and input/output hashes.
