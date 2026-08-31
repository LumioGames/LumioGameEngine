//! Generated Root ABI binding. Do not hand-edit.
//! Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27. ADR-040.
//! Layout profile: linux-x86_64-glibc.

#![allow(non_camel_case_types)]

pub const ABI_VERSION: u32 = 1;
pub const ENTRY_SYMBOL: &str = "lumio_core_get_api_v1";
pub const SYMBOL_PREFIX: &str = "lumio_";
pub const CALLING_CONVENTION: &str = "C";
pub const CAPABILITY_BITS: u64 = 7;
pub const TARGET_PROFILE_ID: &str = "linux-x86_64-glibc";
pub const POINTER_BYTES: usize = 8;
pub const MAX_ALIGNMENT: usize = 8;
pub const ROOT_HEADER_BYTES: usize = 16;
pub const TABLE_HEADER_BYTES: usize = 16;

pub type LumioStatus = i32;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LumioHandle {
    pub index: u32,
    pub generation: u32,
    pub context: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LumioBuffer {
    pub ptr: *mut core::ffi::c_void,
    pub len: u64,
    pub capacity: u64,
}

#[repr(C)]
pub struct LumioCoreConfigV1 {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct LumioVoxelWorldDescV1 {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct LumioCoreApi {
    pub version: u32,
    pub struct_size: u32,
    pub reserved0: u64,
    pub lumio_core_init: Option<extern "C" fn(config: *const LumioCoreConfigV1, out_context: LumioHandle) -> LumioStatus>,
    pub lumio_core_shutdown: Option<extern "C" fn(context: LumioHandle) -> LumioStatus>,
    pub lumio_core_last_error_detail: Option<extern "C" fn(context: LumioHandle, out_detail: LumioBuffer) -> LumioStatus>,
    pub reserved: [*mut core::ffi::c_void; 1],
}

#[repr(C)]
pub struct LumioVoxelApi {
    pub version: u32,
    pub struct_size: u32,
    pub reserved0: u64,
    pub lumio_voxel_world_create: Option<extern "C" fn(context: LumioHandle, desc: *const LumioVoxelWorldDescV1, out_world: LumioHandle) -> LumioStatus>,
    pub lumio_voxel_world_destroy: Option<extern "C" fn(world: LumioHandle) -> LumioStatus>,
}

#[repr(C)]
pub struct LumioRootApi {
    pub abi_version: u32,
    pub struct_size: u32,
    pub capability_bits: u64,
    pub lumio_core_api: *const LumioCoreApi,
    pub lumio_voxel_api: *const LumioVoxelApi,
    pub reserved_tail: [u8; 32],
}

/// Layout Golden: `(struct, field, offset)` triples the consumer asserts.
pub const SLOT_OFFSETS: &[(&str, &str, usize)] = &[
    ("lumio_core_api", "lumio_core_init", 16),
    ("lumio_core_api", "lumio_core_shutdown", 24),
    ("lumio_core_api", "lumio_core_last_error_detail", 32),
    ("lumio_voxel_api", "lumio_voxel_world_create", 16),
    ("lumio_voxel_api", "lumio_voxel_world_destroy", 24),
];

pub const STRUCT_SIZES: &[(&str, usize)] = &[
    ("lumio_handle_t", 16),
    ("lumio_buffer_t", 24),
    ("lumio_core_api", 48),
    ("lumio_voxel_api", 32),
    ("lumio_root_api", 64),
];

const _: () = {
    assert!(core::mem::size_of::<LumioHandle>() == 16);
    assert!(core::mem::size_of::<LumioBuffer>() == 24);
    assert!(core::mem::size_of::<LumioCoreApi>() == 48);
    assert!(core::mem::size_of::<LumioVoxelApi>() == 32);
    assert!(core::mem::size_of::<LumioRootApi>() == 64);
    assert!(core::mem::offset_of!(LumioCoreApi, lumio_core_init) == 16);
    assert!(core::mem::offset_of!(LumioCoreApi, lumio_core_shutdown) == 24);
    assert!(core::mem::offset_of!(LumioCoreApi, lumio_core_last_error_detail) == 32);
    assert!(core::mem::offset_of!(LumioVoxelApi, lumio_voxel_world_create) == 16);
    assert!(core::mem::offset_of!(LumioVoxelApi, lumio_voxel_world_destroy) == 24);
    assert!(core::mem::offset_of!(LumioRootApi, lumio_core_api) == 16);
    assert!(core::mem::offset_of!(LumioRootApi, lumio_voxel_api) == 24);
};

/// ADR-040 section 7 (D-015): capability keys projected from the ID Registry.
/// `ids/index.json` is the authority for these numerics; this table is its only
/// published projection. A consumer reads it instead of inventing a private key.
/// These are enumeration keys, not bit positions: `capability_bits` semantics
/// stay unfrozen (ADR-040 section 7).
pub const CAPABILITY_KEYS: &[(&str, u32, &str)] = &[
    ("Native", 1, "Active"),
    ("HybridCLR", 2, "Reserved"),
    ("ReferenceVoxel", 3, "Active"),
    ("VoxelSnapshot", 4, "Active"),
    ("VoxelStreaming", 5, "Active"),
    ("VoxelSpatial", 6, "Active"),
    ("VoxelMeshCollision", 7, "Active"),
    ("VoxelAllResident", 8, "Active"),
    ("VoxelVolatileChunks", 9, "Active"),
];

pub fn capability_key(name: &str) -> Option<u32> {
    let mut i = 0;
    while i < CAPABILITY_KEYS.len() {
        if CAPABILITY_KEYS[i].0.as_bytes() == name.as_bytes() {
            return Some(CAPABILITY_KEYS[i].1);
        }
        i += 1;
    }
    None
}
