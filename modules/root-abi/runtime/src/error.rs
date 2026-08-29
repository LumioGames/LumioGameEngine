//! 失败到稳定 ErrorCode 的映射（规格 §8.2 `error.rs`、§8.3 错误映射、§6.2）。
//!
//! 本 crate **不发明错误码**：公共语义的唯一来源是架构源 ID Registry，经
//! `lumio-core-contracts` 消费。`RootAbiErrorKind` 只是仓内失败分类，用于诊断，
//! 不参与任何跨仓契约；跨边界可断言的只有 [`RootAbiError::code`]。

use std::fmt;

use lumio_core_contracts::ErrorCode;

/// 仓内失败分类（诊断用；契约面是 [`RootAbiError::code`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootAbiErrorKind {
    /// 唯一 entry symbol 在镜像里不存在。
    EntrySymbolMissing,
    /// 唯一 entry symbol 解析到多个候选（ADR-006：一个包只允许一个 root entry）。
    EntrySymbolCollision,
    /// entry 自身返回了非零 status，码值由被调方给出。
    EntryRejected,
    /// ABI 身份、版本、大小、能力位、指针宽度、endianness 或布局不匹配。
    AbiMismatch,
    /// Handle 已失效（换代或已释放）。
    HandleInvalid,
    /// Handle 重复释放。
    HandleDoubleRelease,
}

/// 绑定期与 Handle 期的仓内错误；`code()` 是唯一跨边界稳定的部分。
#[derive(Debug, Clone)]
pub struct RootAbiError {
    code: ErrorCode,
    kind: RootAbiErrorKind,
    message: String,
}

impl RootAbiError {
    fn new(code: ErrorCode, kind: RootAbiErrorKind, message: impl Into<String>) -> Self {
        Self {
            code,
            kind,
            message: message.into(),
        }
    }

    /// 架构源已登记的稳定 ErrorCode——跨仓可断言的唯一契约面。
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// 仓内失败分类，只用于诊断。
    pub fn kind(&self) -> RootAbiErrorKind {
        self.kind
    }

    /// 仓内诊断消息，不是契约；不得据其文本做判定。
    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn entry_symbol_missing(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::SymbolMissing,
            RootAbiErrorKind::EntrySymbolMissing,
            message,
        )
    }

    pub(crate) fn entry_symbol_collision(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::SymbolCollision,
            RootAbiErrorKind::EntrySymbolCollision,
            message,
        )
    }

    /// entry 返回了架构源**已登记**的非零码：原样透传，不重新分类。
    pub(crate) fn entry_rejected(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(code, RootAbiErrorKind::EntryRejected, message)
    }

    pub(crate) fn abi_mismatch(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::NativeAbiMismatch,
            RootAbiErrorKind::AbiMismatch,
            message,
        )
    }

    pub(crate) fn invalid_handle(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::InvalidHandle,
            RootAbiErrorKind::HandleInvalid,
            message,
        )
    }

    pub(crate) fn handle_double_release(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::HandleDoubleRelease,
            RootAbiErrorKind::HandleDoubleRelease,
            message,
        )
    }
}

impl fmt::Display for RootAbiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}({}) {}: {}",
            self.code,
            self.code.numeric(),
            match self.kind {
                RootAbiErrorKind::EntrySymbolMissing => "EntrySymbolMissing",
                RootAbiErrorKind::EntrySymbolCollision => "EntrySymbolCollision",
                RootAbiErrorKind::EntryRejected => "EntryRejected",
                RootAbiErrorKind::AbiMismatch => "AbiMismatch",
                RootAbiErrorKind::HandleInvalid => "HandleInvalid",
                RootAbiErrorKind::HandleDoubleRelease => "HandleDoubleRelease",
            },
            self.message
        )
    }
}

impl std::error::Error for RootAbiError {}
