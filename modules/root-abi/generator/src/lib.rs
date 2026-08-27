//! lumio-core-root-abi-generator——ABI 生成 Adapter（generate / verify API，规格 §8.3）。
//! 只消费锁定的架构源 compiler 与输入集合，输出只读发布；本仓不拥有任何模板或 slot 映射。
//!
//! 脚手架状态（LCE-P0-001）：AG-001 未关闭——架构源尚未发布可消费的 ABI compiler
//! 坐标与布局 Golden。Gate 关闭前只允许 blocked guard，因此本 crate 刻意不含任何
//! 模块与公共项。
