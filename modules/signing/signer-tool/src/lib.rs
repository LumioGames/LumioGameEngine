//! lumio-core-signer-tool——离线/CI Signer（ADR 0002 四安全域之一，规格 §11.2）。
//! 签名载荷只覆盖架构源定义的 Canonical ManifestBody 精确字节；Signer、私钥 Provider、
//! 测试密钥与 evidence generator 绝不进入运行时发布产物。
//!
//! 脚手架状态（LCE-P0-001）：SignatureEnvelope 密码学 Profile（AG-004：签名输入、编码、
//! 公钥容器、域分隔、拒绝优先级）与只读 trust metadata Schema（AG-007）未冻结。
//! 不得自行选择 raw/DER、prehash 或 key 文件格式，因此本 crate 刻意不含任何模块与公共项。
