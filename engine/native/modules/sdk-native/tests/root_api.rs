use std::ffi::c_void;

use lumio_engine_native::{
    lumio_engine_get_api_v1, LumioEngineRootApiV1, LumioStatus, NativeVoxelProvider,
    VoxelBlockReadCellResult, VoxelBlockReadResult, VoxelBoxRequest, VoxelPresence,
    VoxelSectionKey, VoxelSectionRevisionResult, VoxelWorldCoordinate, VoxelWriteReceipt,
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
    assert_eq!(LumioStatus::TimerStaleHandle as i32, 6);
    assert_eq!(LumioStatus::TimerScopeInvalid as i32, 7);
    assert_eq!(LumioStatus::TimerScopeGenerationMismatch as i32, 8);
    assert_eq!(LumioStatus::TimerInvalidDueTick as i32, 9);
    assert_eq!(LumioStatus::TimerInvalidInterval as i32, 10);
    assert_eq!(LumioStatus::TimerScheduleBudgetExceeded as i32, 11);
    assert_eq!(LumioStatus::TimerSlotClosed as i32, 12);
    assert_eq!(LumioStatus::TimerSlotUnbound as i32, 13);
    assert_eq!(LumioStatus::TimerSlotDispatchMismatch as i32, 14);
    assert_eq!(LumioStatus::TimerSlotQueueFull as i32, 15);
    assert_eq!(LumioStatus::TimerLateCompletion as i32, 16);
    assert_eq!(LumioStatus::TimerManagerShutdown as i32, 17);
}

#[test]
fn live_root_table_covers_c4_timer_slots() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(1, &mut table) };
    assert_eq!(status, LumioStatus::Success as i32);
    assert!(!table.is_null());

    let table = unsafe { &*table };
    let size = std::mem::size_of::<LumioEngineRootApiV1>();
    assert_eq!(
        table.struct_size as usize, size,
        "struct_size must be size_of the live root table"
    );
    assert!(
        size >= 200,
        "CLR-only root is 88 bytes; C-4 timer_* slots require size_of >= 200, got {size}"
    );
    assert_eq!(size, 304, "A-1 voxel slots extend the C-4 root table");
    assert_ne!(
        size, 88,
        "loaded struct_size 88 is the CLR-only layout that blocked R-00374/R-00376"
    );

    assert!(
        table.timer_create_manager.is_some(),
        "timer_create_manager must be a live function pointer"
    );
    assert!(
        table.timer_destroy_manager.is_some(),
        "timer_destroy_manager must be a live function pointer"
    );
    assert!(
        table.timer_register_dispatch.is_some(),
        "timer_register_dispatch must be a live function pointer"
    );
    assert!(
        table.timer_register_scope.is_some(),
        "timer_register_scope must be a live function pointer"
    );
    assert!(
        table.timer_teardown_scope.is_some(),
        "timer_teardown_scope must be a live function pointer"
    );
    assert!(
        table.timer_create_slot.is_some(),
        "timer_create_slot must be a live function pointer"
    );
    assert!(
        table.timer_bind_slot.is_some(),
        "timer_bind_slot must be wired"
    );
    assert!(
        table.timer_close_slot.is_some(),
        "timer_close_slot must be a live function pointer"
    );
    assert!(
        table.timer_schedule_one_shot.is_some(),
        "timer_schedule_one_shot must be a live function pointer"
    );
    assert!(
        table.timer_schedule_repeating.is_some(),
        "timer_schedule_repeating must be a live function pointer"
    );
    assert!(table.timer_cancel.is_some(), "timer_cancel must be wired");
    assert!(table.timer_advance.is_some(), "timer_advance must be wired");
    assert!(table.timer_pump.is_some(), "timer_pump must be wired");
    assert!(table.timer_drain.is_some(), "timer_drain must be wired");

    let create = table.timer_create_manager.unwrap();
    assert_eq!(
        unsafe { create(0, std::ptr::null_mut()) },
        LumioStatus::InvalidArgument as i32
    );

    let destroy = table.timer_destroy_manager.unwrap();
    assert_eq!(
        unsafe { destroy(std::ptr::null_mut()) },
        LumioStatus::InvalidArgument as i32
    );

    let mut manager = std::ptr::null_mut::<c_void>();
    assert_eq!(
        unsafe { create(0, &mut manager) },
        LumioStatus::Success as i32
    );
    assert!(!manager.is_null());
    assert_eq!(unsafe { destroy(manager) }, LumioStatus::Success as i32);
    assert_eq!(
        unsafe { destroy(manager) },
        LumioStatus::TimerManagerShutdown as i32,
        "shutdown-tombstone must return TimerManagerShutdown (status 17)"
    );
}

#[test]
fn live_root_table_wires_a1_voxel_slots_and_leaves_physics_unfilled() {
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    assert_eq!(table.struct_size as usize, 304);
    assert!(table.block_read_cell.is_some());
    assert!(table.block_read_box.is_some());
    assert!(table.block_read_column.is_some());
    assert!(table.block_write_prepare.is_some());
    assert!(table.block_write_commit.is_some());
    assert!(table.block_write_abort.is_some());
    assert!(table.section_revision_query.is_some());
    assert!(table.residency_pin_declare.is_some());
    assert!(table.residency_pin_release.is_some());
    assert!(table.residency_pin_status.is_some());
    assert!(table.raycast.is_none());
    assert!(table.sweep.is_none());
    assert!(table.overlap.is_none());
}

#[test]
fn voxel_root_round_trip_preserves_unsigned_block_id_and_revision() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 0x8000_0102);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let mut result = VoxelBlockReadCellResult::default();
    let coordinate = VoxelWorldCoordinate::new(0, 1, 0);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut result)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(result.presence, VoxelPresence::Ready);
    assert_eq!(result.has_block_id, 1);
    assert_eq!(result.block_id, 0x8000_0102);
    assert_eq!(result.section_revision, 12);
}

#[test]
fn voxel_pending_batch_read_remains_explicit_and_prepare_commit_is_atomic() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 0x8000_0102);
    provider.seed_pending_section(VoxelSectionKey::new(1, 0, 0), 99);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };

    let request = VoxelBoxRequest::new(
        VoxelWorldCoordinate::new(0, 1, 0),
        VoxelWorldCoordinate::new(16, 1, 0),
    );
    let mut cells = [VoxelBlockReadResult::default(); 17];
    let mut cell_count = 0;
    let mut segments = [Default::default(); 2];
    let mut segment_count = 0;
    let mut truncated = 0;
    assert_eq!(
        unsafe {
            (table.block_read_box.unwrap())(
                provider.as_opaque_ptr(),
                (&request as *const VoxelBoxRequest).cast(),
                cells.as_mut_ptr(),
                cells.len() as u32,
                &mut cell_count,
                segments.as_mut_ptr(),
                segments.len() as u32,
                &mut segment_count,
                &mut truncated,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(cell_count, 17);
    assert_eq!(cells[0].presence, VoxelPresence::Ready);
    assert_eq!(cells[16].presence, VoxelPresence::Pending);
    assert_eq!(cells[16].has_block_id, 0);
    assert_eq!(segment_count, 2);
    assert_eq!(segments[0].section_key.x, 0);
    assert_eq!(segments[0].section_key.y, 0);
    assert_eq!(segments[0].section_key.z, 0);
    assert_eq!(segments[0].presence, VoxelPresence::Ready);
    assert_eq!(segments[0].first_result, 0);
    assert_eq!(segments[0].result_count, 16);
    assert_eq!(segments[1].section_key.x, 1);
    assert_eq!(segments[1].section_key.y, 0);
    assert_eq!(segments[1].section_key.z, 0);
    assert_eq!(segments[1].presence, VoxelPresence::Pending);
    assert_eq!(segments[1].first_result, 16);
    assert_eq!(segments[1].result_count, 1);
    assert_eq!(truncated, 0);

    let entry = lumio_engine_native::VoxelBlockWriteEntry::new(
        VoxelSectionKey::new(0, 0, 0),
        256,
        0x8000_0103,
        12,
    );
    let mut token = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.block_write_prepare.unwrap())(provider.as_opaque_ptr(), 7, &entry, 1, &mut token)
        },
        LumioStatus::Success as i32
    );
    let mut receipts = [VoxelWriteReceipt::default(); 1];
    let mut receipt_count = 0;
    assert_eq!(
        unsafe {
            (table.block_write_commit.unwrap())(
                provider.as_opaque_ptr(),
                token,
                receipts.as_mut_ptr(),
                1,
                &mut receipt_count,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(receipt_count, 1);
    assert_eq!(receipts[0].up_to_section_revision, 13);

    let mut after = VoxelBlockReadCellResult::default();
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(
                provider.as_opaque_ptr(),
                &VoxelWorldCoordinate::new(0, 1, 0),
                &mut after,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(after.presence, VoxelPresence::Ready);
    assert_eq!(after.block_id, 0x8000_0103);
    assert_eq!(after.section_revision, 13);

    let mut untouched = VoxelBlockReadCellResult::default();
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(
                provider.as_opaque_ptr(),
                &VoxelWorldCoordinate::new(1, 1, 0),
                &mut untouched,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(untouched.presence, VoxelPresence::Ready);
    assert_eq!(untouched.block_id, 0x8000_0102);
    assert_eq!(untouched.section_revision, 13);

    let mut revision = VoxelSectionRevisionResult::default();
    assert_eq!(
        unsafe {
            (table.section_revision_query.unwrap())(
                provider.as_opaque_ptr(),
                &VoxelSectionKey::new(0, 0, 0),
                &mut revision,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(revision.presence, VoxelPresence::Ready);
    assert_eq!(revision.section_revision, 13);
}

#[test]
fn voxel_revision_and_null_arguments_return_stable_statuses_without_panicking() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let key = VoxelSectionKey::new(0, 0, 0);
    let mut revision = VoxelSectionRevisionResult::default();
    assert_eq!(
        unsafe {
            (table.section_revision_query.unwrap())(provider.as_opaque_ptr(), &key, &mut revision)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(revision.presence, VoxelPresence::Ready);
    assert_eq!(revision.section_revision, 12);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(
                provider.as_opaque_ptr(),
                std::ptr::null(),
                &mut Default::default(),
            )
        },
        LumioStatus::InvalidArgument as i32
    );
}

#[test]
fn native_provider_is_backed_by_a_running_voxel_world() {
    let provider = NativeVoxelProvider::new();
    assert_eq!(provider.world_state().lifecycle(), "Running");
    assert_eq!(provider.world_state().role(), "Authority");
}

#[test]
fn residency_slots_route_to_the_paired_pin_manager() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let key = VoxelSectionKey::new(0, 0, 0);
    let mut pin = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.residency_pin_declare.unwrap())(provider.as_opaque_ptr(), &key, 1, 1, &mut pin)
        },
        LumioStatus::Success as i32
    );
    let mut status = lumio_engine_native::VoxelPinStatus {
        ready: 0,
        _reserved: [0; 7],
        section_count: 0,
        ready_section_count: 0,
    };
    assert_eq!(
        unsafe {
            (table.residency_pin_status.unwrap())(provider.as_opaque_ptr(), pin, &mut status)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(status.ready, 1);
    assert_eq!(status.section_count, 1);
    assert_eq!(status.ready_section_count, 1);
    assert_eq!(
        unsafe { (table.residency_pin_release.unwrap())(provider.as_opaque_ptr(), pin) },
        LumioStatus::Success as i32
    );
}

#[test]
fn ready_pin_rejects_later_pending_and_unavailable_abi_reads() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let key = VoxelSectionKey::new(0, 0, 0);
    let mut pin = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.residency_pin_declare.unwrap())(provider.as_opaque_ptr(), &key, 1, 1, &mut pin)
        },
        LumioStatus::Success as i32
    );

    let coordinate = VoxelWorldCoordinate::new(0, 1, 0);
    let mut cell = VoxelBlockReadCellResult::default();
    provider.seed_pending_section(key, 13);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut cell)
        },
        1048
    );

    let request = VoxelBoxRequest::new(coordinate, coordinate);
    let mut cells = [VoxelBlockReadResult::default(); 1];
    let mut cell_count = 0;
    let mut segments = [Default::default(); 1];
    let mut segment_count = 0;
    let mut truncated = 0;
    assert_eq!(
        unsafe {
            (table.block_read_box.unwrap())(
                provider.as_opaque_ptr(),
                (&request as *const VoxelBoxRequest).cast(),
                cells.as_mut_ptr(),
                1,
                &mut cell_count,
                segments.as_mut_ptr(),
                1,
                &mut segment_count,
                &mut truncated,
            )
        },
        1048
    );

    provider.seed_unavailable_section(key, 14);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut cell)
        },
        1048
    );
    assert_eq!(
        unsafe { (table.residency_pin_release.unwrap())(provider.as_opaque_ptr(), pin) },
        LumioStatus::Success as i32
    );
}

#[test]
fn pending_pin_remains_explicit_before_ready() {
    let mut provider = NativeVoxelProvider::new();
    let key = VoxelSectionKey::new(0, 0, 0);
    provider.seed_pending_section(key, 12);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let mut pin = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.residency_pin_declare.unwrap())(provider.as_opaque_ptr(), &key, 1, 1, &mut pin)
        },
        LumioStatus::Success as i32
    );
    let mut pin_status = lumio_engine_native::VoxelPinStatus {
        ready: 1,
        _reserved: [0; 7],
        section_count: 0,
        ready_section_count: 1,
    };
    assert_eq!(
        unsafe {
            (table.residency_pin_status.unwrap())(provider.as_opaque_ptr(), pin, &mut pin_status)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(pin_status.ready, 0);

    let coordinate = VoxelWorldCoordinate::new(0, 1, 0);
    let mut result = VoxelBlockReadCellResult::default();
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut result)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(result.presence, VoxelPresence::Pending);
    assert_eq!(result.has_block_id, 0);
    assert_eq!(
        unsafe { (table.residency_pin_release.unwrap())(provider.as_opaque_ptr(), pin) },
        LumioStatus::Success as i32
    );
}

#[test]
fn block_write_abort_releases_the_paired_mutation_reservation() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let entry =
        lumio_engine_native::VoxelBlockWriteEntry::new(VoxelSectionKey::new(0, 0, 0), 256, 2, 12);
    let mut token = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.block_write_prepare.unwrap())(provider.as_opaque_ptr(), 8, &entry, 1, &mut token)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(
        unsafe { (table.block_write_abort.unwrap())(provider.as_opaque_ptr(), token) },
        LumioStatus::Success as i32
    );
    let mut replacement = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.block_write_prepare.unwrap())(
                provider.as_opaque_ptr(),
                8,
                &entry,
                1,
                &mut replacement,
            )
        },
        LumioStatus::Success as i32
    );
}
