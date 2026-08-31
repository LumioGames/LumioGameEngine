//! `lumio-core-smoke`——冒烟 CLI（子命令 verify-package / load / p0-slice，规格 §3.4、§14.7）。
//!
//! 脚手架守卫（LCE-P0-001）：被验证的整条 P0 链（BuildPlan/ABI/平台包/Manifest/签名/
//! Verifier/Loader）均未实现，公共输入受 LGE-GATE-P0-001/-002/-003 阻塞。按规格 §14.8，
//! 前置 Gate 缺失时相关 case 标 `BlockedOnArchitectureGate`、报告命令退出非零，
//! 不能把 blocked 当 passed。

use std::process::ExitCode;

/// 与 composition CLI 对齐（规格 §7.4）：5 = Architecture Gate；仓内工具退出码，非公共 ErrorCode。
const EXIT_BLOCKED_ON_ARCHITECTURE_GATE: u8 = 5;

fn main() -> ExitCode {
    eprintln!(
        "lumio-core-smoke: error[BlockedOnArchitectureGate]: \
         P0 链路上游未实现且公共输入受 LGE-GATE-P0-001/-002/-003 阻塞；\
         相关 case 标记 blocked，拒绝产出 passed 报告"
    );
    ExitCode::from(EXIT_BLOCKED_ON_ARCHITECTURE_GATE)
}
