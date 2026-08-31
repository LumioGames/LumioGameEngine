//! 仅 `test-support` feature 编译的 in-memory `OpenedArtifactSetFixtureBuilder`
//! （规格 §9.2、§9.3）。
//!
//! 存在的唯一理由：runtime-verifier 要用架构源 Fixture 的 bytes 构造被验对象，而
//! 真实的 `open_package` 需要一个落盘的 package。**normal / runtime 依赖不可见**——
//! 这条不是靠自觉，是靠 `default = []` 加仓级 `cargo tree` 断言。
//!
//! 这里的 `ArtifactBytes` 实现是纯内存切片：没有文件、没有 fd、没有 OS 调用。
//! 因此它能证伪「read_at 不共享游标」之类的接口性质，但**证不了**任何与真实文件
//! 打开有关的性质（symlink 替换、原地写入、sealed fd）——那些只能由
//! platform-runtime 的真实后端测试覆盖（LCE-P0-014）。

use std::collections::BTreeMap;
use std::io;
use std::sync::Arc;

use crate::artifact_view::{ArtifactBytes, PlatformFileIdentity};
use crate::opened_set::OpenedArtifactSet;
use crate::package_layout::ControlFileKind;
use crate::package_path::PackagePath;

/// Fixture 组装失败。与 `PlatformRuntimeError` 分开：那是运行时错误面，
/// 这些只可能出现在测试代码写错时。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestFixtureBuildError {
    MissingControlFile(ControlFileKind),
    DuplicateArtifact(PackagePath),
}

impl std::fmt::Display for TestFixtureBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestFixtureBuildError::MissingControlFile(kind) => {
                write!(f, "Fixture 缺少控制文件 {}", kind.relative_path())
            }
            TestFixtureBuildError::DuplicateArtifact(path) => {
                write!(f, "Fixture 重复登记 Artifact {path}")
            }
        }
    }
}

impl std::error::Error for TestFixtureBuildError {}

/// 内存字节视图。
struct InMemoryArtifact {
    bytes: Arc<[u8]>,
    identity: PlatformFileIdentity,
}

impl ArtifactBytes for InMemoryArtifact {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_at(&self, offset: u64, dst: &mut [u8]) -> io::Result<usize> {
        // 越界读返回 0 而不是报错，与真实后端口径一致（见 ArtifactBytes 文档）。
        let Ok(start) = usize::try_from(offset) else {
            return Ok(0);
        };
        if start >= self.bytes.len() {
            return Ok(0);
        }
        let available = &self.bytes[start..];
        let count = available.len().min(dst.len());
        dst[..count].copy_from_slice(&available[..count]);
        Ok(count)
    }

    fn platform_identity(&self) -> PlatformFileIdentity {
        self.identity
    }
}

/// in-memory `OpenedArtifactSet` 构造器（仅测试）。
#[derive(Default)]
pub struct OpenedArtifactSetFixtureBuilder {
    control: BTreeMap<ControlFileKind, Arc<[u8]>>,
    artifacts: Vec<(PackagePath, Arc<[u8]>)>,
}

impl OpenedArtifactSetFixtureBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn control(mut self, kind: ControlFileKind, bytes: Arc<[u8]>) -> Self {
        self.control.insert(kind, bytes);
        self
    }

    /// 重复路径不在这里报错，留到 `build`：builder 链式调用中途 panic 会让测试
    /// 失败在无关的行上。
    pub fn artifact(mut self, path: PackagePath, bytes: Arc<[u8]>) -> Self {
        self.artifacts.push((path, bytes));
        self
    }

    pub fn build(self) -> Result<OpenedArtifactSet, TestFixtureBuildError> {
        for kind in ControlFileKind::ALL {
            if !self.control.contains_key(&kind) {
                return Err(TestFixtureBuildError::MissingControlFile(kind));
            }
        }

        // 身份靠序号区分：真实后端用 (dev, ino)，Fixture 只需保证「不同条目不同身份、
        // 同一条目身份稳定」，否则用它写的 same-object 测试会假通过。
        let mut next_identity = 0u128;
        let mut identity = || {
            next_identity += 1;
            PlatformFileIdentity::new(0, next_identity)
        };

        let mut control = BTreeMap::new();
        for kind in ControlFileKind::ALL {
            let bytes = self.control.get(&kind).expect("上面已校验齐备").clone();
            let artifact: Arc<dyn ArtifactBytes> = Arc::new(InMemoryArtifact {
                bytes,
                identity: identity(),
            });
            control.insert(kind, artifact);
        }

        let mut artifacts: BTreeMap<PackagePath, Arc<dyn ArtifactBytes>> = BTreeMap::new();
        for (path, bytes) in self.artifacts {
            if artifacts.contains_key(&path) {
                return Err(TestFixtureBuildError::DuplicateArtifact(path));
            }
            let artifact: Arc<dyn ArtifactBytes> = Arc::new(InMemoryArtifact {
                bytes,
                identity: identity(),
            });
            artifacts.insert(path, artifact);
        }

        Ok(OpenedArtifactSet::from_opened_parts(control, artifacts)
            .expect("控制文件齐备，组装不会失败"))
    }
}
