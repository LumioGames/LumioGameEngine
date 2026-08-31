# D-005 Consumer Track: R-00231 Recovery/Checkpoint/Migration

Target repository: `C:\Work\LumioGames\LumioServer`.

Read first: `C:\Users\g923\AppData\Local\Temp\d005-cards\R-00231.md` (complete detail/comments/attachments/acceptance read-back). Architecture bytes and baseline are the same as the D-005 briefs: commits `c14df420ac05b0d23f1fb674977b9a4c957edac5` / `f71cac137733b7f1609ae8235676d44c9f324858`, document hashes `d69c69374ef960b1968f0e8b2fdd4195d1abd52ed5ab34fd00b406fa85f141f1` and `82ed79a72ced56913c79ffa0bfb6d3763221ff2312c13c4a4d34f56e89b56f7c`, Baseline `LGE-V1.4-2026-08-27`.

This is a dependent preflight: R-00228 and R-00212 must be read and their actual implementation evidence checked before any code. Do not invent a migration DAG, wall-clock source, listener gate, snapshot format, or public contract. The current target inventory has no persistence-host crate. If the dependency or crate shell is absent, report `BLOCKED` without edits, citing exact evidence and the safe upstream owner. If all prerequisites are verifiably present, implement only the five listed files and acceptance behaviors with TDD.

Report path: `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\d005-server-recovery-report.md`.
