//! 只读随机访问、长度、平台文件身份接口（规格 §9.2、§9.3）。

use std::io;

/// 平台侧的「同一个文件对象」判据。
///
/// 存在的理由是规格 §9.7 的测试面要求「同 sealed fd 摘要与映射」：验证与映射必须
/// 消费同一个对象，而不是同一个**路径**下先后两次打开的两个对象。路径可以在两次
/// 打开之间被换掉（symlink rebind、原地替换），身份不会。
///
/// 字段是平台无关的抽象：Linux 用 (st_dev, st_ino)，Windows 用
/// (volume serial, file index)。本 crate 不依赖任何 OS crate，只定义形状，
/// 由 platform-runtime 填值（LCE-P0-014）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlatformFileIdentity {
    pub device_id: u64,
    pub file_id: u128,
}

impl PlatformFileIdentity {
    pub fn new(device_id: u64, file_id: u128) -> Self {
        PlatformFileIdentity { device_id, file_id }
    }
}

/// 已打开 Artifact 的只读视图。
///
/// `read_at` 而不是 `Read + Seek`：**不共享可变游标**（卡面实现要求）。共享游标的
/// 句柄一旦被两处并发使用，第二个读者会读到第一个读者移动后的位置——摘要与映射
/// 各读一半，且这种错误只在并发下偶发。带偏移的读没有这个状态。
///
/// `Send + Sync`：Loader 与 Verifier 在不同线程消费同一集合。
pub trait ArtifactBytes: Send + Sync {
    /// 字节长度。打开后即固定：底层是不可变 bytes 或 sealed 快照。
    fn len(&self) -> u64;

    /// 从 `offset` 读入 `dst`，返回实际读取字节数。
    ///
    /// 返回 0 表示 `offset` 已达或超过末尾，不是错误——调用方据此判定结束，
    /// 不需要先查长度再决定读不读。
    fn read_at(&self, offset: u64, dst: &mut [u8]) -> io::Result<usize>;

    /// 平台文件身份，用于证明两次消费的是同一个对象。
    fn platform_identity(&self) -> PlatformFileIdentity;

    /// 长度为 0。由 `len` 派生，只为满足 clippy 对 `len` 的惯例要求。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
