//! 本仓平台包控制文件路径约定；**非公共 Schema**（规格 §9.2）。
//!
//! 目录形状取自规格 §3.8 的 P0 发布目录。它是本仓约定，不注册进架构源，
//! 也不随 ManifestBody 发布——下游按 `ControlFileKind` 取路径，不要各自拼字符串。

use crate::package_path::PackagePath;

/// package 的三个控制文件（规格 §9.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ControlFileKind {
    ManifestBody,
    ArtifactIndex,
    SignatureEnvelope,
}

impl ControlFileKind {
    /// 全部三种，顺序固定——构造与遍历都靠它，避免各处各写一份清单。
    pub const ALL: [ControlFileKind; 3] = [
        ControlFileKind::ManifestBody,
        ControlFileKind::ArtifactIndex,
        ControlFileKind::SignatureEnvelope,
    ];

    /// 相对 package root 的路径（规格 §3.8）。
    pub fn relative_path(self) -> &'static str {
        match self {
            ControlFileKind::ManifestBody => "metadata/core-engine-manifest.json",
            ControlFileKind::ArtifactIndex => "metadata/artifact-index.json",
            ControlFileKind::SignatureEnvelope => "metadata/signature-envelope.json",
        }
    }

    /// 同上，但已是校验过的 `PackagePath`。
    ///
    /// 常量路径本身合法，这里仍走一遍 `parse` 而不是绕过构造器：绕过一次就得解释
    /// 「什么情况下可以绕过」，而 `expect` 在这里只可能因为改坏了上面的常量而触发。
    pub fn package_path(self) -> PackagePath {
        PackagePath::parse(self.relative_path())
            .expect("控制文件路径常量必须满足 PackagePath 不变量")
    }
}
