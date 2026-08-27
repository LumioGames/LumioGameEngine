# Contract Tooling

`lumio_contract.py` is the bootstrap contract gate for the architecture source repository.

```text
python3 tools/lumio_contract.py validate
python3 tools/lumio_contract.py validate --fixture txn/committed
python3 tools/lumio_contract.py validate --json > contract-result.json
python3 tools/lumio_contract.py canonical fixtures/valid/cross-world-txn-committed.json
python3 tools/lumio_contract.py hash schemas/cross-world-txn.schema.json
```

The validator first attempts the pinned upstream `jsonschema` Draft 2020-12 implementation from `requirements-dev.txt`. In a documentation-only checkout where that package is not installed, it uses the deterministic subset implemented in the same file; CI should install the pinned dependency for full standards coverage. The semantic checks are intentionally narrow: they enforce architecture decisions such as commit ordering, revision monotonicity, exact release matching, maintenance action pairing and V1 Mod restrictions, while leaving domain policy to the owning repository.

Generated serializers, ABI headers, bindings and language-specific validators are future outputs of this registry. They must record the compiler version, input hash and output hash and must never silently edit a checked-in fixture.
