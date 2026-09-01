# D-005 Consumer Track: R-00141 Canonical Binary Codec

Target repository: `C:\Work\LumioGames\LumioGameRuntime`.

Read this first: `C:\Users\g923\AppData\Local\Temp\d005-cards\R-00141.md`. It is the complete Workflow card read-back, including the detail, all comments, attachments, and acceptance items. The card's historical MessagePack wording conflicts with the current architecture decision: `LumioBinV1` is authoritative and MessagePack is a rejected alternative. Follow the current architecture source, not the stale MessagePack line.

Architecture authority is the original bytes from architecture commit `c14df420ac05b0d23f1fb674977b9a4c957edac5` (also present in `f71cac137733b7f1609ae8235676d44c9f324858`):

- `docs/specs/lumio-save-design-overview.md`, SHA-256 `d69c69374ef960b1968f0e8b2fdd4195d1abd52ed5ab34fd00b406fa85f141f1`.
- `docs/specs/2026-08-30-save-load-architecture-decisions.md`, SHA-256 `82ed79a72ced56913c79ffa0bfb6d3763221ff2312c13c4a4d34f56e89b56f7c`.
- Baseline `LGE-V1.4-2026-08-27`.

Implement only the R-00141 file set listed in the card. Do not edit generated sources, shared manifests outside the listed project files, Workflow, or architecture sources. Implement the executable `LumioBinV1` primitive/record codec with checked input/output/depth budgets, deterministic declaration-order records, strict trailing/duplicate/UTF-8/type/range rejection, and tests for every applicable card acceptance item. Use generated `LumioBinForm`/golden declarations as read-only inputs. Follow TDD: write a focused failing test, run it and capture the expected failure, then implement the minimum code and run focused plus module verification. Do not add a second schema or infer a durability tier; this track owns encoding only.

Report path: `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\d005-canonical-report.md`. Write full evidence there (files, RED/GREEN commands and outputs, hashes, known gaps, exact commit). Return only status, commit, one-line test summary, concerns, and report path.
