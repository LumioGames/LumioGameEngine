//! `AbiGenerationError`——生成期仓内错误面（规格 §8.3）。
//!
//! 这些不是公共 ErrorCode：公共错误语义的唯一来源是架构源，本类型只用于本仓工具的
//! 失败分类与退出码（规格 §6.2）。

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiGenerationErrorKind {
    /// 输入配置不合法（路径缺失、lock 不可解析等）。
    InvalidConfiguration,
    /// 锁定 compiler 的 SHA-256 与上游 bundle 声明的 `compiler.digest` 不符。
    CompilerDigestMismatch,
    /// 调用锁定 compiler 失败（进程起不来、非零退出、输出不可解析）。
    CompilerInvocationFailed,
    /// 输入集合摘要与上游 bundle 声明的 `inputHash` 不符。
    InputHashMismatch,
    /// 某份产物摘要与上游 bundle 声明的 `outputFiles[].digest` 不符，或回读时已被改动。
    OutputHashMismatch,
    /// 生成目录里出现未登记文件。
    UnregisteredFile,
    /// 目标目录已存在——已发布的生成物不可覆盖。
    OutputAlreadyExists,
    /// 原子发布失败。
    AtomicPublishFailed,
    /// AG-001 未关闭：上游 Root ABI bundle 不可用。
    BlockedOnArchitectureGate,
}

impl AbiGenerationErrorKind {
    /// 仓内工具退出码（与 composition 同一口径：2 配置；3 漂移；4 发布；5 Gate）。
    pub fn exit_code(self) -> u8 {
        match self {
            AbiGenerationErrorKind::InvalidConfiguration => 2,
            AbiGenerationErrorKind::CompilerDigestMismatch
            | AbiGenerationErrorKind::CompilerInvocationFailed
            | AbiGenerationErrorKind::InputHashMismatch
            | AbiGenerationErrorKind::OutputHashMismatch
            | AbiGenerationErrorKind::UnregisteredFile => 3,
            AbiGenerationErrorKind::OutputAlreadyExists
            | AbiGenerationErrorKind::AtomicPublishFailed => 4,
            AbiGenerationErrorKind::BlockedOnArchitectureGate => 5,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            AbiGenerationErrorKind::InvalidConfiguration => "InvalidConfiguration",
            AbiGenerationErrorKind::CompilerDigestMismatch => "CompilerDigestMismatch",
            AbiGenerationErrorKind::CompilerInvocationFailed => "CompilerInvocationFailed",
            AbiGenerationErrorKind::InputHashMismatch => "InputHashMismatch",
            AbiGenerationErrorKind::OutputHashMismatch => "OutputHashMismatch",
            AbiGenerationErrorKind::UnregisteredFile => "UnregisteredFile",
            AbiGenerationErrorKind::OutputAlreadyExists => "OutputAlreadyExists",
            AbiGenerationErrorKind::AtomicPublishFailed => "AtomicPublishFailed",
            AbiGenerationErrorKind::BlockedOnArchitectureGate => "BlockedOnArchitectureGate",
        }
    }
}

#[derive(Debug)]
pub struct AbiGenerationError {
    kind: AbiGenerationErrorKind,
    message: String,
}

impl AbiGenerationError {
    pub fn new(kind: AbiGenerationErrorKind, message: impl Into<String>) -> Self {
        AbiGenerationError {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> AbiGenerationErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AbiGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{}]: {}", self.kind.as_str(), self.message)
    }
}

impl std::error::Error for AbiGenerationError {}

pub(crate) fn err(kind: AbiGenerationErrorKind, message: impl Into<String>) -> AbiGenerationError {
    AbiGenerationError::new(kind, message)
}

pub(crate) fn invalid(message: impl Into<String>) -> AbiGenerationError {
    AbiGenerationError::new(AbiGenerationErrorKind::InvalidConfiguration, message)
}
