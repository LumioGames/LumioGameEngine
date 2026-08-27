//! composition——把锁定的 Source、Feature、TargetProfile 与工具链约束解析为不可变 BuildPlan
//! 与 ProvenanceRecord；不 clone、不编译、不链接（ADR 0001、规格 §7）。
//!
//! 脚手架状态（LCE-P0-001）：契约输入（architecture lock 与只读架构镜像 LCE-P0-002、
//! 生成 ContractTypes LCE-P0-003 / LGE-GATE-P0-001、-002）尚未在本仓建立。
//! 在公共输入存在之前暴露接口等于私定公共语义，因此本 crate 刻意不含任何模块与公共项。
