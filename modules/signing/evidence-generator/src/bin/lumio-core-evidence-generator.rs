//! `lumio-core-evidence-generator`——证据生成 CLI（子命令 generate，规格 §3.4）。
//!
//! 脚手架守卫（LCE-P0-001）：Evidence Profile（AG-011）未冻结，上游 FrozenBuildPlan
//! （LCE-P0-004）与平台 staging（LCE-P0-008）也未建立。按规格 §3.4，Gate 输入缺失时
//! 必须以结构化 `BlockedOnArchitectureGate` 仓内工具错误终止，不得以工具默认输出
//! 冒充证据 profile。

use std::process::ExitCode;

/// 与 composition CLI 对齐（规格 §7.4）：5 = Architecture Gate；仓内工具退出码，非公共 ErrorCode。
const EXIT_BLOCKED_ON_ARCHITECTURE_GATE: u8 = 5;

fn main() -> ExitCode {
    eprintln!(
        "lumio-core-evidence-generator: error[BlockedOnArchitectureGate]: \
         Evidence Profile（AG-011）未冻结，FrozenBuildPlan（LCE-P0-004）与平台 staging\
         （LCE-P0-008）未建立；拒绝以工具默认输出冒充证据 profile"
    );
    ExitCode::from(EXIT_BLOCKED_ON_ARCHITECTURE_GATE)
}
