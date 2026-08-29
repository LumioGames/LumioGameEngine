//! `LoadBackend` trait（规格 §9.2、§9.3）。

use std::path::PathBuf;
use std::sync::Arc;

use crate::error::PlatformRuntimeError;
use crate::opened_set::OpenedArtifactSet;
use crate::package_path::PackagePath;

/// 打开 package 的请求。
///
/// 两个上限是**必填**而非 Option：没有上限的解析面对恶意包就是一个解压炸弹入口，
/// 而「忘了设上限」和「有意不设上限」在 Option 里长得一模一样。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPackageRequest {
    pub package_root: PathBuf,
    pub maximum_control_file_bytes: u64,
    pub maximum_artifact_bytes: u64,
}

/// 已映射的原生映像。
///
/// 内部表示由 platform-runtime 拥有（LCE-P0-014），本 crate 只提供不透明句柄类型，
/// 使 Loader 见不到 OS handle（规格 §9.1 非职责：不泄漏 OS handle）。
/// `SymbolResolver` 的实现同样在 platform-runtime 侧，本 crate 不依赖 root-abi。
pub struct MappedNativeImage {
    _sealed: (),
}

impl MappedNativeImage {
    /// 仅供 platform-runtime 构造（同 `OpenedArtifactSet` 的可见性说明）。
    pub fn opaque() -> Self {
        MappedNativeImage { _sealed: () }
    }
}

impl std::fmt::Debug for MappedNativeImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 不打印任何内部状态：映像句柄进日志没有用，且是泄漏面。
        f.write_str("MappedNativeImage(<opaque>)")
    }
}

/// 平台加载后端。P0 只有 Linux DynamicLibrary 实现（LCE-P0-014）。
///
/// `open_package` 的固定步骤、失败即整体不可见等语义见规格 §9.3；本 trait 只定形状。
pub trait LoadBackend: Send + Sync {
    fn open_package(
        &self,
        request: OpenPackageRequest,
    ) -> Result<OpenedArtifactSet, PlatformRuntimeError>;

    fn map_native(
        &self,
        opened: &OpenedArtifactSet,
        native_artifact: &PackagePath,
    ) -> Result<Arc<MappedNativeImage>, PlatformRuntimeError>;
}
