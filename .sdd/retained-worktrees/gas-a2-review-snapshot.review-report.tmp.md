# GAS-A2 / R-00301 Independent Review

## Scope

- Reviewed commit `a4c1c57f0c273346c45ea9a3b6c39c7a223ded16` against recorded base `88a3f14379e9f09982acd31c30d5ca8d5a53109` in detached worktree `gas-a2-review-snapshot`.
- Full diff: 58 files, 1882 additions, 28 deletions; `git diff --check` exit 0.
- No tracked files were edited by this review.

## Spec Verdict: RETURN / P1

The schemas and fixtures express the requested four-container, FxComponent prohibition, Handle invalidation, permanent Tag table/hash handshake, field visibility/hash domains, and frame-keyed rollback boundaries. However, two acceptance-critical semantic holes allow invalid contracts to validate as `valid`:

1. **P1: hierarchy coverage can be omitted.** [`tools/lumio_contract.py`](tools/lumio_contract.py:2093) only rejects incomplete query-mode coverage when `query_modes` is non-empty. A valid `gas-tag` record with `queries=[]` passes both structural and semantic validation, so the counted hierarchical Tag contract is not mandatory.
2. **P1: replay frames are not required to be later than the rejected input.** [`tools/lumio_contract.py`](tools/lumio_contract.py:2261) checks replay frames only against `confirmedFrame` and excludes equality with `inputFrame` at line 2270; it never requires `frame > inputFrame`. A record with `confirmedFrame=98`, rejected `inputFrame=100`, and `replayInputFrames=[99]` passes as valid, despite replaying an earlier frame rather than later unconfirmed inputs. [`schemas/gas-prediction.schema.json`](schemas/gas-prediction.schema.json:56) likewise only requires a non-empty unique integer array.

No P0 issue was found. The component validator does enforce exactly four unique container names and terminal Handle invalidation; Tag table and schema hashes are recomputed; replication inclusion/exclusion complements and hash preimages are checked; and prediction rejects Effect removal/period/out-of-simulation prediction plus server rollback.

## Quality Verdict: CONDITIONAL RETURN

- `python3 tools/lumio_contract.py validate --json`: exit 0, `validated=236`, `failures=0`.
- Targeted A2 fixtures (12 registered fixtures): all exit 0, expected valid/invalid outcomes honored.
- `python3 -m py_compile tools/lumio_contract.py`: exit 0.
- Draft 2020-12 schema check: exit 0, `schemas 54 draft2020-check=0`.
- Official generation to `.review-generated`: exit 0; 12 artifacts, stable output hash; tracked `packages/` vs generated tree diff exit 0.
- `node --test .spec/tools/spec-lint.test.mjs`: exit 0, 15/15 tests pass.
- `node .spec/tools/spec-lint.mjs`: exit 1 only for three Windows symlink materialization checks (`.claude/agents`, `.claude/skills`, `.agents/skills`), an environment gap also present in the producer worktree.

## Additional Notes

- Unknown `typeId` strings are not cross-checked against an ID namespace (`_gas_components_errors` at line 1891); this is a possible P2 contract-completeness gap if TypeId is intended to be registry-bound, but no dedicated TypeId registry is frozen in R-00301.
- Generated package changes are consistent with the official generator; no implementation repository paths, RPC/Task/PredictionWindow fields, wall-clock fields, FxComponent declarations, or standalone Modifier ledger fields were introduced.

## Next Action

Require the producer to make query-mode coverage unconditional and require each replay frame to be strictly greater than `inputFrame` (with negative fixtures for both), regenerate official packages, and rerun the same gates.
