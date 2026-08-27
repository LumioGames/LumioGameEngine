//! lumio-core-runtime-verifier——运行时包验证（ADR 0002：signing 四域中唯一进入运行时
//! 闭包的域，规格 §11.4）；对同一实际打开对象校验，唯一成功输出 VerifiedPackageDescriptor。
//!
//! 脚手架状态（LCE-P0-001）：验证语义的前置 Gate 均未关闭——密码学 Profile（AG-004）、
//! ArtifactIndex 投影（AG-002）、Canonical/Digest Golden（AG-005）、capabilitySetDigest
//! （AG-006）、trust metadata Schema（AG-007）、Evidence Profile（AG-011）。VPD 字段与
//! 检查项必须由生成 ContractTypes 决定，因此本 crate 刻意不含任何模块与公共项。
