//! lumio-core-contracts——架构源生成 ContractTypes 的唯一 re-export 面
//! （Digest256、ErrorCode、CapabilityId、PackageIdentity 等，规格 §6.1）。
//!
//! 脚手架状态（LCE-P0-001）：生成制品来自架构源（LCE-P0-003），前置 Gate
//! LGE-GATE-P0-001（ABI generated contracts）与 LGE-GATE-P0-002（canonical/digest
//! profiles）未关闭，只读架构镜像（LCE-P0-002）也尚未建立。Gate 关闭前不得创建
//! 同名临时 struct，因此本 crate 刻意不含任何模块与公共项。
