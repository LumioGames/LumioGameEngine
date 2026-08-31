# R-00316 ADR-051 Review-v1 Fix Evidence

## RED: regression tests before production edits

Command:

```text
python3 -m unittest tools.config_id_test -v
```

Result: exit `1`; 21 tests ran, with 13 failures and 1 error. The failures reproduced unlinked active/direct rows, unlinked AddRow/DeleteRow/RenameRow/RenameColumn history, unknown column-rename endpoint acceptance, inverted persistence declarations, the contradictory row `persisted` field, order-dependent tombstone replacement, rows-empty structural rejection, and incorrect duplicate-tombstone code; the error reproduced an uncaught `TypeError` for `persistence.stableFields = null`.

Production code and schema changes had not yet been made when this RED run was captured.

## GREEN and Fix Coverage

The validator now enforces bidirectional AddRow/assignment/active-row atoms,
operation-to-history links, duplicate logical-record rejection, alias-aware
column rename identity resolution, explicit stable/ephemeral persistence
declarations, complete consumed-ID checks for tombstone replacements, and
fail-closed collection handling. The registry schema permits `rows: []`, drops
the contradictory row `persisted` property, and the source validator now emits
`ConfigDuplicateStableId` for duplicate tombstone row identities while keeping
`ConfigTombstoneReuse` for duplicate/reused tombstone numbers.
The persistence schema is closed over the six ADR stable fields and the sole
`revisionOrdinal` ephemeral field, with `persistedOrdinals` constrained to an
empty array.

Command:

```text
python3 -m unittest tools.config_id_test tools.config_patch_test
```

Result: exit `0`; 54 tests ran, 54 passed.

The focused vectors include direct/preallocated rows, missing and duplicate
history links, both unknown column-rename endpoints, persistence inversion and
malformed declarations, row `persisted` rejection, later-entry tombstone
replacement reuse, a valid tombstone-only snapshot, malformed registry
collections, and duplicate tombstone identity versus numeric reuse codes.

## Verification Evidence

```text
python3 -m py_compile tools/lumio_contract.py tools/lumio_generate.py tools/config_id_test.py tools/config_patch_test.py
exit 0

python3 tools/lumio_contract.py validate
Validated 240 fixture(s), 0 failure(s).
JSON evidence `C:/Temp/R-00316-validate-final.json` SHA-256
`4827b22f65c3f275dc1eedb50ee4c82834478ac0ef52b3048cbfe3c53242f2dd`.

python3 tools/lumio_contract.py generate --out packages
generated 12 artifacts; stable outputHash: yes
compilerHash 127e7165564bb1f6b6384bdada551d49b2794c34276e490b70e30f4aeb27661d
inputHash 49fb34ea7430a192459b5b409dda457772fa1f2d71aa9410fccf40b832d4588b
Root ABI bundle digest 9347330956ca1a0d9e0175c070a82e7057f8765d93c2430dda74dde7f20211c5

External generated-output parity: 70 files, 0 SHA-256 mismatches.
`packages/index.json` SHA-256 `408ADDBD477CB8D68F868EFBB1E12E412CE9B793815C7F46A0C6106CFAB0C227`.
`packages/abi/root-abi-bundle.json` SHA-256 `9347330956CA1A0D9E0175C070A82E7057F8765D93C2430DDA74DDE7F20211C5`.
```

Clean materialization:

```text
path C:/Temp/lumio-r00316-clean-fix-final (core.symlinks=true; junction/symbolic links resolve into .spec)
node .spec/tools/spec-lint.mjs -> spec-lint: OK
python3 tools/lumio_contract.py validate -> Validated 240 fixture(s), 0 failure(s)
clean generator parity -> 70 files, 0 SHA-256 mismatches
python3 -m unittest tools.config_id_test tools.config_patch_test -> 54/54 passed
cargo check --manifest-path packages/rust/Cargo.toml --offline -> exit 0
cargo test --manifest-path packages/rust/Cargo.toml -p lumio-gen-contract-runtime --offline -> 3 passed, 0 failed
dotnet restore (six generated projects) -> 0 failures
dotnet build (six generated projects; netstandard2.1 and net8.0) -> 0 warnings, 0 errors
python3 tools/lumio_kat.py -> csharp/hashlib/rust 3/3; cross-language agreement
```

The ordinary target checkout's direct spec-lint remains limited by the
pre-existing Windows tracked-link materialization (`.claude/agents`,
`.claude/skills`, `.agents/skills`); no tracked symlink placeholder was edited.
`node --test .spec/tools/spec-lint.test.mjs` passes all 13 tests.

`git diff --check` exits 0. Parent-byte checks against accepted R-00315
integration `23b9e019e3254ad3830b86b3fc6a8fd7cb22560e` show unchanged blobs for
ADR-007, ADR-010, ADR-033, ADR-034, ADR-050, `schemas/config-source`,
`schemas/config-patch`, and `tools/config_patch_test.py`.

## Native Acceptance Mapping

1. **Namespace authorization:** unchanged ADR/ID registration remains the
   unique LumioConfig owner/range/serial-authority entry; semantic checks reject
   metadata drift and out-of-band IDs.
2. **Identity layers:** registry/schema/ADR now require stable sourceName and
   stableNumericId declarations, ephemeral revisionOrdinal, empty persisted
   ordinals, and alias-aware stable column ordinals; row `persisted` is rejected.
3. **Concurrency and compatibility:** every active row/AddRow/assignment is a
   one-to-one merge atom; rename/delete/column-rename records are cross-linked;
   replacements check the complete consumed set independent of list order.
4. **Stable failures:** registered negative fixtures cover all five frozen codes
   and the new duplicate-identity distinction; full validation passes 240/240.

## Scope and Gaps

Scoped changes are limited to ADR-051 documentation, its registry schema and
fixtures/index entries, focused ID tests, validator semantics, and generator
outputs regenerated by the required command. Accepted ADR-050 files and
tracked symlink placeholders remain untouched; no Workflow API or credentials
were accessed. ADR-051 remains Draft by design, and no production allocator,
editor, or runtime behavior was added.

## Commit

Branch `Go1c/rm-00009-r00316`, worktree
`C:/Users/g923/orca/workspaces/LumioGameEngineArchitecture/rm-00009-r00316`.
Fix commit: `512da155f6480a9989f6f01815bd1e0aa83f9b54` (parent
`f317b920bfa6401a82cce600d0874f871de1160b`); accepted integration parent
`23b9e019e3254ad3830b86b3fc6a8fd7cb22560e` and prerequisite
`398a2dd5c382260defc9cd6aa70a27e58aa741ba` remain in ancestry.
The commit contains 42 scoped paths; the parent-relative scope audit reports
zero unexpected paths and zero symlink/ADR-050 violations. Post-commit focused
tests (54/54), full validation (240/240), generator parity (70/70),
`git diff --check`, and accepted-ADR byte immutability all passed.
