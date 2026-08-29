//! lumio-core-platform-contracts——平台运行时安全契约（PackagePath、OpenedArtifactSet、
//! LoadBackend，规格 §9.3）；控制文件语义（ManifestBody/ArtifactIndex/SignatureEnvelope）
//! 由架构源 Schema 拥有，本 crate 只定义仓内接口。
//!
//! 本 crate 在运行时发布闭包内（规格 §3.7、ADR 0004 第 2 条），因此：
//! - **不依赖任何平台 OS crate**（libc / rustix / libloading 都归 platform-runtime）；
//! - `test-support` feature 默认关闭，只供 runtime-verifier 的 dev-dependency 构造
//!   in-memory Fixture，不得出现在任何 normal 依赖路径上。

mod artifact_view;
mod backend;
mod error;
mod opened_set;
mod package_layout;
mod package_path;

#[cfg(feature = "test-support")]
mod test_support;

pub use artifact_view::{ArtifactBytes, PlatformFileIdentity};
pub use backend::{LoadBackend, MappedNativeImage, OpenPackageRequest};
pub use error::PlatformRuntimeError;
pub use opened_set::OpenedArtifactSet;
pub use package_layout::ControlFileKind;
pub use package_path::{PackagePath, PackagePathError};

#[cfg(feature = "test-support")]
pub use test_support::{OpenedArtifactSetFixtureBuilder, TestFixtureBuildError};
