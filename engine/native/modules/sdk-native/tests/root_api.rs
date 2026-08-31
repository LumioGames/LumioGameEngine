use std::ffi::c_void;

use lumio_engine_native::{
    lumio_engine_get_api_v1, LumioEngineRootApiV1, LumioStatus,
};

#[test]
fn root_entry_returns_a_callable_table_with_build_identity() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(1, &mut table) };

    assert_eq!(status, LumioStatus::Success as i32);
    assert!(!table.is_null());

    let table = unsafe { &*table };
    assert_eq!(table.abi_version, 1);
    assert!(table.struct_size as usize >= std::mem::size_of::<LumioEngineRootApiV1>());
    assert_ne!(table.abi_hash, [0; 32]);
    assert_ne!(table.build_id, [0; 16]);
    assert!(table.ping.is_some());

    let ping = table.ping.unwrap();
    let mut marker = 0_u32;
    let marker_ptr = (&mut marker as *mut u32).cast::<c_void>();
    assert_eq!(unsafe { ping(marker_ptr) }, LumioStatus::Success as i32);
    assert_eq!(marker, 1);
}

#[test]
fn root_entry_rejects_an_unknown_requested_version() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(99, &mut table) };

    assert_ne!(status, LumioStatus::Success as i32);
    assert!(table.is_null());
}
