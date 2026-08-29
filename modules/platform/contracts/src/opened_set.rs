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
/// **不可保证的另一半，说清楚免得被误当成防线**：Rust 没有跨 crate 的 friend 可见性，
/// 而 `LoadBackend` 的实现（platform-runtime）是**另一个 crate**，它必须能构造本类型。
/// 因此 [`OpenedArtifactSet::from_opened_parts`] 只能是 `pub`。它保证的是
/// 「集合完整且此后不可变」，**不是**「这些字节确实来自一次安全打开」——后者由调用方
/// （即 LoadBackend 实现）负责，本类型无从校验。要机器保证的那一条是 `test-support`
/// 不进运行时闭包，那由 feature gate + 仓级 `cargo tree` 断言覆盖。
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
    /// 只应由 `LoadBackend` 的实现调用（见类型文档对可见性的说明）。
    pub fn from_opened_parts(
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
