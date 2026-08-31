//! 临时目录 → 全量验证 → 只读权限 → 原子 rename（规格 §3.6 只读生成协议）。
//!
//! 与 composition 的冻结协议同一形状、同一理由：目标已存在即拒绝，不覆盖已发布生成物；
//! 失败不留下可发现的半成品。差别只在这里还要把发布出来的文件置为只读——
//! 「生成物不得手改」这条规则因此有了一层文件系统上的提醒（不是防线：有写权限的人
//! 仍可改回来，真正的判据是 `verify_generated` 的逐份摘要对账）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{err, AbiGenerationError, AbiGenerationErrorKind};

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = restore_writable(&self.0);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn atomic_failed(message: impl Into<String>) -> AbiGenerationError {
    err(AbiGenerationErrorKind::AtomicPublishFailed, message)
}

/// 递归恢复写权限——只读目录树删不掉，清理路径上必须先解除。
fn restore_writable(root: &Path) -> std::io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            restore_writable(&path)?;
        } else {
            let mut permissions = std::fs::metadata(&path)?.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            std::fs::set_permissions(&path, permissions)?;
        }
    }
    Ok(())
}

fn set_readonly(path: &Path) -> Result<(), AbiGenerationError> {
    let mut permissions = std::fs::metadata(path)
        .map_err(|e| atomic_failed(format!("取 {} 权限失败：{e}", path.display())))?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)
        .map_err(|e| atomic_failed(format!("置 {} 只读失败：{e}", path.display())))
}

/// 把已验证的字节原子发布到 `output_directory`。
///
/// 调用前所有内容必须已在内存中验证完毕（规格 §3.6：临时目录 → 全量验证 → 只读 →
/// 原子 rename）；进了这里就只剩 I/O。
pub(crate) fn publish(
    output_directory: &Path,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<(), AbiGenerationError> {
    if output_directory.exists() {
        return Err(err(
            AbiGenerationErrorKind::OutputAlreadyExists,
            format!(
                "生成目录 {} 已存在；已发布生成物不可覆盖，重建请发布到新目录",
                output_directory.display()
            ),
        ));
    }
    let parent = output_directory
        .parent()
        .ok_or_else(|| atomic_failed("生成目录没有父目录".to_string()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| atomic_failed(format!("创建 {} 失败：{e}", parent.display())))?;

    let nonce = {
        use std::hash::{BuildHasher, Hasher};
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_usize(std::process::id() as usize);
        format!("{:016x}", hasher.finish())
    };
    let name = output_directory
        .file_name()
        .ok_or_else(|| atomic_failed("生成目录路径以 .. 结尾".to_string()))?
        .to_string_lossy()
        .into_owned();
    let temp_root = parent.join(format!(".{name}.tmp-{nonce}"));
    std::fs::create_dir(&temp_root)
        .map_err(|e| atomic_failed(format!("创建临时目录 {} 失败：{e}", temp_root.display())))?;
    let temp = TempDir(temp_root);

    for (relative, bytes) in files {
        let path = temp.0.join(relative);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| atomic_failed(format!("创建 {} 失败：{e}", dir.display())))?;
        }
        std::fs::write(&path, bytes)
            .map_err(|e| atomic_failed(format!("写入 {} 失败：{e}", path.display())))?;
        set_readonly(&path)?;
    }

    // 目标已存在的判定交给 rename 本身（同 composition 的理由：先 exists 再 rename
    // 之间有竞态窗口）；上面的 exists 检查只是为了给出更准的错误信息。
    std::fs::rename(&temp.0, output_directory).map_err(|e| {
        if output_directory.exists() {
            err(
                AbiGenerationErrorKind::OutputAlreadyExists,
                format!("生成目录 {} 已存在（{e}）", output_directory.display()),
            )
        } else {
            atomic_failed(format!(
                "原子发布到 {} 失败：{e}",
                output_directory.display()
            ))
        }
    })?;
    Ok(())
}
