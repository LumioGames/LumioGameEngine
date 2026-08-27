//! `lumio-core-manifest`——Manifest CLI（子命令 generate / validate / print-digest，
//! 规格 §10.5）。
//!
//! 脚手架守卫（LCE-P0-001）：CanonicalSerializer 与 Digest Golden（AG-005）、
//! capabilitySetDigest 投影（AG-006）、ABI 生成记录（AG-001）、ArtifactIndex 投影
//! （AG-002）未发布。按规格 §3.4，Gate 输入缺失时必须以结构化
//! `BlockedOnArchitectureGate` 仓内工具错误终止，不得自定 canonical 格式。

use std::process::ExitCode;

/// 与 composition CLI 对齐（规格 §7.4）：5 = Architecture Gate；仓内工具退出码，非公共 ErrorCode。
const EXIT_BLOCKED_ON_ARCHITECTURE_GATE: u8 = 5;

fn main() -> ExitCode {
    eprintln!(
        "lumio-core-manifest: error[BlockedOnArchitectureGate]: \
         CanonicalSerializer/Digest Golden（AG-005）、capabilitySetDigest（AG-006）、\
         ABI 生成记录（AG-001）、ArtifactIndex 投影（AG-002）未发布；\
         拒绝自定 canonical 字节格式"
    );
    ExitCode::from(EXIT_BLOCKED_ON_ARCHITECTURE_GATE)
}
