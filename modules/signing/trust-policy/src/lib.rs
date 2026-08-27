//! lumio-core-trust-policy——只读信任策略评估（ADR 0002 四安全域之一，规格 §11.3）；
//! 只读 trust metadata 与 trust decision，不拥有任何写路径。
//!
//! 脚手架状态（LCE-P0-001）：运行时只读 trust metadata 的公共 Schema/Fixture（AG-007）
//! 未发布——key encoding、时间/撤销规则由架构源冻结。不得本仓发布跨仓 TrustStore
//! 格式，因此本 crate 刻意不含任何模块与公共项。
