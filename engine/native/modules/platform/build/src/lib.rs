//! lumio-core-platform-build——唯一 build/link/layout/ArtifactIndex 执行入口
//! （execute_build / finalize_platform / archive_platform，ADR 0001、规格 §9.4）；
//! 不回写 BuildPlan，全仓只有本 package 调用 cargo/rustc/linker。
//!
//! 脚手架状态（LCE-P0-001）：输入链未就位——FrozenBuildPlan（LCE-P0-004、ADR-0006）、
//! ABI 生成物（AG-001）与 ArtifactIndex 公共投影（AG-002）均未建立。AG-002 未关闭时
//! finalize 必须在写公共 index 前失败，因此本 crate 刻意不含任何模块与公共项。
