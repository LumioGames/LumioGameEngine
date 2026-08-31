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

    let projection = source_tree_projection(&native, &voxel);

    Ok(SourceLock {
        repositories: [native, voxel],
        source_tree_digest: sha256_hex(projection.as_bytes()),
    })
}

/// `source_tree_digest` 的投影：component 与 tree_id 按声明序逐行拼接后取 SHA-256。
///
/// **单射性不是这个格式自己保证的，是载荷在 [`require_git_object_id`] 上的。** tree_id
/// 恒为 40 位 `[0-9a-f]`，既不含作行分隔的换行、也不含 component 标签，投影因此是定长
/// 4 行的定宽编码，`(native_tree, voxel_tree)` 能从字节串唯一还原。约束一旦放宽，同一段
/// 拼接立刻可伪造——测试模块里有那对显式碰撞输入。
///
/// 这条依赖跨函数（拼接在这里，约束在 `inspect_one` 里），所以它必须被测试钉住：
/// 否则谁放宽了约束，拼接会静默变回可伪造，而所有现有测试保持绿。
fn source_tree_projection(native: &SourceRepository, voxel: &SourceRepository) -> String {
    format!(
        "{}\n{}\n{}\n{}\n",
        native.component.as_str(),
        native.tree_id,
        voxel.component.as_str(),
        voxel.tree_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceComponent;
    use std::collections::BTreeMap;

    const NATIVE_LABEL: &str = "LumioNativeCore";
    const VOXEL_LABEL: &str = "LumioVoxelEngine";

    fn repo(component: SourceComponent, tree_id: &str) -> SourceRepository {
        SourceRepository {
            component,
            repository: "https://example.invalid/r".to_string(),
            checkout_root: "build/sources/r".to_string(),
            commit: "0".repeat(40),
            tree_id: tree_id.to_string(),
        }
    }

    fn projection_of(native_tree: &str, voxel_tree: &str) -> String {
        source_tree_projection(
            &repo(SourceComponent::LumioNativeCore, native_tree),
            &repo(SourceComponent::LumioVoxelEngine, voxel_tree),
        )
    }

    /// 从投影字节还原 `(native_tree, voxel_tree)`。投影是定长 4 行、行 1/3 为 component
    /// 标签；**能唯一还原即单射**。还原不出来说明投影格式变了。
    fn recover(projection: &str) -> Option<(String, String)> {
        let lines: Vec<&str> = projection.split('\n').collect();
        // 末尾那个 LF 会切出一个空尾项，故恰好 5 段。
        if lines.len() != 5 || !lines[4].is_empty() {
            return None;
        }
        if lines[0] != NATIVE_LABEL || lines[2] != VOXEL_LABEL {
            return None;
        }
        Some((lines[1].to_string(), lines[3].to_string()))
    }

    /// **约束这一半。** `source_tree_digest` 的拼接投影靠 tree_id 的字母表排除掉分隔符与
    /// 标签；这条测试钉住的就是「排除」本身，与投影格式无关。
    #[test]
    fn object_id_constraint_excludes_the_projection_delimiter_and_labels() {
        require_git_object_id(&"a".repeat(40), "tree_id")
            .expect("约束这一半塌了：40 位小写十六进制本应被接受");

        // 前两项刻意**恰好 40 字符**——它们必须被字母表挡住，而不是被长度挡住，
        // 否则「排除换行 / 排除标签」这两条性质其实没被验证到，只验证了长度。
        let alphabet_only = [
            ("0".repeat(39) + "\n", "含换行，即投影的行分隔符"),
            ("0".repeat(24) + VOXEL_LABEL, "含 component 标签"),
        ];
        for (value, why) in &alphabet_only {
            assert_eq!(
                value.chars().count(),
                40,
                "本用例要验证的是字母表而不是长度，{why} 的样本必须恰好 40 字符"
            );
        }

        let hostile = [
            ("0".repeat(41), "长度不是 40"),
            ("A".repeat(40), "大写十六进制"),
            ("g".repeat(40), "非十六进制字符"),
        ];
        for (value, why) in alphabet_only.iter().chain(hostile.iter()) {
            assert!(
                require_git_object_id(value, "tree_id").is_err(),
                "约束这一半塌了：require_git_object_id 不再拒绝「{why}」的 tree_id，\
                 source_tree_projection 的拼接随之可伪造"
            );
        }
    }

    /// **投影这一半。** 在约束成立的前提下，投影必须单射：任一合法组合都能从字节串唯一
    /// 还原，且不同组合不产生同一字节串。
    #[test]
    fn projection_is_injective_under_the_object_id_constraint() {
        let ids: Vec<String> = (0..64u32).map(|i| format!("{i:040x}")).collect();
        let mut seen: BTreeMap<String, (String, String)> = BTreeMap::new();

        for native in &ids {
            for voxel in &ids {
                require_git_object_id(native, "native tree_id")
                    .expect("约束这一半塌了：构造的 40 位小写十六进制本应合法");
                require_git_object_id(voxel, "voxel tree_id")
                    .expect("约束这一半塌了：构造的 40 位小写十六进制本应合法");

                let projection = projection_of(native, voxel);
                assert_eq!(
                    recover(&projection).as_ref(),
                    Some(&(native.clone(), voxel.clone())),
                    "投影这一半塌了：source_tree_projection 的格式变了，\
                     (native_tree, voxel_tree) 不再能从字节串唯一还原"
                );
                if let Some(previous) = seen.insert(projection, (native.clone(), voxel.clone())) {
                    panic!(
                        "投影这一半塌了：{previous:?} 与 ({native}, {voxel}) 产生同一投影字节串"
                    );
                }
            }
        }
    }

    /// **两半的接合点。** 值替换方向的显式碰撞：拼接式投影**自己**不保证单射，挡住它的
    /// 是字母表约束。这条测试同时钉住「碰撞确实存在」与「约束确实挡得住」——只钉后者的话，
    /// 将来有人删掉约束时，没有任何东西提醒他删掉的是什么。
    #[test]
    fn collision_ruled_out_by_the_object_id_constraint() {
        let a = ("X".to_string(), format!("Y\n{VOXEL_LABEL}\nZ"));
        let b = (format!("X\n{VOXEL_LABEL}\nY"), "Z".to_string());

        assert_ne!(a, b, "这对输入语义必须不同，否则测不出碰撞");
        assert_eq!(
            projection_of(&a.0, &a.1),
            projection_of(&b.0, &b.1),
            "投影这一半变了：这对输入本应拼出同一字节串（碰撞的存在性是本测试的前提）"
        );

        for tree_id in [&a.0, &a.1, &b.0, &b.1] {
            assert!(
                require_git_object_id(tree_id, "tree_id").is_err(),
                "约束这一半塌了：{tree_id:?} 本应被拒绝；它一旦可用，\
                 上面那对输入就是 source_tree_digest 的真实碰撞"
            );
        }
    }
}
