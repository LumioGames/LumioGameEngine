# W0 Byte-Authority Review

Date: 2026-08-30  
Base: `b7db298658967f63cba37f5de7f4478b5851c18f`  
Candidate: uncommitted patch from `review-w0-byte-authority-uncommitted.diff`

## Verdicts

- **Spec verdict: PASS.** The exact candidate patch establishes LF checkout materialization for every raw byte input and generated text output whose digest is checked. Fresh candidate checkouts with both requested Git modes produce the same identities, validate all fixtures, and preserve the base contract surface.
- **Quality verdict: CHANGES REQUIRED.** The implementation is technically sound, but the fix report still needs one documentation correction for the earlier P1: it must explicitly distinguish generator-run worktree bytes, Git blob bytes, and clean-checkout bytes rather than referring to "tracked packages" without naming the byte source.

## Scope And Patch

The materialized candidate changes only `.gitattributes` (10 added lines). No generator, contract, schema, ID registry, fixture, baseline, ADR, public semantic, or generated package file is changed. The patch is valid Git attributes syntax and is limited to compiler sources, hashed inputs, and generated package text.

## Findings

### P0

None.

### P1

1. **Fix report does not explicitly label all three byte domains.** In `w0-line-ending-fix-report.md:41-43`, "Two independent scratch generations" and "tracked `packages/**`" are reported without stating that the first is generator-run working-tree output and the second is comparison against committed Git blob bytes. Lines 45-52 separately describe clean-clone bytes, but the required three-way distinction remains implicit. Amend the report with an explicit table naming: (a) generator output working-tree bytes, (b) `git show <commit>:<path>` blob bytes, and (c) fresh-checkout bytes for each `core.autocrlf` mode. The report correctly states at lines 71-72 that actual GitHub Ubuntu CI was not run; that statement should remain.

### P2

None.

## Byte-Authority Evidence

- Generator raw-byte paths inspected in `tools/lumio_generate.py`: `compiler_hash` hashes `tools/lumio_contract.py` and `tools/lumio_generate.py`; `input_hash` hashes all JSON under `schemas/`, `ids/index.json`, and `fixtures/valid/`; `abi_input_hash` hashes `schemas/native-managed-abi.schema.json` and `fixtures/valid/native-managed-abi.json`.
- Generated digest paths inspected: `dir_output_hash` hashes every file under each generated Rust/C# package except descriptor JSON; Root ABI records digests for `abi/lumio_core.h`, `root_abi.rs`, and `RootAbi.cs`; package/index/profile digests are generated as text. The candidate `/packages/** text eol=lf` covers all of them.
- `git check-attr eol` over 217 hashed/input/output paths returned zero non-LF results. `git ls-files --eol` representatives show `i/lf w/lf` and `text: set, eol: lf` in fresh clones. Package extensions are only `.json`, `.h`, `.rs`, `.cs`, `.csproj`, `.lock`, `.toml`, `.md`, and `.gitignore`; no binary artifact is unintentionally classified.
- Attribute syntax was accepted by Git and produced the expected checkout materialization in fresh clones with `core.autocrlf=true` and `core.autocrlf=false`.

## Identity And Generation Evidence

- Two fresh generations (one per checkout mode) each emitted 70 files; the trees were byte-for-byte equal. Each generated tree matched all 70 tracked package Git blobs (`git show HEAD:packages/<path>`), with zero mismatches.
- Both modes published identical identities: `compilerHash=0aaf61d65153aadc4ddda1b36fa1b7bfb38373d52e8ba3299457cefe16864bff`, `inputHash=bb95d87078c83b40e5148f58d68aa7a1df7cded94d28657a0f11e4f1231c2ff`, Root ABI `inputHash=696a58d0525b897b549dd1e432166ae1020835902a5984221a8e60d5d8285bb3`, and bundle digest `02dce705a9a6fe7a437ed2e4137b03de7341ed614f30f10b614659c5226184a7`.
- `python tools/lumio_contract.py validate` passed in both fresh modes: `Validated 201 fixture(s), 0 failure(s).`
- Existing identities are correct: base package Git blobs already contain the same LF-derived hashes, and official generation produced no package diff. No BaselineId or schema/ID/fixture/package semantic refresh is required.

## Required Checks

- `node .spec/tools/spec-lint.mjs`: direct run on this Windows host reports three pre-existing broken symlink materializations (`.claude/agents`, `.claude/skills`, `.agents/skills`). With junctions in the isolated candidate clone, the unmodified command returns `spec-lint: OK`.
- Unmodified `node --test .spec/tools/spec-lint.test.mjs`: 13/13 pass, 0 fail. The test intentionally exercises malformed temporary repositories; its diagnostic output is expected and all assertions pass.
- Python compilation, `python tools/lumio_kat.py`, baseline SHA-256, and `git diff --check` all pass. KAT independently reports C#, hashlib, and Rust agreement on 3 FIPS vectors.
- Rust workspace `cargo test` passes (3 contract-runtime tests, including SHA-256 vectors) and `cargo clippy --workspace --all-targets -- -D warnings` passes.
- All six generated C# projects build successfully for their published `netstandard2.1;net8.0` targets.
- R-00293 negatives pass with the expected rejections: `playerCommand` additional property, `mvpVoxelPayload` additional property, and non-string `mappingSetHash`.

## Cannot Verify

- Actual GitHub Actions Ubuntu execution was not run, because this review forbids push/PR/CI writes. The two fresh checkout modes provide local byte and validator evidence, but do not replace hosted CI.
- The direct Windows `spec-lint` failure is an environment symlink privilege/materialization issue; junction-assisted lint and all 13 unmodified tests pass.

## Next Action

Amend `w0-line-ending-fix-report.md` with the explicit worktree/Git-blob/clean-checkout evidence table, then rerun the repository-policy workflow on Ubuntu. Do not commit or push the current candidate, and keep W1 consumers blocked until that CI job is green.
