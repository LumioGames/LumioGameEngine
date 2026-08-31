//! lumio-core-diagnostics——观测适配平面（ADR 0003：非生产模块，规格 §13）。
//! 只做 LoggingEvent 映射、Failure Evidence Fragment 组装与 Host EventSink Adapter；
//! 队列、批处理、落盘与 Bundle Assembly 归 Host。
//!
//! 脚手架状态（LCE-P0-001）：LoggingEvent/FailureBundle 公共 Schema 与只读架构镜像
//! （LCE-P0-002）未建立，Failure Evidence Fragment 的公共形态（AG-008）未冻结。
//! 不得创建第二套序列化契约，因此本 crate 刻意不含任何模块与公共项。
