//! 平台 runtime 错误（规格 §9.7）。
//!
//! 公共错误语义的唯一来源是架构源：本类型**不发明** ErrorCode，只在每个变体上带一个
//! 生成的 `ErrorCode`（规格 §6.2）。仓内细节放在 message 里，跨边界只看 code。

use lumio_core_contracts::ErrorCode;

use crate::package_path::PackagePath;
use crate::ControlFileKind;

/// 打开 / 映射 package 时的失败。变体到 ErrorCode 的映射由规格 §9.7 固定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformRuntimeError {
    /// 安全打开失败或条目缺失。
    ArtifactMissing { path: PackagePath, detail: String },
    /// 控制文件缺失——三个控制文件是 package 的完整性前提。
    ControlFileMissing {
        kind: ControlFileKind,
        detail: String,
    },
    /// Host 与包的 TargetProfile 不匹配。
    TargetProfileMismatch { detail: String },
    /// 映射内存不足。
    LoaderOutOfMemory { detail: String },
    /// 超出 `OpenPackageRequest` 声明的字节上限。
    LimitExceeded {
        path: PackagePath,
        limit: u64,
        actual: u64,
    },
}

impl PlatformRuntimeError {
    /// 规格 §9.7 的运行时映射。`LimitExceeded` 归 ArtifactMissing：超限的条目对上层
    /// 等同于「拿不到这个 Artifact」，而不是另一类公共语义——本仓不得新增公共 ErrorCode。
    pub fn error_code(&self) -> ErrorCode {
        match self {
            PlatformRuntimeError::ArtifactMissing { .. }
            | PlatformRuntimeError::ControlFileMissing { .. }
            | PlatformRuntimeError::LimitExceeded { .. } => ErrorCode::ArtifactMissing,
            PlatformRuntimeError::TargetProfileMismatch { .. } => ErrorCode::TargetProfileMismatch,
            PlatformRuntimeError::LoaderOutOfMemory { .. } => ErrorCode::LoaderOutOfMemory,
        }
    }
}

impl std::fmt::Display for PlatformRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformRuntimeError::ArtifactMissing { path, detail } => {
                write!(f, "Artifact {path} 不可用：{detail}")
            }
            PlatformRuntimeError::ControlFileMissing { kind, detail } => {
                write!(f, "控制文件 {kind:?} 不可用：{detail}")
            }
            PlatformRuntimeError::TargetProfileMismatch { detail } => {
                write!(f, "TargetProfile 不匹配：{detail}")
            }
            PlatformRuntimeError::LoaderOutOfMemory { detail } => {
                write!(f, "映射内存不足：{detail}")
            }
            PlatformRuntimeError::LimitExceeded {
                path,
                limit,
                actual,
            } => write!(f, "Artifact {path} 超出上限：limit {limit}，实际 {actual}"),
        }
    }
}

impl std::error::Error for PlatformRuntimeError {}
