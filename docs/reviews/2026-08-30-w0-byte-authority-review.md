# W0 Architecture Gate: Byte Authority Review

Date: 2026-08-30

## Verdict

Candidate commit `753920e35b5c6cd590063a9febe8e9254a3ae6e6` on `Go1c/w0-architecture-gate` is rejected. It republishes 26 generator-owned JSON files with identities derived from a CRLF working tree. The candidate is internally self-consistent in that working tree, but it is not reproducible from the repository blobs and must not be merged or used for downstream pins.

## Finding

`tools/lumio_generate.py` hashes the raw bytes of the compiler sources and JSON inputs, and `dir_output_hash()` hashes raw generated package bytes. The previous `* text=auto` rule allowed `core.autocrlf=true` to materialize those files as CRLF. The candidate therefore recorded:

```text
candidate compilerHash       6f51b99ebd1b64f3045aff9a3bbd8047bd707ff2d5ec0c9b80e476b83d89e745
candidate inputHash          d2ed2c9e4046fe7bd5ed81e2dd74ef02db6a5671cb971e9163835f763f87bb2f
candidate Root ABI digest    708ccb7e1bd25cb3c66caa3a13bdadfa5446ff4403a0d043333f59e737eae583
```

An LF checkout of the same source derives different identities and fails `python tools/lumio_contract.py validate` against the candidate. The candidate changes no public schema semantics; its failure is a repository byte-materialization defect.

## Corrective change

The main worktree adds explicit path-scoped `eol=lf` attributes for the compiler sources, hashed JSON inputs, and every file under `packages/**` (the generated Root ABI and package outputs). The broad `*` rule is retained; changing it to `eol=lf` globally was tested and rejected because this Windows checkout represents `docs/adr/*` symlink entries as regular link placeholders and the broad rule exposes unrelated type changes.

No generated package, Schema, ID, Fixture, Baseline, or public wire field is edited by this correction. The rejected 753920e package metadata remains isolated on its branch.

## Fresh-checkout evidence

Two fresh clones were created from the current repository plus the attribute correction. One used `core.autocrlf=true` (Windows simulation); the other used `core.autocrlf=false` (LF simulation). Both checked out every hashed text file as LF and produced the same results:

```text
compilerHash       0aaf61d65153aadc4ddda1b36fa1b7bfb38373d52e8ba3299457cefe16864bff
inputHash          bb95d87078c83b40e5148f58d68aa7a1df7cded94d28657a0f11e4f1231c2ff9
Root ABI digest    02dce705a9a6fe7a437ed2e4137b03de7341ed614f30f10b614659c5226184a7
contract-runtime-rust outputHash 3f9357242b67ce513cd3e1c102f9e96d7402922ba0a04ec976ed70a60d45cc52
```

| Check | Windows simulation | LF simulation |
|---|---:|---:|
| `python tools/lumio_contract.py validate` | exit 0; 201 fixtures, 0 failures | exit 0; 201 fixtures, 0 failures |
| fresh `generate` vs checked-in package tree | 70 vs 70; 0 mismatches | 70 vs 70; 0 mismatches |
| `python tools/lumio_kat.py` | exit 0; Rust/C#/hashlib agree | exit 0; Rust/C#/hashlib agree |
| `git status --short` after checkout | clean | clean |

The direct Windows Node spec-lint limitation is separate: the existing checkout has junction/symlink restrictions. The junction-compatible local fixture helper runs all 13 tests; an unmodified symlink fixture can still return `EPERM` without Developer Mode. This is recorded as an environment gap and is not used as evidence that the Ubuntu policy job passed.

## Remaining gate

The `.gitattributes` correction is committed as `e1705e9` on `Go1c/w0-byte-authority-fix` and integrated on `main` as `6e3d80b`; it was re-verified from fresh Windows/LF clones. The official Ubuntu repository-policy run is still pending. Only after that gate may the canonical identities above be communicated for coordinated downstream re-pin. The cross-repo re-pin notification is therefore still outstanding. W1 consumers remain blocked until the byte authority and pin are released.
