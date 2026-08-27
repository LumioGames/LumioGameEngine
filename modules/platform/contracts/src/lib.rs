//! lumio-core-platform-contracts——平台运行时安全契约（PackagePath、OpenedArtifactSet、
//! LoadBackend，规格 §9.3）；控制文件语义（ManifestBody/ArtifactIndex/SignatureEnvelope）
//! 由架构源 Schema 拥有，本 crate 只定义仓内接口。
//!
//! 脚手架状态（LCE-P0-001）：实现卡片 LCE-P0-007 未开工，且其输入依赖只读架构镜像
//! （LCE-P0-002）与生成 ContractTypes（LCE-P0-003，受 LGE-GATE-P0-001/-002 阻塞）。
//! 契约输入就位前不预发布接口形状，因此本 crate 刻意不含任何模块与公共项。
