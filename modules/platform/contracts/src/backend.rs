//! `LoadBackend` trait（规格 §9.2、§9.3）。
//!
//! # 为什么后端实现的是 `open_parts` 而不是 `open_package`
//!
//! 规格 §9.3 要求「`OpenedArtifactSet` 生产构造器不公开」。难点在于 Rust 没有跨 crate
//! 的 friend 可见性，而 `LoadBackend` 的实现（platform-runtime）是**另一个 crate**。
//!
//! 解法是**把构造反转**：后端只提供零件（`open_parts` / `map_native_payload`），
//! 由本 crate 在 trait 的**默认方法体**里组装。默认方法体在 contracts 内编译，因此
//! 能调用 `pub(crate)` 构造器——`OpenedArtifactSet` 与 `MappedNativeImage` 的构造器
//! 于是都无需公开，而 `open_package` / `map_native` 的签名与 §9.3 一字不差。
//!
//! 这两个默认方法在语法上可被覆盖，但覆盖者拿不到构造器，造不出返回值，所以覆盖不可行。
//!
//! 顺带一条**不要走的岔路**：不要用 feature gate 来「只对 platform-runtime 开放构造器」。
//! `test-support` 之所以能靠 feature 隔离，是因为它经 **dev**-dependency 启用，
//! resolver v2 在非测试构建里不统一 dev-dep 的 feature；而 platform-runtime 是运行时
//! 闭包内的 **normal** 依赖，它启用的任何 feature 都会在同一 build graph 里对 loader
//! 一并统一。feature gate 在这里挡不住任何人，只会给出虚假的安全感。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::artifact_view::ArtifactBytes;
use crate::error::PlatformRuntimeError;
use crate::opened_set::OpenedArtifactSet;
use crate::package_layout::ControlFileKind;
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

/// 后端持有的原生映像内部状态。对 Loader 完全不透明。
///
/// 本 crate 不依赖 root-abi，也不知道映像里有什么；`SymbolResolver` 的实现在
/// platform-runtime 侧（规格 §9.2 `symbol_resolver.rs`）。这里只要求它跨线程安全，
/// 因为 resident registry 会长期持有它。
pub trait NativeImagePayload: Send + Sync {}

/// 已映射的原生映像。
///
/// 不透明句柄：Loader 见不到 OS handle（规格 §9.1 非职责）。载荷由后端提供，
/// 构造器 `pub(crate)`，外部只能经 [`LoadBackend::map_native`] 取得。
pub struct MappedNativeImage {
    payload: Arc<dyn NativeImagePayload>,
}

impl MappedNativeImage {
    pub(crate) fn new(payload: Arc<dyn NativeImagePayload>) -> Self {
        MappedNativeImage { payload }
    }

    /// 后端取回自己的载荷以做符号解析。Loader 不该调用它——它拿到的
    /// `&dyn NativeImagePayload` 上没有任何方法。
    pub fn payload(&self) -> &Arc<dyn NativeImagePayload> {
        &self.payload
    }
}

impl std::fmt::Debug for MappedNativeImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 不打印任何内部状态：映像句柄进日志没有用，且是泄漏面。
        f.write_str("MappedNativeImage(<opaque>)")
    }
}

/// 后端打开 package 后交出的零件。
type OpenedParts = (
    BTreeMap<ControlFileKind, Arc<dyn ArtifactBytes>>,
    BTreeMap<PackagePath, Arc<dyn ArtifactBytes>>,
);

/// 平台加载后端。P0 只有 Linux DynamicLibrary 实现（LCE-P0-014）。
///
/// 实现方只需写 `open_parts` 与 `map_native_payload`；消费方（Loader / Verifier）
/// 只看 `open_package` 与 `map_native`——后两个的语义（固定步骤、任何一步失败集合
/// 整体不可见）见规格 §9.3。
pub trait LoadBackend: Send + Sync {
    /// 安全打开 package root 下的控制文件与全部 entry，交出不可变字节。
    ///
    /// 「安全」的含义由实现负责（P0 Linux：`openat2(RESOLVE_BENEATH|RESOLVE_NO_SYMLINKS)`，
    /// 规格 §6.3）——`PackagePath` 只挡住了词法层的逃逸，跟随符号链接这一半在这里。
    fn open_parts(&self, request: OpenPackageRequest) -> Result<OpenedParts, PlatformRuntimeError>;

    /// 把某个 native entry 映射为进程内映像，返回后端自有的不透明载荷。
    fn map_native_payload(
        &self,
        opened: &OpenedArtifactSet,
        native_artifact: &PackagePath,
    ) -> Result<Arc<dyn NativeImagePayload>, PlatformRuntimeError>;

    /// 规格 §9.3 的消费面签名。默认实现即全部实现，后端不需要也无法有效覆盖。
    fn open_package(
        &self,
        request: OpenPackageRequest,
    ) -> Result<OpenedArtifactSet, PlatformRuntimeError> {
        let (control, artifacts) = self.open_parts(request)?;
        OpenedArtifactSet::from_opened_parts(control, artifacts)
    }

    /// 规格 §9.3 的消费面签名。同上。
    fn map_native(
        &self,
        opened: &OpenedArtifactSet,
        native_artifact: &PackagePath,
    ) -> Result<Arc<MappedNativeImage>, PlatformRuntimeError> {
        let payload = self.map_native_payload(opened, native_artifact)?;
        Ok(Arc::new(MappedNativeImage::new(payload)))
    }
}
