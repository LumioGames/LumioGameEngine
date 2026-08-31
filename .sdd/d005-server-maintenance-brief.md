# D-005 Consumer Track: R-00245 Maintenance Dual Durable Ack

Target repository: `C:\Work\LumioGames\LumioServer`.

Read first: `C:\Users\g923\AppData\Local\Temp\d005-cards\R-00245.md` (complete read-back). Architecture authority is commits `c14df420ac05b0d23f1fb674977b9a4c957edac5` / `f71cac137733b7f1609ae8235676d44c9f324858`, source hashes `d69c69374ef960b1968f0e8b2fdd4195d1abd52ed5ab34fd00b406fa85f141f1` and `82ed79a72ced56913c79ffa0bfb6d3763221ff2312c13c4a4d34f56e89b56f7c`, Baseline `LGE-V1.4-2026-08-27`.

This track depends on R-00242, R-00244, R-00236, R-00227, and R-00233, all of which must be read in full and have actual implementation evidence. Preserve independent PersistenceCommitAck/AuditDurableAck semantics and D-005 profile selection; never add TargetActivated, implicit defaults, direct spawn/sleep, or a second contract. If any dependency or the maintenance-agent crate is absent, stop `BLOCKED` with exact read-only evidence and no edits. If all are present, implement only the four listed files with deterministic TDD.

Report path: `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\d005-server-maintenance-report.md`.
