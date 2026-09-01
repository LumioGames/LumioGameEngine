# [Account] Account Server Login-or-Register and AccountEntity

## Metadata

- Blueprint: `ecs-entity-chat-20260901/r1/R-ACCOUNT`
- Target repository: Account Server integration owner (central account service)
- Category: server / account
- Module: Account Server
- Wave: 1
- Priority: P0
- Risk: high
- Readiness: conditional until the Account Auth/Profile Port is frozen

## Background

Every Bot and Browser must authenticate through one real Account Server before entering a Room. The test harness must not bypass the account system or use a pre-provisioned in-memory map.

## Goal

Provide username/password login-or-register, stable AccountId allocation, long-lived AccountEntity load/create, profile response and an opaque Game Server admission credential.

## Preconditions

- Existing auth behavior and verifier Port: `R-00218`.
- Source and decisions in `docs/specs/2026-09-01-ecs-formal-entity-chat-requirements.md`.

## Requirements

- Accept username and password; use default test password `123456` only in the controlled Hello World profile.
- If username is absent, create the account and AccountEntity during the request; if present, validate without overwriting.
- Permit `Bot` plus decimal digits usernames only from the authenticated Bot-tool registration context; reject ordinary client attempts to create or claim that namespace.
- Return the same stable AccountId on repeated successful logins.
- Never return password material to clients or Game Server.
- Support Bot launcher names `Bot01` through `Bot100`; the caller generates names in a loop rather than relying on pre-provisioning.
- Return an opaque admission credential for Game Server Room admission.

## Acceptance

- First `Bot01` request creates one AccountEntity and AccountId; repeat returns the same AccountId.
- Wrong password for an existing username is rejected and leaves the account unchanged.
- Ordinary Browser/client registration of `Bot01`-style names is rejected; the Bot tool can register its generated names through the approved context.
- Concurrent first requests for one username converge on one AccountEntity and AccountId.
- 100 generated Bot names can authenticate through the same endpoint.

## Boundary

Do not implement production identity providers, password rotation policy or Game Server account bypass in this card. The default password is test-profile configuration, not a production security contract.
\n