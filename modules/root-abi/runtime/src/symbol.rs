//! 唯一 entry symbol 的解析接口（规格 §8.2 `symbol.rs`、§8.3）。
//!
//! 本 crate 不打开、不映射、不卸载任何镜像——那是 `platform-runtime` / `loader` 的
//! 职责。这里只定义「谁能把一个符号名换成进程内地址」，以及该实现必须承担的寿命义务。

use std::ffi::{c_void, CStr};
use std::ptr::NonNull;

/// 符号解析失败的原因；映射到 §8.3 的稳定 ErrorCode。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolLookupError {
    /// 镜像里没有这个符号 → `SymbolMissing`(1021)。
    NotFound,
    /// 符号解析到多个候选 → `SymbolCollision`(1022)。
    ///
    /// ADR-006：一个组合包的符号表里**恰好**有一个跨仓 root entry；解析到多个
    /// 是契约违反，不是可以「取第一个」的歧义。
    Collision,
}

/// 把符号名解析为进程内地址。
///
/// 实现对象必须同时拥有并保持对应 MappedNativeImage 的进程内生命周期：
/// [`crate::RootApiTableView`] 私有持有 `Arc<dyn SymbolResolver>`，正是靠这一点
/// 把 API 表的寿命绑定到常驻映像（规格 §8.3）。
pub trait SymbolResolver: Send + Sync + 'static {
    /// # Safety
    ///
    /// 调用方保证 `symbol` 是一个合法的 NUL 结尾符号名。实现方保证：返回的地址在
    /// `self` 存活期间始终有效，且指向 `symbol` 所命名的那个导出项——否则调用方
    /// 依此构造的函数指针会是悬垂的。
    unsafe fn resolve(&self, symbol: &CStr) -> Result<NonNull<c_void>, SymbolLookupError>;
}
