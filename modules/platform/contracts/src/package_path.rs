//! 规范相对路径与 traversal 防护（规格 §6.3、§9.3）。
//!
//! 包内路径进入运行时的唯一闸门。不变量在**构造时**一次性校验完毕，之后
//! `PackagePath` 的存在本身即证明这些性质成立——没有「先构造再检查」的中间态，
//! 也没有会改变已构造值的 API。
//!
//! 本类型不碰文件系统。它挡的是词法层：由于 `..` 一律拒绝，「进入符号链接目录再
//! 用 `..` 跳出」这条逃逸路在这里就断了；真正跟随符号链接的防御是打开时的
//! `openat2(RESOLVE_BENEATH|RESOLVE_NO_SYMLINKS)`，归 platform-runtime。
//! 两层缺一不可——只做其中一层都不足以拒绝 symlink escape。

use std::fmt;

/// 构造失败的原因。逐项对应 §6.3 的不变量，便于调用方定位而不是只知道「非法」。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackagePathError {
    /// 空字符串。
    Empty,
    /// 以 `/` 开头（绝对路径），含 `//server/share` 形态的 UNC。
    NotRelative,
    /// `C:` 之类的盘符前缀。
    DriveLetter,
    /// 含反斜杠——在 Windows 上它是分隔符，只按 `/` 切分会把 `a\..\b` 看成单个分量。
    BackslashSeparator,
    /// 含 NUL：C 侧路径 API 以 NUL 结尾，截断后指向的是另一个文件。
    ContainsNul,
    /// 含 `..` 分量。
    ParentTraversal,
    /// 含 `.` 分量。
    CurrentDirectory,
    /// 连续分隔符造成的空分量。
    EmptyComponent,
    /// 以 `/` 结尾。
    TrailingSeparator,
}

impl fmt::Display for PackagePathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            PackagePathError::Empty => "路径为空",
            PackagePathError::NotRelative => "必须是相对路径，不得以 / 开头",
            PackagePathError::DriveLetter => "不得含盘符前缀",
            PackagePathError::BackslashSeparator => "不得含反斜杠，分隔符只用正斜杠",
            PackagePathError::ContainsNul => "不得含 NUL",
            PackagePathError::ParentTraversal => "不得含 .. 分量",
            PackagePathError::CurrentDirectory => "不得含 . 分量",
            PackagePathError::EmptyComponent => "不得含空分量（重复分隔符）",
            PackagePathError::TrailingSeparator => "不得以 / 结尾",
        };
        write!(f, "非法 PackagePath：{reason}")
    }
}

impl std::error::Error for PackagePathError {}

/// 包内相对路径。
///
/// 不变量（§6.3）：UTF-8、相对、正斜杠、非空、无盘符 / NUL / `.` / `..` / 重复分隔符；
/// 因不含 `..`，规范化后必然仍在 package root 内。
///
/// 有序（`Ord`）是刻意的：`OpenedArtifactSet` 用它作 `BTreeMap` 键，遍历顺序因此稳定，
/// 下游对 artifact 集合的任何摘要都不会因插入顺序而变。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackagePath(String);

impl PackagePath {
    /// 校验并构造。任何一条不变量不满足即拒绝，不做「顺手规范化」——
    /// 悄悄改写调用方给的路径会让日志里的路径与实际打开的文件对不上。
    pub fn parse(value: &str) -> Result<Self, PackagePathError> {
        if value.is_empty() {
            return Err(PackagePathError::Empty);
        }
        if value.contains('\0') {
            return Err(PackagePathError::ContainsNul);
        }
        if value.contains('\\') {
            return Err(PackagePathError::BackslashSeparator);
        }
        if value.starts_with('/') {
            return Err(PackagePathError::NotRelative);
        }
        // `C:` / `c:x`：ASCII 字母 + 冒号开头。冒号本身在其他位置是合法文件名字符
        // （Linux 上），只有盘符形态才拒绝。
        let bytes = value.as_bytes();
        if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            return Err(PackagePathError::DriveLetter);
        }
        if value.ends_with('/') {
            return Err(PackagePathError::TrailingSeparator);
        }

        for component in value.split('/') {
            match component {
                "" => return Err(PackagePathError::EmptyComponent),
                "." => return Err(PackagePathError::CurrentDirectory),
                ".." => return Err(PackagePathError::ParentTraversal),
                _ => {}
            }
        }

        Ok(PackagePath(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 按 `/` 切分的分量，顺序即路径顺序。每个分量都非空且不是 `.` / `..`。
    pub fn components(&self) -> impl ExactSizeIterator<Item = &str> {
        self.0.split('/').collect::<Vec<_>>().into_iter()
    }
}

impl fmt::Display for PackagePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for PackagePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
