//! `CompositionError`——仓内错误面（规格 §6.2、§7.3）。
//!
//! 这些不是公共 ErrorCode：公共错误语义的唯一来源是架构源，本 crate 的错误只用于
//! 本仓工具的失败分类与退出码，不得被投影成 wire 上的错误值。

use std::fmt;

/// 失败分类（规格 §7.3 枚举，逐项一一对应，不增删）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionErrorKind {
    InvalidConfiguration,
    ArchitectureLockMismatch,
    SourceCommitMismatch,
    SourceTreeDigestMismatch,
    DirtySourceTree,
    UnknownFeature,
    FeatureConflict,
    ToolchainMismatch,
    TargetProfileReferenceMismatch,
    TargetNotApplicable,
    RootAbiContractUnavailable,
    NonDeterministicPlan,
    OutputAlreadyExists,
    AtomicPublishFailed,
    BlockedOnArchitectureGate,
}

impl CompositionErrorKind {
    /// 规格 §7.4 的 CLI 退出码：2 配置；3 Source/Feature/Toolchain 漂移；
    /// 4 冻结失败；5 Architecture Gate。仓内工具退出码，不是公共 ErrorCode。
    pub fn exit_code(self) -> u8 {
        match self {
            CompositionErrorKind::InvalidConfiguration => 2,
            CompositionErrorKind::ArchitectureLockMismatch
            | CompositionErrorKind::SourceCommitMismatch
            | CompositionErrorKind::SourceTreeDigestMismatch
            | CompositionErrorKind::DirtySourceTree
            | CompositionErrorKind::UnknownFeature
            | CompositionErrorKind::FeatureConflict
            | CompositionErrorKind::ToolchainMismatch
            | CompositionErrorKind::TargetProfileReferenceMismatch
            | CompositionErrorKind::TargetNotApplicable => 3,
            CompositionErrorKind::NonDeterministicPlan
            | CompositionErrorKind::OutputAlreadyExists
            | CompositionErrorKind::AtomicPublishFailed => 4,
            CompositionErrorKind::RootAbiContractUnavailable
            | CompositionErrorKind::BlockedOnArchitectureGate => 5,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            CompositionErrorKind::InvalidConfiguration => "InvalidConfiguration",
            CompositionErrorKind::ArchitectureLockMismatch => "ArchitectureLockMismatch",
            CompositionErrorKind::SourceCommitMismatch => "SourceCommitMismatch",
            CompositionErrorKind::SourceTreeDigestMismatch => "SourceTreeDigestMismatch",
            CompositionErrorKind::DirtySourceTree => "DirtySourceTree",
            CompositionErrorKind::UnknownFeature => "UnknownFeature",
            CompositionErrorKind::FeatureConflict => "FeatureConflict",
            CompositionErrorKind::ToolchainMismatch => "ToolchainMismatch",
            CompositionErrorKind::TargetProfileReferenceMismatch => {
                "TargetProfileReferenceMismatch"
            }
            CompositionErrorKind::TargetNotApplicable => "TargetNotApplicable",
            CompositionErrorKind::RootAbiContractUnavailable => "RootAbiContractUnavailable",
            CompositionErrorKind::NonDeterministicPlan => "NonDeterministicPlan",
            CompositionErrorKind::OutputAlreadyExists => "OutputAlreadyExists",
            CompositionErrorKind::AtomicPublishFailed => "AtomicPublishFailed",
            CompositionErrorKind::BlockedOnArchitectureGate => "BlockedOnArchitectureGate",
        }
    }
}

#[derive(Debug)]
pub struct CompositionError {
    kind: CompositionErrorKind,
    message: String,
}

impl CompositionError {
    pub fn new(kind: CompositionErrorKind, message: impl Into<String>) -> Self {
        CompositionError {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> CompositionErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{}]: {}", self.kind.as_str(), self.message)
    }
}

impl std::error::Error for CompositionError {}

/// 内部构造捷径：`invalid("…")` 比到处写全名可读。
pub(crate) fn invalid(message: impl Into<String>) -> CompositionError {
    CompositionError::new(CompositionErrorKind::InvalidConfiguration, message)
}

pub(crate) fn err(kind: CompositionErrorKind, message: impl Into<String>) -> CompositionError {
    CompositionError::new(kind, message)
}
