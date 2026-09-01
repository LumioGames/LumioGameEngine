# ADR-023: Generated Contract Artifact Publication

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.2-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameEngineArchitecture` (toolchain and published artifacts)
- **Baseline**: `LGE-V1.2-2026-08-27`
- **Relation**: Refines [ADR-007](ADR-007-contract-toolchain.md) publication and dependency rules; does not supersede the registry or hash contract.

## Context

ADR-007 already forbids embedding generators in CoreEngine or Game and requires reproducible generated outputs. It does not name the publisher that LumioClient and LumioGame may both reference, nor does it forbid generated packages from depending on either implementation tree. Client core projects must not reference `LumioGame.ClientGameplay`; Game must not take a Client implementation dependency to read Mapping or Protocol artifacts.

## Decision

`LumioGameEngineArchitecture` is the sole publisher of generated contract artifacts. The toolchain in this repository (today `tools/lumio_contract.py`, later header/binding/validator generators) consumes:

- architecture-owned Schema, ID Registry and fixtures;
- domain *source* schemas owned by Game, Voxel or Native (data, not implementation projects).

Published artifacts are read-only generated packages. They record BaselineId, SchemaEpoch, compiler/input/output hashes. They have zero project or package dependency on LumioClient or LumioGame implementation assemblies. Both LumioClient and LumioGame (and Runtime/Server) may reference the same artifact package.

Artifact kinds in V1: `ProtocolPermissionValidator`, `MappingTable`, `CanonicalSerializer`, `LanguageBinding`, `ContractTypes`. Game-owned Mapper/Binding *implementations* stay in the Game Release Artifact and are injected through published ports; they are not this package.

Reverse source dependencies (implementation → handwritten duplicate MessageId/layout) remain rejected.

## Contract

`generated-contract-artifact.schema.json` records publisher, kind, hashes and `implementationDependencies`. The last list must be empty. `forbiddenDependents` names `LumioClient` and `LumioGame` implementation trees.

## Failure semantics

A package that depends on a Client or Game implementation project, lacks hashes, or claims a publisher other than this repository is rejected before publication. Consumers must not vendor a rewritten copy.

## Alternatives

Publishing artifacts from LumioGame was rejected because Client would import a Game implementation graph. Publishing from LumioClient was rejected because Game and Server could not share the package. Dual handwritten packages were rejected for drift.

## Compatibility and migration

Additive in `LGE-V1.2-2026-08-27`. Existing Mapping source schemas remain Game-owned; only the compiled artifact publication path is frozen here.

## Verification

Fixtures `gencfg/validator` (positive) and `gencfg/client-implementation-dep` (failure: implementation dependency on LumioClient).
