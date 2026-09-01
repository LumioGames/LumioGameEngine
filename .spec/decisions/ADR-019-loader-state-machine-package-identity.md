# ADR-019: Loader State Machine, PackageIdentity and Single-Process Lock

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioCoreEngine` (`loader`, `signing/runtime-verifier`)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

ADR-006 fixed `OnePackagePerProcess` but left "reject a second incompatible package" underspecified: identity by what? compared when? verified against which bytes? Version-string identity is forgeable and file-path identity is TOCTOU-racy (the file can change between verification and mapping).

## Decision

- **PackageIdentity** is the tuple (Manifest Digest, Artifact Set Digest, ABI Identity, TargetProfile Digest, Capability Set Digest). Two packages are the same iff every component matches. No version strings, no paths.
- **Single-process lock**: the first successfully leased load latches the process to that PackageIdentity. Any later load request with a different identity — even a "compatible upgrade" — fails with stable error `PackageIdentityConflict`. Re-requesting the identical identity returns the existing lease.
- **VerifiedPackageDescriptor** (`verified-package-descriptor.schema.json`) is the only trust input the Loader accepts: identity, `trustDecision` (`Trusted`/`Rejected` + `rejectReason`), `trustDomain`, verifier version/time and per-check results. Offline or CI verification conclusions are not load inputs; the runtime verifier re-verifies against the actual opened file handles (preflight on the same descriptors that will be mapped) to close the TOCTOU window.
- **State machine**: `Uninitialized → Preflighting → Verified → Binding → ApiReady → Leased`, with `Quiescing → Released` on shutdown and any pre-lease failure landing in `FailedRolledBack` (partial mappings unwound, process still usable, stable error reported). V1 performs no physical unload after `Leased` (`No-Physical-Unload`, pending revisit in V2); `Released` frees leases and API table views but may keep the image mapped.
- Every failure maps to a stable ErrorCode from the `ErrorCode` namespace (1007–1030 families: manifest, artifact, signature/trust, target/capability, symbol, identity conflict, timeout/cancel/OOM, rollback, handle).

## Contract

`verified-package-descriptor.schema.json`; `failure-bundle.schema.json` `coreEngine` block (`packageIdentity`, `loaderState`, `errorCode`, `trustDecision`) required for `CoreEngineLoad`/`SupplyChain` incidents; ErrorCode registrations 1007–1030 in `ids/index.json`.

## Failure semantics

Verification failure never reaches `Binding`. Binding/symbol failure rolls back to `FailedRolledBack` and reports `SymbolMissing`/`SymbolCollision`. A `Trusted` descriptor with any failed check is itself invalid (semantic rule), so a corrupted verifier output cannot authorize a load. Loader failures are `Process`-scope events: they carry no fabricated session or world identity.

## Alternatives

Per-load re-verification without latching was rejected: it permits two half-compatible packages in one process, which ADR-006 forbids. Path-based identity was rejected for TOCTOU. Allowing hot swap to a "newer compatible" package was rejected in V1: without physical unload the old image stays resident and symbol resolution becomes ambiguous.

## Compatibility and migration

New contract, no deployed consumer; lands in `LGE-V1.1-2026-08-27`. If V2 introduces physical unload or multi-package processes, a new ADR must supersede this one and redefine the latch.

## Verification

Fixtures `vpd/trusted`, `vpd/failed-check`, `failure/coreengine-load` (Process-scope load failure with `coreEngine` block), `failure/missing-snapshot` (scope discipline) and ErrorCode registry fixtures (`ids/registry`, `ids/duplicate`). Implementation must add repeated-load, conflict-latch and rollback tests per ADR-006's verification list.
