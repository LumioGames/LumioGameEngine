//! lumio-core-smoke——验证平面（ADR 0003：非生产模块，规格 §14）。
//! 串起完整 P0 E2E（Source Lock → BuildPlan → ABI → 平台包 → Manifest → 签名 →
//! 验证 → LoaderLease），输出测试报告与 Fixture 引用。
//!
//! 脚手架状态（LCE-P0-001）：被验证的整条链均未实现，且其公共输入受
//! LGE-GATE-P0-001/-002/-003 阻塞。按规格 §14.8，前置 Gate 缺失时相关 case 只能标
//! blocked、报告退出非零，不得把 blocked 当 passed，因此本 crate 刻意不含任何模块
//! 与公共项。
