---
name: 2026-09-01-owner-wire-landing
description: Owner 对 RM-00011 C-1 wire 落地的书面确认件;核 ADR-049 合入授权时查
metadata:
  type: doc
  status: 已交付
---

# Owner confirmation: RM-00011 C-1 wire landing (2026-09-01)

This is the written Owner artifact required before merging ADR-049 / `lumio.gameplay-envelope.v1`.

Source: Owner dispatch of RM-00011 (contracts-first vertical slice) plus continuation of the interrupted Claude session that already executed this landing. The Owner instructed the dispatcher to continue that work rather than restart or self-select a different public-semantic path.

## Confirmed landing

1. **Path.** `engine/wire/<name>-v1.json` — one JSON contract file per Wave 0 C card. C-1 is `engine/wire/gameplay-command-envelope-v1.json`.
2. **Validator.** New unified entry `eng/verify-wire.mjs`. It auto-discovers `engine/wire/*.json`. `hello-wire-v1.json` is included in the same entry and must pass unchanged. `eng/verify-hello-wire.mjs` is not modified and must keep passing. C-1 must not extend `hello-wire-v1`.
3. **ABI.** `engine/abi/native-abi.json` is unchanged by C-1. `node eng/generate-abi.mjs` remains zero-diff.
4. **Downstream consume.** Implementation repos pull the frozen JSON from architecture `origin/main` after the matching C-card merge. Production code that parses the new envelope / binding-query / account-port / timer must not appear on a repo `main` dated before that merge SHA. Non-semantic scaffolding and CI may start immediately.
5. **Forbidden.** Restoring `schemas/`, `ids/`, `fixtures/`, `packages/`, `tools/lumio_contract.py`, Baseline, or seven-repo mirrors as a C-card byproduct. That restoration would be a separate architecture decision and would require Owner re-boundary with LumioConfig.

## Card-literal deviation (recorded, not reconstructed)

Wave 0 card Core Prompts still recite the deleted delivery chain “ADR → Schema/ID → Fixture → Baseline → seven-repo mirrors” (Room Review 2026-09-01 was written against that chain). The live architecture surface after `59866ec` is ABI + `engine/wire`. Delivery of C-1…C-4 uses ADR + `engine/wire` JSON + shipped validator and embedded positive/negative cases as the development-state equivalent. Workflow closeout comments must cite this file so cards are not failed against an unsatisfiable literal.
