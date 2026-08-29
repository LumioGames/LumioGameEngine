//! 不透明、不可变、实际打开的 Artifact 集合（规格 §9.2、§9.3）。

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::artifact_view::ArtifactBytes;
use crate::error::PlatformRuntimeError;
use crate::package_layout::ControlFileKind;
use crate::package_path::PackagePath;

/// 一次 `open_package` 的全部产物。
///
/// **消费面只读**：字段私有，没有任何 `&mut self` 方法，也没有取出内部 `Arc` 再改的
/// 途径。Verifier / Loader 拿到它之后能做的只有读——这正是规格 §9.3
/// 「生产构造器不公开；Verifier/Loader 只能读取」里可被类型系统保证的那一半。
///
/// 生产构造器是 `pub(crate)`：后端（platform-runtime，另一个 crate）经
/// [`crate::LoadBackend::open_parts`] 只交出零件，由本 crate 在 trait 默认方法体里组装
/// ——默认方法体在本 crate 内编译，因此够得着私有构造器。构造反转的完整理由见
/// `backend.rs` 的模块文档。
///
/// **仍然保证不了的那一半，说清楚免得被误当成防线**：本类型保证「集合完整且此后不可变」，
/// **不是**「这些字节确实来自一次安全打开」——安全打开由后端负责（`openat2`
/// `RESOLVE_BENEATH|RESOLVE_NO_SYMLINKS`，规格 §6.3），本类型无从校验。
///
/// 与之相邻的另一条不变量（`test-support` 不进运行时闭包）由 justfile 的
/// `runtime-deps` recipe 在仓级断言，已接入 `just check`。
#[derive(Clone)]
pub struct OpenedArtifactSet {
    control: BTreeMap<ControlFileKind, Arc<dyn ArtifactBytes>>,
    artifacts: BTreeMap<PackagePath, Arc<dyn ArtifactBytes>>,
}

impl OpenedArtifactSet {
    /// 由已安全打开的条目组装集合。
    ///
    /// 三个控制文件必须齐备，缺一即 `ControlFileMissing`——`control()` 返回
    /// `&dyn ArtifactBytes` 而非 `Option`，这条不变量只能在构造时立住。
    ///
    /// 只经 `LoadBackend` 的默认方法体调用。
    pub(crate) fn from_opened_parts(
        control: BTreeMap<ControlFileKind, Arc<dyn ArtifactBytes>>,
        artifacts: BTreeMap<PackagePath, Arc<dyn ArtifactBytes>>,
    ) -> Result<Self, PlatformRuntimeError> {
        for kind in ControlFileKind::ALL {
            if !control.contains_key(&kind) {
                return Err(PlatformRuntimeError::ControlFileMissing {
                    kind,
                    detail: format!("组装 OpenedArtifactSet 时缺少 {}", kind.relative_path()),
                });
            }
        }
        // 控制文件路径不得同时出现在 artifacts 里：否则恶意 ArtifactIndex 可以驱动
        // §9.3 第 3 步把控制文件当普通 entry**再打开一次**，同一路径就被独立打开了两次，
        // 而规格 §6.3 明文「摘要和映射不允许在验证后重新按可变路径打开」。
        for kind in ControlFileKind::ALL {
            let control_path = kind.package_path();
            if artifacts.contains_key(&control_path) {
                return Err(PlatformRuntimeError::ControlFilePathReused {
                    kind,
                    path: control_path,
                });
            }
        }

        Ok(OpenedArtifactSet { control, artifacts })
    }

    /// 取控制文件。三种 kind 恒定存在（构造时已保证）。
    pub fn control(&self, kind: ControlFileKind) -> &dyn ArtifactBytes {
        self.control
            .get(&kind)
            .map(Arc::as_ref)
            .expect("控制文件齐备是构造期不变量")
    }

    /// 按路径取 Artifact；不存在返回 `None`。
    pub fn artifact(&self, path: &PackagePath) -> Option<&dyn ArtifactBytes> {
        self.artifacts.get(path).map(Arc::as_ref)
    }

    /// 全部 Artifact 路径。`BTreeMap` 使顺序稳定且与插入顺序无关——
    /// 下游对集合做的任何摘要因此可复现。
    pub fn artifact_paths(&self) -> impl ExactSizeIterator<Item = &PackagePath> {
        self.artifacts.keys()
    }
}

impl std::fmt::Debug for OpenedArtifactSet {
    /// 手写而非 derive：`dyn ArtifactBytes` 没有 `Debug`，而且这里也**不该**打印内容
    /// ——集合里有待验证的包字节，进日志既无用又是泄漏面。只报形状。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenedArtifactSet")
            .field("control_files", &self.control.len())
            .field("artifacts", &self.artifacts.len())
            .finish()
    }
}
