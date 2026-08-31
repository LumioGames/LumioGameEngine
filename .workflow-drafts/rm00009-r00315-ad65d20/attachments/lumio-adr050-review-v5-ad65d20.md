# Independent Review: R-00315 / ADR-050 Review-v5

## Scope and isolation

- Reviewed committed range: d7a2cb6f871861a37f068487888e5dd14a8a20bc..ad65d20eb471f7d360fe9cb73638f0b0c21da474.
- Candidate commit: ad65d20eb471f7d360fe9cb73638f0b0c21da474; parent d7a2cb6f871861a37f068487888e5dd14a8a20bc.
- Fresh isolated review worktree: C:/Temp/rm00009-r00315-review-v5-ad65d20.
- Fresh symlink-enabled clone: C:/Temp/rm00009-r00315-v5-symlink2.
- Both snapshots were clean before and after verification. The original repository source was not edited, staged, committed, pushed, or sent to Workflow; no credentials were read or changed.
- Prior materials read in full:
  - C:/Users/g923/orca/reports/lumio-adr050-review-v4-d7a2cb6.md
  - C:/Temp/rm00009-r00315-review-v4-final-report.md
- Relevant ADR/spec material read: ADR-050, accepted ADR-007/010/033/034, testing/repository-architecture/workflow standards, and the complete changed diff.
- Range scope: 29 paths, 273 insertions and 61 deletions. The only non-generated paths are ADR-050, tools/lumio_contract.py, and tools/config_patch_test.py.

## Verdict

PASS

No P0, P1, or P2 findings were found in the reviewed range. Every requested ADR-050 replay passed, including argument-order equality for mixed unregistered successors, monotonic hash-disagreement metadata, required-column removal rejection, the optional missing-cell matrix, intrinsic-error precedence, exact old preconditions, deterministic row/field merges, malformed input rejection, accepted-ADR immutability, generated-artifact parity, Rust/C#/KAT checks, and core.symlinks=true spec-lint.

## Acceptance matrix

| Criterion | Result | Exact implementation/evidence |
|---|---|---|
| Sole source and machine/human gate boundary | PASS | ADR-050 lines 16-57; merge reference path tools/lumio_contract.py lines 1536-1703. No editor, overwrite, signing, or activation path was added. |
| Mixed unregistered successor/base claims reject identically in both orders | PASS | tools/lumio_contract.py lines 1572-1591 validate the trusted source independently, then both claims. The assertion replay produced identical Rejected/InvalidArgument output in forward and reverse order. |
| Newer observed successor plus base-hash disagreement carries observed/retry metadata | PASS | tools/lumio_contract.py lines 1631-1657 and 1498-1533. For observed revision 99 and expected 12, both orders returned observedRevision 99 and retryRevision 100. |
| Required published column cannot be removed | PASS | tools/lumio_contract.py lines 1439-1440, included by intrinsic validation at lines 1464-1495. Both orders returned Rejected/InvalidArgument with the required-column diagnostic. |
| Optional missing-cell add/replace/remove matrix | PASS | tools/lumio_contract.py lines 1380-1389. An in-memory registered source with missing optional cooldown accepted add/value and add/default, and rejected replace/remove as required. |
| Exact old preconditions | PASS | tools/lumio_contract.py lines 1108-1119 and 1441-1460. Scalar and state-descriptor matches were accepted without coercion; mismatches were rejected on a current base. |
| Intrinsic operation/duplicate/target/type errors before stale arbitration | PASS | tools/lumio_contract.py lines 1601-1689. Duplicate, malformed remove shape, unknown table/row/column, row-name, wrong type, range, null, and add-with-old vectors all returned deterministic InvalidArgument with observed revision 99. |
| Documented stale precedence for state/old checks | PASS | ADR-050 lines 42-55; merge comments/branches at tools/lumio_contract.py lines 1601-1606 and 1658-1681. Current-base state/old checks reject; newer-source state/old vectors return retryable RevisionConflict before those source-state checks. |
| Valid deterministic row/field merges and same-unit race | PASS | tools/lumio_contract.py lines 1691-1703; focused tests passed 32/32 and reverse row merge equality held. |
| Malformed input handling | PASS | Structural rejection path tools/lumio_contract.py lines 1559-1570; malformed non-object patch probes returned Rejected/InvalidArgument without exceptions. |
| Accepted ADR-007/010/033/034 immutability | PASS | Candidate blob IDs equal both parent and baseline for all four accepted ADRs; see evidence below. |
| Generated artifacts and scope | PASS | Generator output and all 70 tracked package files matched byte-for-byte; changed JSON fields are compilerHash/root ABI digest updates produced by the generator. |

## Findings

None. There is no file:line defect to report at P0, P1, or P2 severity in this range.

## Focused and adversarial replay

Command:

    python -m unittest tools.config_patch_test -v

Output:

    Ran 32 tests in 7.950s
    OK
    EXIT=0

Raw output: C:/Temp/rm00009-r00315-v5-focused.txt

The consolidated independent assertion replay (raw output C:/Temp/rm00009-r00315-v5-assertions.txt) ended with ASSERTIONS_OK and EXIT=0. It covered:

    PASS mixed successor symmetry
    PASS mixed rejected
    PASS hash mismatch symmetry
    PASS hash mismatch metadata
    PASS required remove symmetry
    PASS required remove rejected
    PASS optional add-value
    PASS optional add-default
    PASS optional replace-missing
    PASS optional remove-missing
    PASS stale intrinsic duplicate
    PASS stale intrinsic shape
    PASS stale intrinsic tableId
    PASS stale intrinsic rowId
    PASS stale intrinsic column
    PASS stale intrinsic rowName
    PASS stale intrinsic value
    PASS stale intrinsic range
    PASS stale intrinsic null
    PASS stale intrinsic add-old
    PASS stale precondition add-existing
    PASS stale precondition wrong-old
    PASS row merge deterministic
    PASS field merge accepted
    PASS same-unit conflict
    PASS malformed left None
    PASS malformed left []
    PASS malformed left {}
    PASS malformed left 'x'
    PASS malformed left 1
    ASSERTIONS_OK
    EXIT=0

Representative exact replay results:

    mixed.forward={"errorCode":"InvalidArgument","errors":["published source context successor cannot be used as an unregistered merge base"],"retryable":false,"status":"Rejected"}
    mixed.reverse={"errorCode":"InvalidArgument","errors":["published source context successor cannot be used as an unregistered merge base"],"retryable":false,"status":"Rejected"}

    hash.forward={"conflictReason":"base source hash changed","conflictingUnits":[],"errorCode":"RevisionConflict","expectedRevision":12,"observedRevision":99,"retryHint":"refresh base revision and resubmit","retryRevision":100,"retryable":true,"status":"Conflict"}
    hash.reverse={"conflictReason":"base source hash changed","conflictingUnits":[],"errorCode":"RevisionConflict","expectedRevision":12,"observedRevision":99,"retryHint":"refresh base revision and resubmit","retryRevision":100,"retryable":true,"status":"Conflict"}

    required.forward={"errorCode":"InvalidArgument","errors":["patch operation combat-skills/fireball/damage remove cannot target required published column"],"retryable":false,"status":"Rejected"}
    required.reverse={"errorCode":"InvalidArgument","errors":["patch operation combat-skills/fireball/damage remove cannot target required published column"],"retryable":false,"status":"Rejected"}

    stale-pre.wrong-old={"conflictReason":"base source snapshot is stale","conflictingUnits":[],"errorCode":"RevisionConflict","expectedRevision":12,"observedRevision":99,"retryHint":"refresh base revision and resubmit","retryRevision":100,"retryable":true,"status":"Conflict"}

## Contract/spec gates

Command:

    python -m py_compile tools/lumio_contract.py tools/lumio_generate.py tools/config_patch_test.py

Output:

    EXIT=0

Command:

    python tools/lumio_contract.py validate

Output (full 221-fixture output is in C:/Temp/rm00009-r00315-v5-validate.txt):

    PASS world/authority-ready (valid)
    PASS world/bad-role (invalid)
    Validated 221 fixture(s), 0 failure(s).
    EXIT=0

Command:

    node .spec/tools/spec-lint.mjs

Output:

    spec-lint: OK
    EXIT=0

Command:

    node --test .spec/tools/spec-lint.test.mjs

Output:

    tests 13
    pass 13
    fail 0
    EXIT=0

The test command intentionally prints the 11 negative fixture diagnostics before the 13 passing test records; all 13 tests passed. Raw outputs: C:/Temp/rm00009-r00315-v5-speclint.txt and C:/Temp/rm00009-r00315-v5-speclint-test.txt.

## Generator and generated-file comparison

Command:

    python tools/lumio_contract.py generate --out C:/Temp/rm00009-r00315-v5-generated

Output:

    generated 12 artifacts under C:\Temp\rm00009-r00315-v5-generated
    compilerHash 449c9985eb2247a609a4ac040a9e7936e51122390a8f28b4d0692377ae88bfe5
    inputHash 668f4e8bcb1ee0bd043460956278fc05ceca0f007b39ec3abac6534b173f58bd
    rootAbi bundle abi/root-abi-bundle.json digest ea809368ec8b120142b23bb81f87ce42872fe474ee8656ef09ba0098613b7485
    stable outputHash: yes
    EXIT=0

Independent byte comparison against the candidate packages tree:

    generated_count=70
    committed_count=70
    path_sets_equal=True
    compared=70
    mismatches=0

All changed package JSON documents were recursively compared against the parent. Each descriptor changed only compilerHash; packages/index.json changed only the twelve artifact compilerHash values, index compilerHash, root ABI bundle digest, and root ABI compiler digest. The root bundle compiler digest, index compilerHash, and every artifact compilerHash are the same 449c9985 value; the index rootAbi bundleDigest equals the independently computed bundle SHA-256.

Raw generator output: C:/Temp/rm00009-r00315-v5-generate.txt.

## Rust, C#, KAT, and purity checks

Environment:

    Python 3.12.10
    Node v24.18.0
    cargo 1.98.0
    dotnet 10.0.111
    jsonschema 4.25.1

Commands and outputs:

    cargo check --manifest-path packages/rust/Cargo.toml --offline
    Finished dev profile; EXIT=0

    cargo tree --manifest-path packages/rust/Cargo.toml -p lumio-gen-contract-runtime --offline
    lumio-gen-contract-runtime v0.0.0 (...)
    EXIT=0

    cargo test --manifest-path packages/rust/Cargo.toml -p lumio-gen-contract-runtime --offline
    running 3 tests
    test truncated_buffer ... ok
    test chain_round_trip ... ok
    test sha256_known_answer_vectors ... ok
    test result: ok. 3 passed; 0 failed
    EXIT=0

    cargo clippy --manifest-path packages/rust/Cargo.toml --all-targets --offline -- -D warnings
    Finished dev profile; EXIT=0

    python tools/lumio_kat.py
    csharp   OK (3 vectors)
    hashlib  OK (3 vectors)
    rust     OK (3 vectors)
    csharp + hashlib + rust agree on 3 FIPS 180-4 vectors
    EXIT=0

The CI purity grep was run before any C# build output existed in the fresh symlink clone:

    purity=OK

All six C# projects were then built sequentially in the isolated review worktree. Each reported 0 warnings and 0 errors, with PROJECTS=6 FAILED=0. The six project names were Lumio.Gen.CanonicalSerializer, Lumio.Gen.ContractRuntime, Lumio.Gen.ContractTypes, Lumio.Gen.LanguageBinding, Lumio.Gen.MappingTable, and Lumio.Gen.ProtocolPermissionValidator.

Raw Rust/KAT outputs: C:/Temp/rm00009-r00315-v5-cargo-check.txt, C:/Temp/rm00009-r00315-v5-cargo-tree.txt, C:/Temp/rm00009-r00315-v5-cargo-test.txt, C:/Temp/rm00009-r00315-v5-cargo-clippy.txt, and C:/Temp/rm00009-r00315-v5-kat.txt.

## Fresh core.symlinks=true clone

Clone verification:

    git rev-parse HEAD
    ad65d20eb471f7d360fe9cb73638f0b0c21da474
    git config --get core.symlinks
    true
    git status --porcelain=v1
    (empty)
    .claude/agents -> ..\.spec\agents (SymbolicLink)

Commands:

    node .spec/tools/spec-lint.mjs
    spec-lint: OK
    EXIT=0

    node --test .spec/tools/spec-lint.test.mjs
    tests 13; pass 13; fail 0; EXIT=0

    python -m unittest tools.config_patch_test -v
    Ran 32 tests ... OK
    EXIT=0

    python tools/lumio_contract.py validate
    Validated 221 fixture(s), 0 failure(s).
    EXIT=0

The clone remained clean after these checks; clone generation also reproduced compilerHash 449c9985 and root ABI digest ea809368.

## Immutability, baseline, and hygiene

Accepted ADR blob comparison (candidate equals both parent and baseline a7c1221):

    .spec/decisions/ADR-007-contract-toolchain.md
      parent=1455ed3331f5929a589f2ab486222e8dbc1922a0
      candidate=1455ed3331f5929a589f2ab486222e8dbc1922a0
      baseline=1455ed3331f5929a589f2ab486222e8dbc1922a0
    .spec/decisions/ADR-010-persistence-config.md
      parent=67e7678c72adf161ffe778232a248a02a73cbfd4
      candidate=67e7678c72adf161ffe778232a248a02a73cbfd4
      baseline=67e7678c72adf161ffe778232a248a02a73cbfd4
    .spec/decisions/ADR-033-config-typed-columns.md
      parent=9b6342c14bbeafc633ff13d0eb5d2812f246631b
      candidate=9b6342c14bbeafc633ff13d0eb5d2812f246631b
      baseline=9b6342c14bbeafc633ff13d0eb5d2812f246631b
    .spec/decisions/ADR-034-hot-reload-dual-scope.md
      parent=e67307440695b54f116aee3ccc51cb332ad0077a
      candidate=e67307440695b54f116aee3ccc51cb332ad0077a
      baseline=e67307440695b54f116aee3ccc51cb332ad0077a

Baseline:

    expected=f1d36acf33a1f5e8326a9e58d609fcf7d9fa85177f9b5b60bb3f4742c1afebd0
    actual=f1d36acf33a1f5e8326a9e58d609fcf7d9fa85177f9b5b60bb3f4742c1afebd0
    baseline=OK

Range hygiene:

    git diff --check d7a2cb6..ad65d20
    diff-check: OK
    changed_paths=29
    unexpected=[]
    all_expected=True

Post-review status:

    primary worktree: clean
    fresh symlink clone: clean

## Residual notes

- The optional missing-cell matrix was replayed with an in-memory source substituted as the registered trust root because the committed fixture set has one published source whose optional cooldown cells are present. This is a verification technique only; no fixture or repository file was changed.
- The independent clone is the authoritative clean-clone spec-lint reproduction.
- The generated command and all build/test commands wrote only temporary or ignored outputs outside the committed range.

Report path: C:/Users/g923/orca/reports/lumio-adr050-review-v5-ad65d20.md
