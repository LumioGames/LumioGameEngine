//! lumio-core-root-abi——Root API 运行时视图与绑定（AbiExpectation、RootApiTableView、
//! bind_root_api，规格 §8.3）；slot、布局与 Handle 语义一律来自架构源，本仓不补写。
//!
//! 脚手架状态（LCE-P0-001）：`GeneratedRootApiTable`/`GeneratedApiTablesView` 必须由架构源
//! 生成（LGE-GATE-P0-001 未关闭），只读架构镜像（LCE-P0-002）也尚未建立。Gate 关闭前
//! 不存在可绑定的契约输入，因此本 crate 刻意不含任何模块与公共项。
