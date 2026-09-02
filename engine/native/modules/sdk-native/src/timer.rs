//! C-4 `timer_*` root slots. Implementations call NativeCore `lumio-timer`
//! (the only kernel). Handles stay as opaque pointers; no function-pointer
//! callbacks (ADR-006). Destroy leaves a shutdown tombstone.

use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use lumio_timer::{
    CallbackSlot, DispatchId, DispatchTarget, ScopeKind, SlotLifecycle, TimerError, TimerHandle,
    TimerManager, TimerMode, TimerScope,
};

use crate::LumioStatus;

#[repr(C)]
#[derive(Clone, Copy)]
struct TimerHandleAbi {
    index: u32,
    generation: u32,
    context: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TimerDrainRecord {
    handle_index: u32,
    handle_generation: u32,
    handle_context: u64,
    due: u64,
    schedule_sequence: u64,
    slot_dispatch_id: u32,
    pad: u32,
}

const _: () = assert!(std::mem::size_of::<TimerHandleAbi>() == 16);
const _: () = assert!(std::mem::size_of::<TimerDrainRecord>() == 40);
const _: () = assert!(std::mem::offset_of!(TimerDrainRecord, slot_dispatch_id) == 32);

struct FfiManager {
    kernel: TimerManager,
    mode: TimerMode,
}

static NEXT_CONTEXT: AtomicU64 = AtomicU64::new(1);
static ISSUED: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();

fn issued() -> std::sync::MutexGuard<'static, HashSet<usize>> {
    ISSUED
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn map_timer_error(error: TimerError) -> i32 {
    match error {
        TimerError::StaleHandle => LumioStatus::TimerStaleHandle as i32,
        TimerError::ScopeInvalid => LumioStatus::TimerScopeInvalid as i32,
        TimerError::ScopeGenerationMismatch => LumioStatus::TimerScopeGenerationMismatch as i32,
        TimerError::InvalidDueTick => LumioStatus::TimerInvalidDueTick as i32,
        TimerError::InvalidInterval => LumioStatus::TimerInvalidInterval as i32,
        TimerError::ScheduleBudgetExceeded => LumioStatus::TimerScheduleBudgetExceeded as i32,
        TimerError::SlotClosed => LumioStatus::TimerSlotClosed as i32,
        TimerError::SlotUnbound => LumioStatus::TimerSlotUnbound as i32,
        TimerError::SlotDispatchMismatch => LumioStatus::TimerSlotDispatchMismatch as i32,
        TimerError::SlotQueueFull => LumioStatus::TimerSlotQueueFull as i32,
        TimerError::LateCompletion => LumioStatus::TimerLateCompletion as i32,
        TimerError::ManagerShutdown => LumioStatus::TimerManagerShutdown as i32,
    }
}

fn pack_slot(slot: CallbackSlot) -> *mut c_void {
    let packed = (u64::from(slot.index()) << 32) | u64::from(slot.generation());
    packed as *mut c_void
}

fn unpack_slot(ptr: *mut c_void) -> Option<CallbackSlot> {
    if ptr.is_null() {
        return None;
    }
    let packed = ptr as u64;
    Some(CallbackSlot::from_abi((packed >> 32) as u32, packed as u32))
}

fn manager_mut(ptr: *mut c_void) -> Result<&'static mut FfiManager, i32> {
    if ptr.is_null() {
        return Err(LumioStatus::InvalidArgument as i32);
    }
    if !issued().contains(&(ptr as usize)) {
        return Err(LumioStatus::InvalidArgument as i32);
    }
    // SAFETY: pointer was issued by timer_create_manager and is never freed
    // (shutdown tombstone lives until process exit).
    Ok(unsafe { &mut *ptr.cast::<FfiManager>() })
}

fn running_manager(ptr: *mut c_void) -> Result<&'static mut FfiManager, i32> {
    let manager = manager_mut(ptr)?;
    if !manager.kernel.is_running() {
        return Err(LumioStatus::TimerManagerShutdown as i32);
    }
    Ok(manager)
}

fn scope_kind(raw: u32) -> Result<ScopeKind, i32> {
    match raw {
        0 => Ok(ScopeKind::World),
        1 => Ok(ScopeKind::Session),
        2 => Ok(ScopeKind::Adapter),
        _ => Err(LumioStatus::TimerScopeInvalid as i32),
    }
}

fn write_handle(out: *mut c_void, handle: TimerHandle) {
    // SAFETY: caller supplied a 16-byte TimerHandle out-param.
    unsafe {
        out.cast::<TimerHandleAbi>().write(TimerHandleAbi {
            index: handle.index(),
            generation: handle.generation(),
            context: handle.context(),
        });
    }
}

fn read_handle(ptr: *mut c_void) -> Option<TimerHandle> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: caller supplied a 16-byte TimerHandle.
    let abi = unsafe { ptr.cast::<TimerHandleAbi>().read() };
    Some(TimerHandle::from_abi(
        abi.index,
        abi.generation,
        abi.context,
    ))
}

fn schedule_scope(scope_id: u64, kind: u32, generation: u32) -> Result<TimerScope, i32> {
    Ok(TimerScope::new(scope_id, scope_kind(kind)?, generation))
}

/// # Safety
///
/// `out_manager` must be writable or null (null is rejected).
pub unsafe extern "C" fn timer_create_manager(mode: u32, out_manager: *mut *mut c_void) -> i32 {
    if out_manager.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    let Some(mode) = TimerMode::from_abi(mode) else {
        // SAFETY: null checked above.
        unsafe { out_manager.write(std::ptr::null_mut()) };
        return LumioStatus::InvalidArgument as i32;
    };
    let context = NEXT_CONTEXT.fetch_add(1, Ordering::Relaxed);
    let boxed = Box::new(FfiManager {
        kernel: TimerManager::with_mode(context, mode),
        mode,
    });
    let ptr = Box::into_raw(boxed);
    issued().insert(ptr as usize);
    // SAFETY: null checked above; the box is leaked as a process-lifetime tombstone.
    unsafe { out_manager.write(ptr.cast()) };
    LumioStatus::Success as i32
}

/// # Safety
///
/// `manager` is null or a handle from `timer_create_manager`.
pub unsafe extern "C" fn timer_destroy_manager(manager: *mut c_void) -> i32 {
    let mgr = match manager_mut(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    if !mgr.kernel.is_running() {
        return LumioStatus::TimerManagerShutdown as i32;
    }
    mgr.kernel.shutdown();
    LumioStatus::Success as i32
}

/// # Safety
///
/// `manager` is null or a handle from `timer_create_manager`.
pub unsafe extern "C" fn timer_register_dispatch(manager: *mut c_void, dispatch_id: u32) -> i32 {
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    if dispatch_id == 0 {
        return LumioStatus::InvalidArgument as i32;
    }
    let id = DispatchId::from_raw(dispatch_id);
    if mgr.kernel.is_dispatch_registered(id) {
        return LumioStatus::InvalidArgument as i32;
    }
    mgr.kernel.register_dispatch(id, DispatchTarget::Registered);
    LumioStatus::Success as i32
}

/// # Safety
///
/// `out_generation` must be writable or null (null is rejected).
pub unsafe extern "C" fn timer_register_scope(
    manager: *mut c_void,
    scope_id: u64,
    scope_kind_raw: u32,
    out_generation: *mut u32,
) -> i32 {
    if out_generation.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    if mgr.kernel.is_scope_alive(scope_id) {
        return LumioStatus::InvalidArgument as i32;
    }
    let kind = match scope_kind(scope_kind_raw) {
        Ok(k) => k,
        Err(code) => return code,
    };
    match mgr.kernel.register_scope(scope_id, kind) {
        Ok(scope) => {
            // SAFETY: null checked above.
            unsafe { out_generation.write(scope.generation()) };
            LumioStatus::Success as i32
        }
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `manager` is null or a handle from `timer_create_manager`.
pub unsafe extern "C" fn timer_teardown_scope(manager: *mut c_void, scope_id: u64) -> i32 {
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    match mgr.kernel.teardown_scope(scope_id) {
        Ok(_) => LumioStatus::Success as i32,
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `out_slot` must be writable or null (null is rejected).
pub unsafe extern "C" fn timer_create_slot(
    manager: *mut c_void,
    out_slot: *mut *mut c_void,
) -> i32 {
    if out_slot.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    match mgr.kernel.create_slot() {
        Ok(slot) => {
            // SAFETY: null checked above.
            unsafe { out_slot.write(pack_slot(slot)) };
            LumioStatus::Success as i32
        }
        Err(error) => {
            // SAFETY: null checked above.
            unsafe { out_slot.write(std::ptr::null_mut()) };
            map_timer_error(error)
        }
    }
}

/// # Safety
///
/// `slot` is an opaque packed CallbackSlot from `timer_create_slot`.
pub unsafe extern "C" fn timer_bind_slot(
    manager: *mut c_void,
    slot: *mut c_void,
    dispatch_id: u32,
) -> i32 {
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let Some(slot) = unpack_slot(slot) else {
        return LumioStatus::InvalidArgument as i32;
    };
    if dispatch_id == 0 {
        return LumioStatus::InvalidArgument as i32;
    }
    let id = DispatchId::from_raw(dispatch_id);
    if !mgr.kernel.is_dispatch_registered(id) {
        return LumioStatus::InvalidArgument as i32;
    }
    match mgr.kernel.slot_lifecycle(slot) {
        Ok(SlotLifecycle::Armed) => LumioStatus::InvalidArgument as i32,
        Ok(SlotLifecycle::Closed) => LumioStatus::TimerSlotClosed as i32,
        Ok(SlotLifecycle::Unbound) => match mgr.kernel.bind_slot(slot, id) {
            Ok(()) => LumioStatus::Success as i32,
            Err(error) => map_timer_error(error),
        },
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `slot` is an opaque packed CallbackSlot from `timer_create_slot`.
pub unsafe extern "C" fn timer_close_slot(manager: *mut c_void, slot: *mut c_void) -> i32 {
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let Some(slot) = unpack_slot(slot) else {
        return LumioStatus::InvalidArgument as i32;
    };
    match mgr.kernel.close_slot(slot) {
        Ok(()) => LumioStatus::Success as i32,
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `out_handle` must point at 16 writable bytes or be null (null is rejected).
pub unsafe extern "C" fn timer_schedule_one_shot(
    manager: *mut c_void,
    scope_id: u64,
    scope_kind_raw: u32,
    scope_generation: u32,
    due: u64,
    slot: *mut c_void,
    out_handle: *mut c_void,
) -> i32 {
    if out_handle.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let Some(slot) = unpack_slot(slot) else {
        return LumioStatus::InvalidArgument as i32;
    };
    let scope = match schedule_scope(scope_id, scope_kind_raw, scope_generation) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match mgr.kernel.schedule_one_shot(scope, due, slot) {
        Ok(handle) => {
            write_handle(out_handle, handle);
            LumioStatus::Success as i32
        }
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `out_handle` must point at 16 writable bytes or be null (null is rejected).
pub unsafe extern "C" fn timer_schedule_repeating(
    manager: *mut c_void,
    scope_id: u64,
    scope_kind_raw: u32,
    scope_generation: u32,
    first_due: u64,
    interval: u64,
    slot: *mut c_void,
    out_handle: *mut c_void,
) -> i32 {
    if out_handle.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let Some(slot) = unpack_slot(slot) else {
        return LumioStatus::InvalidArgument as i32;
    };
    let scope = match schedule_scope(scope_id, scope_kind_raw, scope_generation) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match mgr
        .kernel
        .schedule_repeating(scope, first_due, interval, slot)
    {
        Ok(handle) => {
            write_handle(out_handle, handle);
            LumioStatus::Success as i32
        }
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `handle` must point at a 16-byte TimerHandle or be null (null is rejected).
pub unsafe extern "C" fn timer_cancel(manager: *mut c_void, handle: *mut c_void) -> i32 {
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let Some(handle) = read_handle(handle) else {
        return LumioStatus::InvalidArgument as i32;
    };
    match mgr.kernel.cancel(handle) {
        Ok(_) => LumioStatus::Success as i32,
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `manager` is null or a handle from `timer_create_manager`.
pub unsafe extern "C" fn timer_advance(manager: *mut c_void, to_tick: u64) -> i32 {
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    if mgr.mode != TimerMode::TickFrame {
        return LumioStatus::InvalidArgument as i32;
    }
    match mgr.kernel.advance(to_tick) {
        Ok(_) => LumioStatus::Success as i32,
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `manager` is null or a handle from `timer_create_manager`.
pub unsafe extern "C" fn timer_pump(manager: *mut c_void, now_ms: u64) -> i32 {
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    if mgr.mode != TimerMode::WallClock {
        return LumioStatus::InvalidArgument as i32;
    }
    match mgr.kernel.pump(now_ms) {
        Ok(_) => LumioStatus::Success as i32,
        Err(error) => map_timer_error(error),
    }
}

/// # Safety
///
/// `out_records` is a `capacity`-long 40-byte record array when capacity > 0.
/// `out_count` must be writable or null (null is rejected).
pub unsafe extern "C" fn timer_drain(
    manager: *mut c_void,
    out_records: *mut c_void,
    capacity: u32,
    out_count: *mut u32,
) -> i32 {
    if out_count.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    if capacity > 0 && out_records.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    let mgr = match running_manager(manager) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let needed = mgr.kernel.pending_record_count();
    if needed > capacity {
        // SAFETY: null checked above.
        unsafe { out_count.write(needed) };
        return LumioStatus::BufferTooSmall as i32;
    }
    match mgr.kernel.drain_records() {
        Ok(records) => {
            // SAFETY: null checked above.
            unsafe { out_count.write(records.len() as u32) };
            let dest = out_records.cast::<TimerDrainRecord>();
            for (index, record) in records.iter().enumerate() {
                // SAFETY: capacity was checked against pending_record_count.
                unsafe {
                    dest.add(index).write(TimerDrainRecord {
                        handle_index: record.handle.index(),
                        handle_generation: record.handle.generation(),
                        handle_context: record.handle.context(),
                        due: record.due_tick,
                        schedule_sequence: record.schedule_sequence,
                        slot_dispatch_id: record.slot_dispatch_id.raw(),
                        pad: 0,
                    });
                }
            }
            LumioStatus::Success as i32
        }
        Err(error) => map_timer_error(error),
    }
}
