# R5-01 Review Findings

Fix all of these before re-review:

1. **P1 resource safety:** `NativeLoaderTimerAbi.Load` acquires `NativeEngineLease` and then performs `NativeLibrary.GetExport`, delegate conversion, table marshaling, and validation without a `try/finally`. Any exception after acquisition leaks the lease/native module. Ensure every post-acquisition failure disposes the lease, including missing export, malformed table, and invalid function pointer cases. Add focused test coverage for the failure path if practical.
2. **P1 acceptance gap:** the card explicitly requires a unit test covering rejection when a required `timer_*` root-table slot is missing. Add a testable root-table validation seam/helper and a real unit test that constructs a missing-slot table and asserts rejection; do not rely only on source-text inspection.
3. **C-1 coverage gap:** add positive/negative fixtures and verifier assertions for the card's required probes: destroy records; field changes with `sync`/`correction` and Owner visibility semantics; and Welcome-before-WorldChange ordering (including rejection of reversed order). Keep all IDs 128-bit and preserve the exact five-message set. Do not restore legacy mappings.

Re-run the focused verifier tests, `spec-lint`, and NativeLoader tests. Append the fix details and actual outputs to `.sdd-scratch/task-r5-01-report.md`, commit the fix, and report the new commit SHA.
