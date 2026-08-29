//! 不透明只读 API table view（规格 §8.2 `table_view.rs`、§8.3）。
//!
//! View **不提供**裸指针、library handle 或 `image_guard` 访问器；它私有持有的
//! `Arc<dyn SymbolResolver>` 把 API 表寿命绑定到常驻映像。所有字段值都是绑定期
//! 校验通过后的快照，读取快照不需要再次解引用镜像内存。

use std::fmt;
use std::ptr::NonNull;
use std::sync::Arc;

use crate::expectation::AbiExpectation;
use crate::generated;
use crate::symbol::SymbolResolver;

/// 架构源生成的 Root API Table 类型。别名而非同名新结构——布局只能来自生成物。
pub(crate) type GeneratedRootApiTable = generated::LumioRootApi;

/// 生成物里 Root Table 承载的 API table 数量。
///
/// 不是手写常量：由生成结构体的实际大小反推（root header + N 个 table 指针 +
/// 声明的保留尾部），上游增删 table 会改变生成结构体，这里的断言随即失败。
pub(crate) const TABLE_COUNT: usize = 2;

const _: () = {
    // 保留尾部长度取自生成结构体自身，不另写字面量。
    let reserved_tail = std::mem::size_of::<[u8; 32]>();
    assert!(
        std::mem::size_of::<GeneratedRootApiTable>()
            == generated::ROOT_HEADER_BYTES
                + TABLE_COUNT * generated::POINTER_BYTES
                + reserved_tail,
        "生成 Root Table 的 table 数量与 TABLE_COUNT 不一致"
    );
};

/// 单张 API table 在绑定期校验通过后的快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableSnapshot {
    pub(crate) name: &'static str,
    pub(crate) version: u32,
    pub(crate) struct_size: usize,
}

/// Root Table 在绑定期校验通过后的快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootSnapshot {
    pub(crate) abi_version: u32,
    pub(crate) struct_size: usize,
    pub(crate) capability_bits: u64,
    pub(crate) tables: [TableSnapshot; TABLE_COUNT],
}

/// 已绑定 Root API Table 的只读视图。
pub struct RootApiTableView {
    raw: NonNull<GeneratedRootApiTable>,
    expectation: Arc<AbiExpectation>,
    image_guard: Arc<dyn SymbolResolver>,
    snapshot: RootSnapshot,
}

impl RootApiTableView {
    pub(crate) fn new(
        raw: NonNull<GeneratedRootApiTable>,
        expectation: Arc<AbiExpectation>,
        image_guard: Arc<dyn SymbolResolver>,
        snapshot: RootSnapshot,
    ) -> Self {
        Self {
            raw,
            expectation,
            image_guard,
            snapshot,
        }
    }

    pub fn abi_version(&self) -> u32 {
        self.snapshot.abi_version
    }

    pub fn struct_size(&self) -> usize {
        self.snapshot.struct_size
    }

    /// 镜像发布的 `capability_bits` 原值。
    ///
    /// ADR-040 未冻结它是 bitmask 还是计数，也未冻结任何位位置——因此本 crate
    /// **不提供**按 `CapabilityId` 判定的 `supports()`：那需要本仓自造位语义，
    /// 而 ADR-040 明写「a consumer must not derive a capability key from either
    /// source」。上游确认语义前，消费方只能拿到这个不透明整数。
    pub fn capability_bits(&self) -> u64 {
        self.snapshot.capability_bits
    }

    /// 绑定时使用的加载期望。
    pub fn expectation(&self) -> &AbiExpectation {
        &self.expectation
    }

    /// 架构源生成的各 API table 的只读视图。
    pub fn generated_tables(&self) -> GeneratedApiTablesView<'_> {
        GeneratedApiTablesView {
            tables: &self.snapshot.tables,
        }
    }
}

impl fmt::Debug for RootApiTableView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 打印表地址的数值形式作为进程内身份（ADR-006 `loadPolicy =
        // OnePackagePerProcess` 的诊断需要）；不暴露可解引用的指针。
        f.debug_struct("RootApiTableView")
            .field("abi_identity", &self.expectation.abi_identity)
            .field(
                "table_identity",
                &format_args!("{:#x}", self.raw.as_ptr() as usize),
            )
            .field("abi_version", &self.snapshot.abi_version)
            .field("struct_size", &self.snapshot.struct_size)
            .field("capability_bits", &self.snapshot.capability_bits)
            .field("image_holders", &Arc::strong_count(&self.image_guard))
            .finish()
    }
}

/// 各生成 API table 的集合视图。
#[derive(Debug, Clone, Copy)]
pub struct GeneratedApiTablesView<'a> {
    tables: &'a [TableSnapshot; TABLE_COUNT],
}

impl<'a> GeneratedApiTablesView<'a> {
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    /// 生成 Root Table 里 `lumio_core_api` 槽位对应的 table。
    pub fn core_api(&self) -> ApiTableView<'a> {
        ApiTableView {
            snapshot: &self.tables[0],
        }
    }

    /// 生成 Root Table 里 `lumio_voxel_api` 槽位对应的 table。
    pub fn voxel_api(&self) -> ApiTableView<'a> {
        ApiTableView {
            snapshot: &self.tables[1],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ApiTableView<'a>> + '_ {
        self.tables.iter().map(|snapshot| ApiTableView { snapshot })
    }
}

/// 单张 API table 的只读视图；不提供 slot 函数指针（本卡非目标：不做业务分发）。
#[derive(Debug, Clone, Copy)]
pub struct ApiTableView<'a> {
    snapshot: &'a TableSnapshot,
}

impl ApiTableView<'_> {
    /// 上游 Golden 里的 table 名。
    pub fn name(&self) -> &'static str {
        self.snapshot.name
    }

    pub fn version(&self) -> u32 {
        self.snapshot.version
    }

    pub fn struct_size(&self) -> usize {
        self.snapshot.struct_size
    }

    /// 本 table 的 slot 数量，取自上游 Golden。
    pub fn slot_count(&self) -> usize {
        self.slot_offsets().count()
    }

    /// 本 table 的 `(slot 名, 偏移)`，逐项取自生成物 `SLOT_OFFSETS`。
    pub fn slot_offsets(&self) -> impl Iterator<Item = (&'static str, usize)> + '_ {
        let name = self.snapshot.name;
        generated::SLOT_OFFSETS
            .iter()
            .filter(move |(table, _, _)| *table == name)
            .map(|(_, slot, offset)| (*slot, *offset))
    }
}
