//! `lumio-core-platform-build`——平台构建 CLI（子命令 build-staging / finalize / archive /
//! verify-layout，规格 §9.7）。
//!
//! 脚手架守卫（LCE-P0-001）：上游输入未建立——FrozenBuildPlan（LCE-P0-004/ADR-0006）、
//! ABI 生成物（AG-001）与 ArtifactIndex 公共投影（AG-002）。按规格 §3.4，Gate 输入缺失时
//! 必须以结构化 `BlockedOnArchitectureGate` 仓内工具错误终止，不得发布半成品。

use std::process::ExitCode;

/// 与 composition CLI 对齐（规格 §7.4）：5 = Architecture Gate；仓内工具退出码，非公共 ErrorCode。
const EXIT_BLOCKED_ON_ARCHITECTURE_GATE: u8 = 5;

fn main() -> ExitCode {
    eprintln!(
        "lumio-core-platform-build: error[BlockedOnArchitectureGate]: \
         上游输入未建立：FrozenBuildPlan（LCE-P0-004/ADR-0006）、ABI 生成物（AG-001）、\
         ArtifactIndex 公共投影（AG-002）；拒绝构建或发布半成品"
    );
    ExitCode::from(EXIT_BLOCKED_ON_ARCHITECTURE_GATE)
}
