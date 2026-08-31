use std::ffi::c_void;

mod abi_generated;

pub const SDK_ENTRY_SYMBOL: &str = abi_generated::ENTRY_SYMBOL;
pub const SDK_ABI_DEFINITION_SHA256: &str = abi_generated::DEFINITION_SHA256;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LumioStatus {
    Success = 0,
    InvalidArgument = 1,
    UnsupportedVersion = 2,
}

/// Compile-time composition marker for the SDK's two provider domains.
pub fn composed_provider_names() -> (&'static str, &'static str) {
    let _ = std::any::TypeId::of::<lumio_kernel::handle::HandleKey>();
    let _ = std::any::TypeId::of::<lumio_voxel_world::world::VoxelWorld>();
    ("LumioNativeCore", "LumioVoxelEngine")
}

#[repr(C)]
pub struct LumioEngineRootApiV1 {
    pub abi_version: u32,
    pub struct_size: u32,
    pub abi_hash: [u8; 32],
    pub build_id: [u8; 16],
    pub ping: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
}

const DEFAULT_ABI_HASH: &str = abi_generated::DEFINITION_SHA256;
const DEFAULT_BUILD_ID: &str = "2222222222222222222222222222222222222222222222222222222222222222";

const ABI_HASH: &str = match option_env!("LUMIO_ABI_HASH") {
    Some(value) => value,
    None => DEFAULT_ABI_HASH,
};
const BUILD_ID: &str = match option_env!("LUMIO_BUILD_ID") {
    Some(value) => value,
    None => DEFAULT_BUILD_ID,
};

const fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => 0,
    }
}

const fn decode_hex<const N: usize>(value: &str) -> [u8; N] {
    let bytes = value.as_bytes();
    let mut output = [0; N];
    let mut index = 0;
    while index < N {
        let offset = index * 2;
        output[index] = (hex_nibble(bytes[offset]) << 4) | hex_nibble(bytes[offset + 1]);
        index += 1;
    }
    output
}

unsafe extern "C" fn ping(marker: *mut c_void) -> i32 {
    if marker.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }

    // SAFETY: the caller owns the marker pointer and the API only writes one u32.
    unsafe { marker.cast::<u32>().write(1) };
    LumioStatus::Success as i32
}

static ROOT_API: LumioEngineRootApiV1 = LumioEngineRootApiV1 {
    abi_version: abi_generated::ABI_VERSION,
    struct_size: std::mem::size_of::<LumioEngineRootApiV1>() as u32,
    abi_hash: decode_hex(ABI_HASH),
    build_id: decode_hex(BUILD_ID),
    ping: Some(ping),
};

/// The only Native SDK symbol. All other functions are reached through this table.
#[no_mangle]
pub unsafe extern "C" fn lumio_engine_get_api_v1(
    requested_version: u32,
    out_api: *mut *const LumioEngineRootApiV1,
) -> i32 {
    if out_api.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    if requested_version != abi_generated::ABI_VERSION {
        // SAFETY: null is checked above and the caller supplied writable storage.
        unsafe { out_api.write(std::ptr::null()) };
        return LumioStatus::UnsupportedVersion as i32;
    }

    // SAFETY: null is checked above and ROOT_API has static storage duration.
    unsafe { out_api.write(&ROOT_API) };
    LumioStatus::Success as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_ping_is_rejected_without_writing_memory() {
        assert_eq!(unsafe { ping(std::ptr::null_mut()) }, LumioStatus::InvalidArgument as i32);
    }
}
