//! lumio-core-evidence-generator——SBOM/License/Provenance 证据生成（ADR 0002 四安全域
//! 之一，规格 §11.1）；只消费 FrozenBuildPlan 与平台 staging，绝不进入运行时闭包。
//!
//! 脚手架状态（LCE-P0-001）：Evidence Profile（AG-011）未冻结——接受的规范版本、媒体
//! 类型、subject 覆盖与 verifier 语义检查均由架构源定义。不得把工具默认输出版本提升为
//! 公共互操作语义，因此本 crate 刻意不含任何模块与公共项。
