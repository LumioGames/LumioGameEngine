//! lumio-core-loader——Loader 状态机与 LoaderLease（规格 §12）：
//! `Uninitialized -> Preflighting -> Verified -> Binding -> ApiReady -> Leased -> Released`，
//! 失败进入回滚状态；首次成功 Acquire 锁定进程唯一 PackageIdentity。
//!
//! 脚手架状态（LCE-P0-001）：整条上游链未就位——runtime-verifier/trust-policy/
//! platform-runtime/root-abi 的契约输入（LGE-GATE-P0-001/-002/-003）以及 Loader 重入与
//! 错误优先级 Fixture（AG-009）、对外同步结果公共形态（AG-008）均未冻结。
//! 状态语义必须由架构源 Fixture 决定，因此本 crate 刻意不含任何模块与公共项。
