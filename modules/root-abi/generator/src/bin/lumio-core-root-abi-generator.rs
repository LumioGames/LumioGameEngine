//! `lumio-core-root-abi-generator`——ABI 生成 CLI（子命令 generate / verify-generated /
//! layout-report，规格 §8.4）。
//!
//! 脚手架守卫（LCE-P0-001）：AG-001 未关闭——架构源未发布 ABI compiler 的名称/版本/摘要
//! 与布局 Golden。按规格 §8.3/§3.4，此时只能以结构化 `BlockedOnArchitectureGate`
//! 仓内工具错误终止，不得回退本仓模板。

use std::process::ExitCode;

/// 与 composition CLI 对齐（规格 §7.4）：5 = Architecture Gate；仓内工具退出码，非公共 ErrorCode。
const EXIT_BLOCKED_ON_ARCHITECTURE_GATE: u8 = 5;

fn main() -> ExitCode {
    eprintln!(
        "lumio-core-root-abi-generator: error[BlockedOnArchitectureGate]: \
         AG-001 未关闭：架构源未发布 ABI compiler 坐标与 C/Rust/C# 布局 Golden；\
         拒绝回退本仓模板生成"
    );
    ExitCode::from(EXIT_BLOCKED_ON_ARCHITECTURE_GATE)
}
