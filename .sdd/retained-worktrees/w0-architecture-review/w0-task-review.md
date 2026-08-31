# W0 Task Review

Date: 2026-08-30  
Base: `b7db298658967f63cba37f5de7f4478b5851c18f`  
Candidate: `753920e35b5c6cd590063a9febe8e9254a3ae6e6`

## Verdicts

- **Spec verdict: FAIL.** The candidate does not satisfy the required cross-environment deterministic identity and `python tools/lumio_contract.py validate` gate. Its new compiler and input identities are derived from a Windows CRLF checkout, while the unchanged repository blobs and the Ubuntu policy checkout retain the prior LF identities.
- **Quality verdict: CHANGES REQUIRED.** The 26 committed files are internally consistent generator output for the implementer's transient mixed-EOL working tree, but they are not a valid repository/CI identity refresh and cannot be released.

## Strengths

- The diff is tightly scoped: exactly 26 generator-owned JSON files, 26 insertions and 26 deletions; no BaselineId, schema, ID, fixture, generated code, public semantic, downstream mirror, or architecture prose changed.
- A structured JSON comparison found only identity fields: two fields in the Root ABI bundle, `compilerHash`/`inputHash` in each descriptor, and the corresponding compiler/input/bundle fields in `packages/index.json`.
- All 12 artifact `outputHash` values are unchanged. An independent Git-blob check found zero descriptor/index/package-descriptor/outputHash/Root-ABI agreement errors, and the candidate bundle bytes hash to the indexed `708ccb7e1bd25cb3c66caa3a13bdadfa5446ff4403a0d043333f59e737eae583`.
- All 70 candidate `packages/**` Git blobs equal a fresh generation byte-for-byte when the generator is run against this Windows CRLF checkout. This is strong evidence the 26 files were generator-produced rather than hand edited, although the input identity is the wrong platform-specific identity.
- `docs/architecture/.baseline.sha256` is unchanged and still verifies as `f1d36acf33a1f5e8326a9e58d609fcf7d9fa85177f9b5b60bb3f4742c1afebd0`. This is correct because neither the architecture document nor public semantics changed.

## Findings

### P0 (Must Fix)

1. **The refreshed identities are CRLF working-tree identities and fail both a clean Windows checkout and the official LF/Ubuntu gate.**

   Evidence:

   - [`tools/lumio_generate.py:68`](../tools/lumio_generate.py#L68) and [`tools/lumio_generate.py:87`](../tools/lumio_generate.py#L87) hash raw `read_bytes()` from schemas, valid fixtures, and the two compiler sources. [`tools/lumio_generate.py:92`](../tools/lumio_generate.py#L92) therefore makes line endings part of compiler identity.
   - [`.gitattributes:1`](../.gitattributes#L1) declares only `* text=auto`; it does not freeze LF for `.py`, `.json`, generated `.h`, `.rs`, `.cs`, `.toml`, or project files. With the installed Git configuration `core.autocrlf=true`, `git ls-files --eol` reports `i/lf w/crlf` for both compiler sources, both Root ABI inputs, and generated Root ABI outputs.
   - The source blobs are identical at base and candidate. Hashing the two candidate Git blobs gives `0aaf61d65153aadc4ddda1b36fa1b7bfb38373d52e8ba3299457cefe16864bff`, exactly the base compiler identity; hashing the Windows CRLF working-tree copies gives the candidate's `6f51b99ebd1b64f3045aff9a3bbd8047bd707ff2d5ec0c9b80e476b83d89e745`.
   - An LF checkout of `753920e` independently derives `compilerHash=0aaf61d...`, artifact `inputHash=bb95d870...`, and Root ABI `inputHash=696a58d0...`, while [`packages/index.json:1`](../packages/index.json#L1) publishes `6f51b99...`, `d2ed2c9e...`, and `50743b77...`. `python tools/lumio_contract.py validate` exits 1 immediately: published compiler `6f51b99...` versus locked compiler `0aaf61d...`.
   - The official Ubuntu workflow runs that exact validation at [`.github/workflows/repository-policy.yml:47`](../.github/workflows/repository-policy.yml#L47) and [`.github/workflows/repository-policy.yml:52`](../.github/workflows/repository-policy.yml#L52), so the candidate cannot pass repository policy.
   - A clean default Windows checkout also exits 1, for the complementary reason: generated output `packages/abi/lumio_core.h` is checked out CRLF with digest `d16947cd...`, while [`packages/abi/root-abi-bundle.json:1`](../packages/abi/root-abi-bundle.json#L1) records the generated LF digest `fa2aaca2...`. The candidate only validates immediately after generation creates LF outputs inside a checkout whose sources/inputs remain CRLF.

   Impact: acceptance items (1), (2), (3), and (6) are not met. The refresh turns a Windows checkout-materialization discrepancy into a committed identity regression and will block all dependent W1 pins.

   Required fix: do not merge `753920e`. First establish one repository-wide byte authority for every hashed compiler/input/generated file. The least semantic option is an explicitly authorized `.gitattributes` correction that forces LF for the relevant source/input/generated text paths; alternatively, changing hash normalization requires an ADR/contract decision because ADR-040 defines a byte digest. Then regenerate from a clean checkout and prove the same compiler/input/bundle identities on both Windows with `core.autocrlf=true` and the Ubuntu policy environment.

### P1 (Must Fix Before Merge)

1. **The implementer report records pre-commit working-tree success as if it were post-commit reproducibility.**

   [`w0-architecture-report.md:21`](../../w0-architecture-gate/.sdd/w0-architecture-report.md#L21) says fresh and formal trees compare byte-for-byte, and line 35 reports `validate` passing. Those claims are true only after generation has overwritten `packages/**` with LF while the input/source files remain CRLF. Neither a new default Windows checkout nor an LF checkout of the reported commit passes. The corrected report must distinguish generator-run working-tree bytes, committed Git blobs, and clean-checkout bytes, and must include a post-commit clean-checkout validation.

### P2 (Non-blocking)

- None beyond the explicit environment/test gaps below.

## R-00293 Adjudication

The implementer's substantive claim that R-00293 is already covered is **correct and independent of this candidate**. In an LF checkout of base `b7db298`, the full validator passes 201 fixtures, and the exact targeted negatives return:

- `replication/ack-smuggled-command`: pass-as-invalid; `playerCommand` is an unexpected additional body property.
- `replication/body-extra-member`: pass-as-invalid; `mvpVoxelPayload` is an unexpected additional body property.
- `replication/mapping-set-hash-type`: pass-as-invalid; `mappingSetHash: 42` is not a string.

These fixtures are registered at [`fixtures/index.json:857`](../fixtures/index.json#L857), line 869, and line 931, and ADR-045 names them as decision evidence at [`.spec/decisions/ADR-045-replication-body-closure.md:112`](../.spec/decisions/ADR-045-replication-body-closure.md#L112). No new R-00293 implementation is needed, but `753920e` cannot reach those fixture checks in an LF checkout because its published identity fails registry setup first.

## Independently Checked Evidence

| Command/check | Result |
|---|---|
| `git diff --stat --numstat b7db298 753920e` | 26 files; each is one JSON line replaced; no files outside `packages/**`. |
| Structured recursive JSON diff | Only compiler/input identity fields and the derived indexed Root ABI bundle digest changed; no semantic field changed. |
| Fresh Windows generation to `C:\Temp\lge-review-753920e-a` | Exit 0; 12 artifacts; candidate CRLF identities; stable outputHash. |
| Fresh tree versus `753920e` Git blobs | 70 files, 0 mismatches. |
| Descriptor/index/package/output/Root ABI blob consistency script | 12 artifacts, 0 issues; all outputHash values unchanged from base. |
| `python -m py_compile tools/lumio_contract.py tools/lumio_generate.py` | Exit 0. |
| `python tools/lumio_contract.py validate` in default Windows candidate checkout | Exit 1; `lumio_core.h` actual `d16947cd...` versus bundle `fa2aaca2...`. |
| Same validate in LF checkout of candidate | Exit 1; compiler actual `0aaf61d...` versus published `6f51b99...`. |
| Same validate in LF checkout of base | Exit 0; `Validated 201 fixture(s), 0 failure(s).` |
| Three R-00293 `--fixture ... --json` commands on base | Each exit 0 with the exact rejection errors listed above. |
| Baseline hashlib verification | Recorded and actual digest both `f1d36acf...`; match. |
| Junction-assisted `node .spec/tools/spec-lint.mjs` | Exit 0, `spec-lint: OK`. |
| Unmodified `node --test .spec/tools/spec-lint.test.mjs` | Exit 1; 13/13 abort at fixture setup with Windows `EPERM` creating symlinks. |

## Cannot Verify / Deliberately Not Repeated

- The actual GitHub Actions Ubuntu job was not run because this review is forbidden to push or open a PR. The LF checkout executes the policy's exact Python validation path far enough to prove it will fail before fixture execution.
- The official Node test suite cannot execute unmodified on this Windows host without symlink privilege/Developer Mode. This remains an environment gap, not a repository test failure.
- Rust tests/clippy and C# builds were not repeated: no generated Rust/C# source or any artifact outputHash changed, and the focused identity checks already found the release-blocking defect.

## Exact Next Action

1. Reject `753920e` and keep W1 consumers blocked.
2. Authorize a narrow line-ending authority fix (prefer explicit LF checkout rules for all byte-hashed compiler inputs and generated text outputs; do not silently redefine ADR-040 digest semantics).
3. Revert/regenerate the 26 identities from a clean checkout after that fix, then validate a new commit in two fresh environments: default Windows Git settings and LF/Ubuntu.
4. On Windows, use the three local junctions only for `node .spec/tools/spec-lint.mjs`; record the unmodified Node test EPERM as a platform gap.
5. Run the complete unmodified `.github/workflows/repository-policy.yml` on Ubuntu CI. W0 can be released only when that job is green and both clean-checkout Python validations publish the same compiler/input/Root ABI identities.
