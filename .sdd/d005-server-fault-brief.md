# D-005 Consumer Track: R-00236 Durability Fault Matrix

Target repository: `C:\Work\LumioGames\LumioServer`.

Read first: `C:\Users\g923\AppData\Local\Temp\d005-cards\R-00236.md` (complete read-back). Use architecture commits `c14df420ac05b0d23f1fb674977b9a4c957edac5` / `f71cac137733b7f1609ae8235676d44c9f324858`, recorded source hashes `d69c69374ef960b1968f0e8b2fdd4195d1abd52ed5ab34fd00b406fa85f141f1` and `82ed79a72ced56913c79ffa0bfb6d3763221ff2312c13c4a4d34f56e89b56f7c`, Baseline `LGE-V1.4-2026-08-27`.

R-00231 is a hard implementation dependency for this test-only track. First inspect the target workspace and dependency evidence. The matrix must classify complete, async-flush, and snapshot-only outcomes without choosing an implicit tier, and must never assert success after a fault. If the persistence-host crate/Recovery implementation is absent, stop `BLOCKED` with read-only evidence; do not add production code, test-only fake contracts, or files outside the two card paths. If present, add only the two listed tests and run them deterministically.

Report path: `C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\d005-server-fault-report.md`.
