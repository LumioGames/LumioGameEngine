//! `SourceInspector` Adapter：验证本地 checkout 的 repository / commit / tree
//! （规格 §7.2）。
//!
//! 只读校验，不实现 VCS：不 clone、不 fetch、不改工作区、不联网。checkout 由操作者
//! 预置到 workspace 内的约定目录（见 `config/p0/*.compose.toml` 的 `checkout_root`）。

use std::path::Path;
use std::process::Command;

use crate::encode::sha256_hex;
use crate::error::{err, invalid, CompositionError, CompositionErrorKind};
use crate::model::{SourceCheckoutRequest, SourceLock, SourceRepository};
use crate::validate::to_workspace_relative;

fn git(checkout_root: &Path, args: &[&str]) -> Result<String, CompositionError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout_root)
        .args(args)
        .output()
        .map_err(|e| {
            invalid(format!(
                "在 {} 执行 git {args:?} 失败：{e}（SourceInspector 需要 git 在场）",
                checkout_root.display()
            ))
        })?;
    if !output.status.success() {
        return Err(invalid(format!(
            "在 {} 执行 git {args:?} 返回非零：{}",
            checkout_root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| invalid(format!("git {args:?} 输出不是 UTF-8：{e}")))
        .map(|text| text.trim().to_string())
}

fn require_git_object_id(value: &str, what: &str) -> Result<(), CompositionError> {
    let ok = value.len() == 40
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
    if ok {
        Ok(())
    } else {
        Err(invalid(format!(
            "{what} 不是 40 位小写十六进制 git object id：{value}"
        )))
    }
}

fn inspect_one(
    request: &SourceCheckoutRequest,
    workspace_root: &Path,
) -> Result<SourceRepository, CompositionError> {
    require_git_object_id(&request.expected_commit, "expected_commit")?;
    require_git_object_id(&request.expected_tree_id, "expected_tree_id")?;

    if !request.checkout_root.is_dir() {
        return Err(invalid(format!(
            "{} 的 checkout 目录不存在：{}（compose 不 clone，checkout 须预先就位）",
            request.component.as_str(),
            request.checkout_root.display()
        )));
    }
    let checkout_root = to_workspace_relative(workspace_root, &request.checkout_root)?;

    let commit = git(&request.checkout_root, &["rev-parse", "HEAD"])?;
    if commit != request.expected_commit {
        return Err(err(
            CompositionErrorKind::SourceCommitMismatch,
            format!(
                "{} commit 漂移：锁定 {}，checkout {}",
                request.component.as_str(),
                request.expected_commit,
                commit
            ),
        ));
    }

    let tree_id = git(&request.checkout_root, &["rev-parse", "HEAD^{tree}"])?;
    if tree_id != request.expected_tree_id {
        return Err(err(
            CompositionErrorKind::SourceTreeDigestMismatch,
            format!(
                "{} tree 漂移：锁定 {}，checkout {}",
                request.component.as_str(),
                request.expected_tree_id,
                tree_id
            ),
        ));
    }

    // 只看已跟踪文件：未跟踪文件不进 tree、不参与编译（.DS_Store 之类），
    // 把它算作漂移会让开发机上永远 compose 不出计划。已跟踪文件被改则相反——
    // commit/tree 都没变，只有这一项能发现「实际会被编译的内容」已经不是被锁定的那份。
    let dirty = git(
        &request.checkout_root,
        &["status", "--porcelain", "--untracked-files=no"],
    )?;
    if !dirty.is_empty() {
        return Err(err(
            CompositionErrorKind::DirtySourceTree,
            format!(
                "{} checkout 有未提交改动，不可用于可复现构建：\n{dirty}",
                request.component.as_str()
            ),
        ));
    }

    Ok(SourceRepository {
        component: request.component,
        repository: request.repository.clone(),
        checkout_root,
        commit,
        tree_id,
    })
}

/// 解析两仓 Source Lock。`repositories` 固定 [LumioNativeCore, LumioVoxelEngine]
/// 声明序（规格 §7.3 的定长数组与 SourceComponent 声明序蕴含）。
pub(crate) fn resolve(
    sources: &[SourceCheckoutRequest; 2],
    workspace_root: &Path,
) -> Result<SourceLock, CompositionError> {
    use crate::model::SourceComponent::{LumioNativeCore, LumioVoxelEngine};
    if sources[0].component != LumioNativeCore || sources[1].component != LumioVoxelEngine {
        return Err(invalid(
            "sources 必须按 [LumioNativeCore, LumioVoxelEngine] 声明序给出".to_string(),
        ));
    }

    let native = inspect_one(&sources[0], workspace_root)?;
    let voxel = inspect_one(&sources[1], workspace_root)?;

    // source_tree_digest 的投影：component 与 tree_id 按声明序逐行拼接后取 SHA-256。
    // 用换行分隔而非直接拼接，避免不同 (component, tree) 组合产生同一字节串。
    let projection = format!(
        "{}\n{}\n{}\n{}\n",
        native.component.as_str(),
        native.tree_id,
        voxel.component.as_str(),
        voxel.tree_id
    );

    Ok(SourceLock {
        repositories: [native, voxel],
        source_tree_digest: sha256_hex(projection.as_bytes()),
    })
}
