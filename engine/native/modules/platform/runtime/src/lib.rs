//! lumio-core-platform-runtime——OS LoadBackend 实现（P0：Linux DynamicLibrary，
//! 规格 §9.5：安全打开 → sealed snapshot → 同对象验证/映射 → 永久 resident）。
//!
//! 脚手架状态（LCE-P0-001）：前置链未就位——平台契约（LCE-P0-007）、本地 ADR-0005
//! （Linux 同对象加载策略）与只读架构镜像（LCE-P0-002）均未建立。同对象加载语义在
//! ADR-0005 冻结前不得动工，因此本 crate 刻意不含任何模块与公共项。
