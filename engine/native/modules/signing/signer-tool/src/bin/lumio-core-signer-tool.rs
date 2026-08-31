//! `lumio-core-signer-tool`——签名 CLI（子命令 sign，规格 §3.4）。
//!
//! 脚手架守卫（LCE-P0-001）：SignatureEnvelope 密码学 Profile（AG-004）与 trust metadata
//! Schema（AG-007）未冻结。按规格 §3.4 与 §11.2，Gate 输入缺失时必须以结构化
//! `BlockedOnArchitectureGate` 仓内工具错误终止，不得自定 raw/DER、prehash 或 key 格式。

use std::process::ExitCode;

/// 与 composition CLI 对齐（规格 §7.4）：5 = Architecture Gate；仓内工具退出码，非公共 ErrorCode。
const EXIT_BLOCKED_ON_ARCHITECTURE_GATE: u8 = 5;

fn main() -> ExitCode {
    eprintln!(
        "lumio-core-signer-tool: error[BlockedOnArchitectureGate]: \
         SignatureEnvelope 密码学 Profile（AG-004）与 trust metadata Schema（AG-007）\
         未冻结；拒绝自定签名载荷、编码或 key 文件格式"
    );
    ExitCode::from(EXIT_BLOCKED_ON_ARCHITECTURE_GATE)
}
