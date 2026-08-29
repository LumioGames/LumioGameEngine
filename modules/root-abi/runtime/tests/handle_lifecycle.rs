//! `InvalidHandle` 与 `HandleDoubleRelease`（规格 §8.2 `tests/handle_lifecycle.rs`）。
//!
//! Handle 语义一律来自架构源 `metadata/native-managed-abi.json` 的 `handleModel`：
//! `encoding = IndexGenerationContext`、`invalidation = GenerationBump`、
//! `doubleDestroy = StableError`。本仓只包装，不定义新编码。

use lumio_core_contracts::ErrorCode;
use lumio_core_root_abi::generated::LumioHandle;
use lumio_core_root_abi::HandleGuard;

fn handle(index: u32, generation: u32) -> LumioHandle {
    LumioHandle {
        index,
        generation,
        context: 0x5155,
    }
}

#[test]
fn live_guard_yields_the_adopted_handle() {
    let guard = HandleGuard::adopt(handle(3, 9));

    assert!(guard.is_live());
    assert_eq!(
        guard.handle().expect("live guard 必须给出 handle"),
        handle(3, 9)
    );
}

#[test]
fn second_release_maps_to_handle_double_release_1030() {
    let mut guard = HandleGuard::adopt(handle(3, 9));

    guard.release().expect("首次释放必须成功");
    let code = guard.release().expect_err("重复释放必须失败").code();

    assert_eq!(code, ErrorCode::HandleDoubleRelease);
    assert_eq!(code as i32, 1030);
}

#[test]
fn use_after_release_maps_to_invalid_handle_1029() {
    let mut guard = HandleGuard::adopt(handle(3, 9));
    guard.release().expect("首次释放必须成功");

    assert!(!guard.is_live());
    let code = guard.handle().expect_err("释放后取用必须失败").code();

    assert_eq!(code, ErrorCode::InvalidHandle);
    assert_eq!(code as i32, 1029);
}

/// `invalidation = GenerationBump`：owner 侧 generation 前进后，
/// 旧 handle 必须判为 InvalidHandle。
#[test]
fn stale_generation_maps_to_invalid_handle_1029() {
    let guard = HandleGuard::adopt(handle(3, 9));

    guard.validate_generation(9).expect("同代必须有效");

    let code = guard
        .validate_generation(10)
        .expect_err("换代后必须失效")
        .code();
    assert_eq!(code, ErrorCode::InvalidHandle);
}

#[test]
fn released_guard_fails_generation_validation_too() {
    let mut guard = HandleGuard::adopt(handle(3, 9));
    guard.release().expect("首次释放必须成功");

    let code = guard
        .validate_generation(9)
        .expect_err("已释放必须失效")
        .code();
    assert_eq!(code, ErrorCode::InvalidHandle);
}
