//! 架构源 Handle model 的包装（规格 §8.2 `handle_guard.rs`）。
//!
//! 编码与失效语义全部来自架构源 `metadata/native-managed-abi.json` 的 `handleModel`：
//! `encoding = IndexGenerationContext`、`invalidation = GenerationBump`、
//! `doubleDestroy = StableError`。本仓**不定义新编码**，也不持有 slot 注册表——
//! 当前代（`current_generation`）由 handle 的所有者给出，本类型只做判定与状态机。

use crate::error::RootAbiError;
use crate::generated::LumioHandle;

/// 单个 Handle 的生命周期守卫：区分「活」「已释放」，并按 GenerationBump 判失效。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleGuard {
    handle: LumioHandle,
    live: bool,
}

impl HandleGuard {
    /// 接管一个由 native 侧发出的 Handle。
    pub fn adopt(handle: LumioHandle) -> Self {
        Self { handle, live: true }
    }

    /// 是否仍处于「活」状态（未释放）。
    pub fn is_live(&self) -> bool {
        self.live
    }

    /// 取用 Handle；已释放则失败为 `InvalidHandle`(1029)。
    pub fn handle(&self) -> Result<LumioHandle, RootAbiError> {
        if !self.live {
            return Err(RootAbiError::invalid_handle(format!(
                "handle index={} generation={} 已释放",
                self.handle.index, self.handle.generation
            )));
        }
        Ok(self.handle)
    }

    /// 按 `invalidation = GenerationBump` 校验：所有者侧当前代与发出时不同即失效。
    ///
    /// 已释放的守卫一律失效——释放本身就是一次失效，不需要等换代。
    pub fn validate_generation(&self, current_generation: u32) -> Result<(), RootAbiError> {
        if !self.live {
            return Err(RootAbiError::invalid_handle(format!(
                "handle index={} generation={} 已释放",
                self.handle.index, self.handle.generation
            )));
        }
        if current_generation != self.handle.generation {
            return Err(RootAbiError::invalid_handle(format!(
                "handle index={} generation={} 已被换代（当前 {current_generation}）",
                self.handle.index, self.handle.generation
            )));
        }
        Ok(())
    }

    /// 释放；第二次起失败为 `HandleDoubleRelease`(1030)。
    pub fn release(&mut self) -> Result<(), RootAbiError> {
        if !self.live {
            return Err(RootAbiError::handle_double_release(format!(
                "handle index={} generation={} 重复释放",
                self.handle.index, self.handle.generation
            )));
        }
        self.live = false;
        Ok(())
    }
}
