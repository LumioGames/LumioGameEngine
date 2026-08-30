# Contract Fixtures

Fixtures are small, reviewable protocol records used by the architecture gate. `index.json` maps each fixture to a registered schema and states whether the record is expected to be accepted or rejected.

Run:

```text
python3 tools/lumio_contract.py validate
```

An `invalid` fixture is successful only when structural or semantic validation rejects it. Registered GAS invalid fixtures additionally declare an `expectedError` rule key in `index.json`, and the exact key must be emitted by the gate; free-form or broad substrings and unrelated errors cannot satisfy the fixture. This keeps failure behavior testable without pretending that a failure fixture is a production payload. Every P0 schema has both an accepted record and a rejected record; the Mod record demonstrates the reserved P2 boundary.

The `gas/*` fixtures cover the closed Ability/Effect state machines, five-step admission, two-check Commit, deterministic Decimal34 evaluation (including exact decimal lexemes, exponent bounds and array permutations), same-Tick Effect ordering, suppression and Tick-only timing. Invalid GAS fixtures must report their registered `expectedError` rule key; silently accepting an expected-invalid record or reporting an unrelated error fails the validation command.

The A2 `gas/*` fixtures additionally cover the four ECS containers and Handle
resolution, the permanent Tag table and hierarchy handshake, field visibility
and dual hash projections, the derived-only Modifier ledger boundary, and
frame-keyed prediction rollback/replay. The replication fixture declares all
seven component-field pairs; hash-domain exclusions describe projection
domains and do not excuse an absent pair. Each new P0 schema has positive and
negative records in this index.

Fixtures intentionally use placeholder hashes and signatures. They exercise shape, correlation, ordering and compatibility rules; release signing and cryptographic verification belong to the Release Toolchain implementation.
