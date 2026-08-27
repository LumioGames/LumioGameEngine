# Contract Fixtures

Fixtures are small, reviewable protocol records used by the architecture gate. `index.json` maps each fixture to a registered schema and states whether the record is expected to be accepted or rejected.

Run:

```text
python3 tools/lumio_contract.py validate
```

An `invalid` fixture is successful only when structural or semantic validation rejects it. This keeps failure behavior testable without pretending that a failure fixture is a production payload. Every P0 schema has both an accepted record and a rejected record; the Mod record demonstrates the reserved P2 boundary.

Fixtures intentionally use placeholder hashes and signatures. They exercise shape, correlation, ordering and compatibility rules; release signing and cryptographic verification belong to the Release Toolchain implementation.
