use std::ffi::c_void;

use lumio_engine_native::{lumio_engine_get_api_v1, LumioEngineRootApiV1, LumioStatus};

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

#[test]
fn root_table_wires_the_clr_host_slots_to_real_implementations() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(1, &mut table) };
    assert_eq!(status, LumioStatus::Success as i32);

    let table = unsafe { &*table };
    assert!(
        table.create_clr_host.is_some(),
        "create_clr_host 槽位必须接线"
    );
    assert!(table.clr_host_call.is_some(), "clr_host_call 槽位必须接线");
    assert!(
        table.destroy_clr_host.is_some(),
        "destroy_clr_host 槽位必须接线"
    );

    // struct_size 随根表扩展自动更新（size_of），不得落后于实际布局。
    assert_eq!(
        table.struct_size as usize,
        std::mem::size_of::<LumioEngineRootApiV1>()
    );

    // 槽位必须转发到真实实现（clr-host 的参数校验路径），而不是空壳。
    let create = table.create_clr_host.unwrap();
    assert_eq!(
        unsafe {
            create(
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        },
        LumioStatus::InvalidArgument as i32
    );
    let call = table.clr_host_call.unwrap();
    let mut written = 0_u32;
    assert_eq!(
        unsafe {
            call(
                std::ptr::null_mut(),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
                &mut written,
            )
        },
        LumioStatus::InvalidArgument as i32
    );
    let destroy = table.destroy_clr_host.unwrap();
    assert_eq!(
        unsafe { destroy(std::ptr::null_mut()) },
        LumioStatus::InvalidArgument as i32
    );
}

#[test]
fn status_codes_match_the_abi_definition() {
    assert_eq!(LumioStatus::Success as i32, 0);
    assert_eq!(LumioStatus::InvalidArgument as i32, 1);
    assert_eq!(LumioStatus::UnsupportedVersion as i32, 2);
    assert_eq!(LumioStatus::ClrInitFailed as i32, 3);
    assert_eq!(LumioStatus::ClrEntryFailed as i32, 4);
    assert_eq!(LumioStatus::BufferTooSmall as i32, 5);
}
