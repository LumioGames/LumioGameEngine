# ADR-015: P2 Mod SDK Extension Boundary

- **Status**: Reserved (P2; not loaded by V1)
- **Owner**: `LumioGame` (content policy), Runtime/Host (sandbox contract)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

The product will eventually need controlled Mod support, but loading arbitrary third-party code before the core architecture is proven would expand the trust, determinism and persistence surface.

## Decision

V1 only reserves `ModManifest`, capability and permission declarations, memory/CPU/entity quotas, lifecycle hooks, typed Schema and save hooks. Future P2 delivery may support signed Data or Managed Mods through the Runtime Contract. Native DLLs, raw pointers, sockets, reflection escapes and unbounded threads are forbidden by the boundary.

## Contract

`mod-manifest.schema.json` is a reservation schema. A manifest is not an authorization grant: signature verification, review, capability allow-listing and per-Release policy remain mandatory. V1 lifecycle is `Reserved`; no third-party Mod is loaded in production.

## Failure semantics

Unsigned, revoked, over-quota, incompatible or capability-unauthorized Mods are rejected before load. A runtime fault disables the Mod scope and preserves the host World/Save; it cannot roll back authoritative state outside its declared transaction.

## Alternatives

Loading arbitrary Native plugins was rejected for process safety and ABI stability. Implicit script discovery was rejected for supply-chain and reproducibility reasons. Shipping a permissive Mod API in V1 was rejected because it would freeze unvalidated semantics.

## Compatibility and migration

Mod schema/capability changes require a new Mod API epoch and explicit Save migrator. A Release can disable Mods without changing core Gameplay; saves containing Mod data must retain a quarantine/migration path.

## Verification

Use the reserved valid fixture and native-library failure fixture, then add signature/revocation, quota, sandbox, save-hook and deterministic replay tests in the P2 ADR before implementation.
