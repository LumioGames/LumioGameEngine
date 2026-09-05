# R5-01 Required Fix Before Review

The current delivery (`aaa0175`) is incomplete and must not proceed to review or acceptance yet.

Implement the full authoritative `R-00406` card, not only the timer adapter:

1. Update `engine/wire/gameplay-command-envelope-v1.json` to the exact C-1\" message set: `Welcome`, `WorldChange`, `InputCommand`, `ConnectionSuperseded`, `Error`; all entity/connection IDs are 128-bit; remove `FullSnapshot`, `Delta`, `entity.identity`, `chat.event`, and `chat.component`; preserve `roomSequence`; add `limits.createsPerPack`.
2. Update `engine/wire/entity-binding-and-query-v1.json` for async admit results without `netEntityId`, remove `listBindings`, update declaration-table placeholder/sha shape, derived `entityType`, derived `tombstoned`, and the `claim` section.
3. Extend `eng/verify-wire.mjs` with positive and negative assertions for the new C-1/C-2 requirements and recompute both schema hashes.
4. Complete the NativeLoader timer wrapper and tests, but do not use a second loader/module handle or an unverified reflection fallback. Keep the public wrapper within the existing NativeLoader ownership model.
5. Synchronize `ecs.md`, `ecs-entity-chat.md`, `lessons.md`, and the four ADR revision-record sections exactly as the card requires. Do not alter ADR decision/status sections.
6. Make `eng/dev-build.sh` and `eng/dev-run.sh` mode 100755 and add a first-line non-Linux `BLOCKED` message; discover `LumioVoxelEngine`/`LumioNativeCore` through environment variables or repository-relative paths.

Stay strictly within the R-00406 ownership scope. Add/update focused tests first, run the card's complete verification commands, commit the changes, and append a complete five-part handback plus TDD evidence to `C:/Work/LumioGames/LumioGameEngineArchitecture/.sdd-scratch/task-r5-01-report.md` (including any remaining concerns). Do not report DONE while any item above is missing.
