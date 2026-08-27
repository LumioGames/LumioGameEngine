//! `lumio-core-compose`——composition CLI（子命令 compose / verify / print，规格 §7.4）。
//!
//! 脚手架守卫（LCE-P0-001）：本 CLI 消费的架构输入（architecture.lock.json 与只读架构镜像
//! LCE-P0-002、生成 ContractTypes LGE-GATE-P0-001/-002）尚未在本仓建立。按规格 §3.4，
//! Gate 输入缺失时必须以结构化 `BlockedOnArchitectureGate` 仓内工具错误终止，不得回退临时格式。

use std::process::ExitCode;

/// 规格 §7.4：5 = Architecture Gate。这是仓内工具退出码，不是公共 ErrorCode。
const EXIT_BLOCKED_ON_ARCHITECTURE_GATE: u8 = 5;

fn main() -> ExitCode {
    eprintln!(
        "lumio-core-compose: error[BlockedOnArchitectureGate]: \
         架构输入尚未在本仓建立（architecture lock/只读镜像：LCE-P0-002；\
         生成 ContractTypes：LGE-GATE-P0-001/-002）；拒绝以临时格式继续"
    );
    ExitCode::from(EXIT_BLOCKED_ON_ARCHITECTURE_GATE)
}
