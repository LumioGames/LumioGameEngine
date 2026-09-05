//! temp-write / fsync / no-replace rename / sidecar digest（ADR-0006 第 6、7 条）。
//!
//! 发布单元是**整个计划目录**（`build-plan.json` + `build-plan.sha256` +
//! `provenance.json`），三文件全有或全无。目标已存在即拒绝，任何情况下不覆盖已发布
//! 计划——ADR 0001 的不可变性在这里落成字节级判据。
//!
//! 「目标已存在」由 rename 的 no-replace 语义在**内核里**判定，不做
//! 「先 exists() 再 rename」的降级：那两步之间存在竞态窗口，恰好是不可变性最需要
//! 守住的地方。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{err, CompositionError, CompositionErrorKind};

/// 清理临时目录。失败不额外报错——此时已经在错误路径上，真正的原因更重要。
struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn atomic_failed(message: impl Into<String>) -> CompositionError {
    err(CompositionErrorKind::AtomicPublishFailed, message)
}

fn write_and_sync(path: &Path, bytes: &[u8]) -> Result<(), CompositionError> {
    let mut file = File::create(path)
        .map_err(|e| atomic_failed(format!("创建 {} 失败：{e}", path.display())))?;
    file.write_all(bytes)
        .map_err(|e| atomic_failed(format!("写入 {} 失败：{e}", path.display())))?;
    file.flush()
        .map_err(|e| atomic_failed(format!("flush {} 失败：{e}", path.display())))?;
    file.sync_all()
        .map_err(|e| atomic_failed(format!("fsync {} 失败：{e}", path.display())))?;
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), CompositionError> {
    // Windows 不支持对目录取句柄做 fsync；那里的目录项更新由 MoveFileEx 自身保证。
    #[cfg(not(windows))]
    {
        let dir = File::open(path)
            .map_err(|e| atomic_failed(format!("打开目录 {} 失败：{e}", path.display())))?;
        dir.sync_all()
            .map_err(|e| atomic_failed(format!("fsync 目录 {} 失败：{e}", path.display())))?;
    }
    #[cfg(windows)]
    let _ = path;
    Ok(())
}

/// no-replace 语义的目录 rename。
///
/// Linux 用 `renameat2(RENAME_NOREPLACE)`，macOS 用 `renameatx_np(RENAME_EXCL)`——
/// 两者都在一次系统调用里完成「不存在才改名」。Windows 的 `std::fs::rename` 走
/// `MoveFileExW`，而 `MOVEFILE_REPLACE_EXISTING` 对目录无效，目标已存在时本就失败，
/// 天然是 no-replace。
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn rename_no_replace(from: &Path, to: &Path) -> Result<(), CompositionError> {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_uint};
    use std::os::unix::ffi::OsStrExt;

    #[cfg(target_os = "linux")]
    const AT_FDCWD: c_int = -100;
    #[cfg(target_os = "macos")]
    const AT_FDCWD: c_int = -2;

    #[cfg(target_os = "linux")]
    const NO_REPLACE: c_uint = 1; // RENAME_NOREPLACE
    #[cfg(target_os = "macos")]
    const NO_REPLACE: c_uint = 0x0000_0004; // RENAME_EXCL

    extern "C" {
        #[cfg(target_os = "linux")]
        fn renameat2(
            olddirfd: c_int,
            oldpath: *const c_char,
            newdirfd: c_int,
            newpath: *const c_char,
            flags: c_uint,
        ) -> c_int;
        #[cfg(target_os = "macos")]
        fn renameatx_np(
            fromfd: c_int,
            from: *const c_char,
            tofd: c_int,
            to: *const c_char,
            flags: c_uint,
        ) -> c_int;
    }

    let from_c = CString::new(from.as_os_str().as_bytes())
        .map_err(|e| atomic_failed(format!("临时目录路径含 NUL：{e}")))?;
    let to_c = CString::new(to.as_os_str().as_bytes())
        .map_err(|e| atomic_failed(format!("计划目录路径含 NUL：{e}")))?;

    // SAFETY: 两个指针都指向刚构造、在本次调用期间存活的 NUL 结尾 C 字符串；
    // 其余参数是常量。调用本身不获取所有权、不保留指针。
    let result = unsafe {
        #[cfg(target_os = "linux")]
        {
            renameat2(
                AT_FDCWD,
                from_c.as_ptr(),
                AT_FDCWD,
                to_c.as_ptr(),
                NO_REPLACE,
            )
        }
        #[cfg(target_os = "macos")]
        {
            renameatx_np(
                AT_FDCWD,
                from_c.as_ptr(),
                AT_FDCWD,
                to_c.as_ptr(),
                NO_REPLACE,
            )
        }
    };

    if result == 0 {
        return Ok(());
    }
    let os_error = std::io::Error::last_os_error();
    match os_error.kind() {
        std::io::ErrorKind::AlreadyExists => Err(err(
            CompositionErrorKind::OutputAlreadyExists,
            format!(
                "计划目录 {} 已存在；已发布计划不可覆盖，可复现性比对请冻结到另一空目录",
                to.display()
            ),
        )),
        _ => Err(atomic_failed(format!(
            "原子发布到 {} 失败：{os_error}",
            to.display()
        ))),
    }
}

#[cfg(windows)]
fn rename_no_replace(from: &Path, to: &Path) -> Result<(), CompositionError> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }

    const ERROR_FILE_EXISTS: i32 = 80;
    const ERROR_ALREADY_EXISTS: i32 = 183;
    let from_wide: Vec<u16> = from
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let to_wide: Vec<u16> = to
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // A zero flag deliberately omits MOVEFILE_REPLACE_EXISTING. MoveFileExW
    // then performs the destination existence check as part of the rename,
    // keeping the no-replace guarantee atomic for both files and directories.
    let moved = unsafe { MoveFileExW(from_wide.as_ptr(), to_wide.as_ptr(), 0) } != 0;
    if moved {
        return Ok(());
    }
    let os_error = std::io::Error::last_os_error();
    if matches!(
        os_error.raw_os_error(),
        Some(ERROR_FILE_EXISTS | ERROR_ALREADY_EXISTS)
    ) {
        return Err(err(
            CompositionErrorKind::OutputAlreadyExists,
            format!(
                "计划目录 {} 已存在；已发布计划不可覆盖（{os_error}）",
                to.display()
            ),
        ));
    }
    Err(atomic_failed(format!(
        "原子发布到 {} 失败：{os_error}",
        to.display()
    )))
}

#[cfg(all(not(target_os = "linux"), not(target_os = "macos"), not(windows)))]
fn rename_no_replace(from: &Path, to: &Path) -> Result<(), CompositionError> {
    match std::fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(e) if to.exists() => Err(err(
            CompositionErrorKind::OutputAlreadyExists,
            format!(
                "计划目录 {} 已存在；已发布计划不可覆盖（{e}）",
                to.display()
            ),
        )),
        Err(e) => Err(atomic_failed(format!(
            "原子发布到 {} 失败：{e}",
            to.display()
        ))),
    }
}

pub(crate) struct FreezeOutput {
    pub(crate) plan_path: PathBuf,
    pub(crate) plan_digest_path: PathBuf,
    pub(crate) provenance_path: PathBuf,
}

/// 把已编码、已摘要的三份字节原子发布到 `output_plan_path`。
///
/// 编码与摘要必须在调用前全部完成（ADR-0006 第 7 条第 1 步）：进了这里就只剩 I/O，
/// 任何失败都不会留下算了一半的内容。
pub(crate) fn publish(
    output_plan_path: &Path,
    plan_bytes: &[u8],
    digest_bytes: &[u8],
    provenance_bytes: &[u8],
) -> Result<FreezeOutput, CompositionError> {
    let parent = output_plan_path.parent().ok_or_else(|| {
        atomic_failed(format!(
            "计划目录 {} 没有父目录",
            output_plan_path.display()
        ))
    })?;
    std::fs::create_dir_all(parent)
        .map_err(|e| atomic_failed(format!("创建 {} 失败：{e}", parent.display())))?;

    // 临时目录与最终目录同父（同文件系统），rename 才可能是原子的。
    //
    // 名字必须是一次性随机的（ADR-0006 第 7 条第 2 步）。pid + 计数不够：两个容器
    // bind-mount 同一个 build/ 时 pid 命名空间彼此独立，pid 会重复，两次 compose 可能
    // 撞进同一个临时目录，交错发布出「A 的 build-plan.json + B 的 provenance.json」。
    // 随机数只进临时目录名，不进计划字节，不损害确定性。
    let nonce = {
        use std::hash::{BuildHasher, Hasher};
        // RandomState 每次构造取新的随机种子，是 std 内唯一的随机源。
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_usize(std::process::id() as usize);
        format!("{:016x}", hasher.finish())
    };
    let file_name = output_plan_path
        .file_name()
        .ok_or_else(|| atomic_failed("计划目录路径以 .. 结尾".to_string()))?
        .to_string_lossy()
        .into_owned();
    let temp_root = parent.join(format!(".{file_name}.tmp-{nonce}"));
    // 不预先删除同名目录：`create_dir` 的 EEXIST 是这里唯一的抢占保护，先 remove 再
    // create 等于把它拆掉。撞名应当报错，不是静默复用别人的临时目录。
    std::fs::create_dir(&temp_root)
        .map_err(|e| atomic_failed(format!("创建临时目录 {} 失败：{e}", temp_root.display())))?;
    // 从这里开始，任何早退都由 Drop 清掉临时目录：失败不得留下可发现的半成品。
    let temp = TempDir(temp_root);

    let plan_tmp = temp.0.join("build-plan.json");
    let digest_tmp = temp.0.join("build-plan.sha256");
    let provenance_tmp = temp.0.join("provenance.json");
    write_and_sync(&plan_tmp, plan_bytes)?;
    write_and_sync(&digest_tmp, digest_bytes)?;
    write_and_sync(&provenance_tmp, provenance_bytes)?;
    sync_dir(&temp.0)?;

    rename_no_replace(&temp.0, output_plan_path)?;
    // rename 成功后临时目录已不在原路径，Drop 的清理会是无害的 no-op。
    sync_dir(parent)?;

    Ok(FreezeOutput {
        plan_path: output_plan_path.join("build-plan.json"),
        plan_digest_path: output_plan_path.join("build-plan.sha256"),
        provenance_path: output_plan_path.join("provenance.json"),
    })
}
