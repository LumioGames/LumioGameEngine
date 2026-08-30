# Contract Tooling

`lumio_contract.py` is the bootstrap contract gate for the architecture source repository.

```text
python3 tools/lumio_contract.py validate
python3 tools/lumio_contract.py validate --fixture txn/committed
python3 tools/lumio_contract.py validate --json > contract-result.json
python3 tools/lumio_contract.py canonical fixtures/valid/cross-world-txn-committed.json
python3 tools/lumio_contract.py hash schemas/cross-world-txn.schema.json
python3 tools/lumio_contract.py generate --out packages
```

The validator first attempts the pinned upstream `jsonschema` Draft 2020-12 implementation from `requirements-dev.txt`. In a documentation-only checkout where that package is not installed, it uses the deterministic subset implemented in the same file; CI should install the pinned dependency for full standards coverage. The semantic checks are intentionally narrow: they enforce architecture decisions such as commit ordering, revision monotonicity, exact release matching, maintenance action pairing and V1 Mod restrictions, while leaving domain policy to the owning repository.

`generate` publishes the ADR-023 six kinds × Rust/C# under `packages/` (plus schema-valid descriptors and `packages/index.json`). Each artifact records `baselineId`, `schemaEpoch`, `compilerHash`, `inputHash` and `outputHash`. Two consecutive runs must produce equal `outputHash` values. Do not hand-edit package sources; regenerate from `schemas/`, `ids/index.json` and `fixtures/valid`. D-009 (protocol-dispatch) and D-011 (Auth wire) stay blocked and do not receive Artifact names. The generator must never silently edit a checked-in fixture.

For GAS A1, the semantic gate checks the frozen lifecycle event table, Admission/Commit order and outcomes, single charging, exact-lexeme Decimal34 arithmetic and bounds, deterministic evaluation/override selection, and same-Tick Effect event order. GAS invalid fixture metadata is matched by registered rule key, never by a substring. The schemas remain the public data contract; this tool is the single architecture validator for cross-field invariants and is not a Runtime implementation.

For GAS A2, the same gate checks the four ECS containers and world-bound
Handle probes, the complete permanent Tag table and hierarchy/hash handshake,
field visibility and exact seven-pair matrix completeness, complementary dual
hash domains, the derived-only Modifier ledger boundary, and frame-keyed
prediction rollback/replay. It does not implement ECS, replication or Runtime.

Compiler, input, Root ABI, and package output digests cover raw checked-in text bytes. The repository pins the participating source/input/output paths to LF in `.gitattributes`; before publishing from any platform, check `git ls-files --eol` and do not publish identities generated from a CRLF materialization.
