---
name: 2026-09-05-nativecore-w0-card-and-kickoff
description: NativeCore 补单草稿——W0「清掉旧合同制残留」卡面（待授权建卡）与开工提示词；派 NativeCore 清理活时查
metadata:
  type: doc
  status: 设计中
---

# NativeCore · W0 清理卡卡面与开工提示词

> 来源：[`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.1 与 §6 D1 / D5（Owner 2026-09-05：三层全清、一张卡做完、`lumio-native-ffi` 整 crate 删）。卡面按 workflow-ops `card-spec`（背景 / 目标 / 验收 / 边界）。**已落单：R-00473**（`01a07226-516e-7f1c-9cfb-c6a1946ca22d`，RM-00002，8 条原生验收项，2026-09-05 Owner 授权后由主 loop 写入并读回）。

## 一、卡面（RM-00002 · R-00473）

**标题**：`[程序·协议/公共][NativeCore·W0] 退出旧合同制：删复印件与 CI 断言、删 contract-types 旧合同门与 native-ffi、内核不再持有旧合同数字`

### 背景

8 月的规矩是「架构仓发布 Baseline 合同（`LGE-V1.4-2026-08-27`，含 `lumio_core.h` / Root ABI bundle），实现仓复印一份、对着复印件写代码、CI 校对复印件」。9 月初架构仓按 ADR-059 与 Living Architecture 废止这套规矩：合同、`packages/`、校验器、mirror 全部删除，唯一 ABI 真值改为 `engine/abi/native-abi.json`，SDK 以 Rust 路径依赖直接编入 NativeCore 的 `lumio-kernel` 与 `lumio-timer`。NativeCore 未跟改：复印件（`docs/architecture/`，276 KB）、README 合同口径、CI 校对步骤、`lumio-contract-types` 对旧合同的布局 golden 与 digest 漂移门、`lumio-native-ffi` 按旧头文件拼的 provider 表（`lumio_core_init` 槽为空、等已退役的 CoreEngine 来取）、`lumio-kernel` 内旧合同错误码 1044–1053 与 capability 注册表键，全部仍在。`architecture.md` §7 第 4 条「活动源码和 CI 不再依赖 CoreEngine、Baselines 或 contract mirror」是迁移完成条件，本仓目前不满足。Owner 2026-09-05 裁决：三层全清，一张卡做完，不留「先兼容」。

### 目标

NativeCore 仓内不再存在任何旧合同制的东西：没有 Baseline 复印件、没有对着旧合同的测试门、没有等退役方来取的表、没有旧合同的数字；仓只剩纯 Rust 内核 crate（`lumio-kernel` / `lumio-job` / `lumio-spatial` / `lumio-timer` / `lumio-platform` / `lumio-test-support`，加默认关的 `lumio-codec` / `lumio-diagnostics`），唯一对外形态是被架构仓 SDK 以路径依赖编入。

### 验收（逐条可机器判定）

1. 第 1 层 · 文档与 CI：`docs/architecture/` 整目录与 `.baseline.sha256` 删除；`.gitattributes` 相关行删除；`.github/workflows/repository-policy.yml` 的 `readme` job 删去全部基线字符串与 sha256 断言，只保留结构性检查（README 必含节改为 Living Architecture 口径）；已跟踪文件中 `git grep -l -E "LGE-V1\.[0-9]|LumioGameEngineArchitecture|root-abi-bundle|lumio_core\.h|lumio_core_get_api_v1|CoreEngine"` **零命中**（`docs/2026-08-27-native-core-module-implementation-frame.md` 与 `.spec/decisions/000[1-8]` 属历史记录，按 spec-lint 口径处理：历史 ADR 不改写，其余删除或改写）。
2. 第 2 层 · 两个 crate：`crates/lumio-native-ffi/` 删除并从 workspace members 移除；`crates/lumio-contract-types/` 删除 `generated.rs` / `generated_data.rs` / `layout.rs` / `registry.rs` 中绑定旧合同的全部内容与对应测试（`wrong_baseline_is_rejected`、`generated_layout_matches_manifest`、`generated_contract_revision_is_readable`、`capability_keys_bind_registry`、`registry_values_are_unique`）；若删后 crate 不再承载任何本仓内部需要的类型则整 crate 删；`xtask` 删除 `gen-contracts` / `check-baseline` 及 `baseline.rs` / `contracts.rs`，`dump-symbols` 改为「workspace 内不存在 cdylib / staticlib 目标」的断言或删除，`check-dep-dag` 白名单同步。
3. 第 3 层 · 内核旧数字：`lumio-kernel` 删除 `error::to_architecture_error_code` 与 1044–1053 一切数值及 `mapping_is_total_for_all_categories` 等对应测试，`ErrorCategory` 保留为内部枚举（跨边界映射归架构仓 `sdk-native` 插头对 `native-abi.json` 状态码，本卡不加状态码）；`capability` 删除对旧合同注册表键的投影（ADR 0006 以新 ADR 取代：capability 键改为本仓内部常量或整块删除，取其一并写明理由）。
4. timer 口径统一：`modules/timer/README.md`、`docs/specs/native-core-module-map.md`、`.spec/knowledge/standards/repository-architecture.md` 三处改为同一句：「内核 `lumio-timer` 在 NativeCore；C ABI 插头在架构仓 `engine/native/modules/sdk-native/src/timer.rs`；经 `engine/abi/native-abi.json` 的 `timer_*` 槽到达托管侧」。
5. README 重写为 Living Architecture 口径：这仓是什么、有哪些 crate、SDK 怎样以路径依赖编入、收口门槛是什么；删除「Architecture Gate」「Baseline」「Generated Contract Dependencies」「Release Composition」等旧制度节。
6. 本仓 `.spec/decisions/` 新增一条 ADR（编号现查），记录退出旧合同制、`lumio-native-ffi` 删除、`contract-types` 收缩或删除、ADR 0001 / 0006 中被取代的条目；`.spec/knowledge/` 与模块 README 的 BaselineStatus / 架构基线 行删除；本仓 `node .spec/tools/spec-lint.mjs` OK。
7. 门全绿并附输出：`cargo fmt --all --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo build --workspace`、`cargo test --workspace`、`cargo xtask check-dep-dag`（及保留下来的 xtask 子命令）。
8. 唯一消费者不受影响并附输出：在架构仓 `engine/native` 执行 `cargo build -p lumio-engine-native` 与 `cargo test -p lumio-engine-native` 通过（`sdk-native` 只引用 `lumio_kernel::handle::HandleKey` 与 `lumio_timer::TimerManager`；若本卡改动使其编译失败，**停下标 BLOCKED 上报**，不得改架构仓文件）。

### 边界

- 只删、只改口径，不加任何功能；不动 `lumio-timer` / `lumio-job` / `lumio-spatial` 的逻辑与公开 API；`lumio-codec` / `lumio-diagnostics` 不动（D6）。
- 不改架构仓任何文件；不恢复任何已删的 Schema / Fixture / 镜像工具链（`repository-architecture.md`「变更顺序」第 4 条）。
- 不为内核错误码在 `native-abi.json` 新增状态码——内核函数进 ABI 时另开卡。
- 历史 ADR 正文不改写，只新增取代条目。

## 二、开工提示词（另开一个 Claude Code 窗口，工作目录 `~/LumioGames/LumioNativeCore`，整段粘贴）

```text
你是 LumioNativeCore 仓的 Native 内核工程师。这一轮只做一张卡：Workflow（lumiogamesengine）R-00473
「[NativeCore·W0] 退出旧合同制：删复印件与 CI 断言、删 contract-types 旧合同门与 native-ffi、内核不再持有旧合同数字」
https://lumiogamesengine.workflow.games/requirements/01a07226-516e-7f1c-9cfb-c6a1946ca22d

【守门（第一步，任一不符立即停下回报 BLOCKED，不得继续）】
1. 本仓 origin/main 是 70b9834 或其后继；工作区干净（两个未跟踪 .DS_Store 可忽略）。
2. 架构仓 ~/LumioGames/LumioGameEngine 的 engine/abi/native-abi.json 存在，且 engine/native/modules/sdk-native/Cargo.toml
   仍以路径依赖引用 ../../../../../LumioNativeCore/crates/lumio-kernel 与 lumio-timer（这是本仓唯一消费者的形状）。
3. 用 workflow-execute 读全 R-00473：正文 + 8 条验收项 + 评论；读不到就停。

【指路】
- 卡面正文就是任务书：三层清理按验收 1 → 8 的顺序做。第 1 层是文档与 CI，第 2 层删 lumio-native-ffi 整 crate 并把
  lumio-contract-types 里对旧合同（LGE-V1.4 / Root ABI bundle / lumio_core.h）的 golden、digest 漂移门、xtask gen-contracts /
  check-baseline 全删（删空则整 crate 删），第 3 层删 lumio-kernel 里旧合同的错误码数值 1044–1053 与 capability 注册表键投影。
- 背景一句话：架构仓已按 ADR-059 废止「Baseline 合同 + 复印 + 校对」制度，唯一 ABI 真值是 native-abi.json，SDK 直接以 Rust
  路径依赖编入本仓；本仓要变成只剩纯 Rust 内核 crate 的样子。设计现状与裁决见架构仓
  .spec/reviews/2026-09-05-engine-repos-progress-assessment.md §2.1 与 §6 D1/D5，只读。
- timer 三处文档统一成同一句：内核 lumio-timer 在本仓；C ABI 插头在架构仓 engine/native/modules/sdk-native/src/timer.rs；
  经 native-abi.json 的 timer_* 槽到达托管侧。
- 本仓 .spec/decisions/ 新增一条 ADR 记录这次退出（编号落笔时现查最高号），ADR 0001 / 0006 只加「被 NNNN 取代」，不改写。
- 验收第 8 条在架构仓只读执行 cargo build / cargo test -p lumio-engine-native；若本仓改动让它编不过，停下标 BLOCKED 上报，
  不改架构仓任何文件。

【立规】
- 领卡先经 Workflow 流转「实现中」并写 reason；改动在 feat/r-00473-exit-legacy-contract 分支，先 push 再回写证据。
- 小步提交，每层一个提交；每次提交前 cargo fmt --all --check、cargo clippy --workspace --all-targets -- -D warnings、
  cargo build --workspace、cargo test --workspace、cargo xtask check-dep-dag（及保留下来的 xtask 子命令）、
  node .spec/tools/spec-lint.mjs 全部 exit 0。测试证据必须是本机实跑的命令与输出，cargo check 不算。
- 交付 = 改动清单 + 验证证据（命令 + 关键输出）+ known gaps + 沉淀落点（本仓新 ADR），写成 PR 描述并同步为 R-00473 的证据评论，
  评论只引用已推送 origin 的提交号；做完流转「验收中」，「已完成」由总调度核验后流转。
- 遇到 bug 或测试失败先找根因再改；同一问题修三次不成，停下上报。

【禁区】
- 不加任何功能；不动 lumio-timer / lumio-job / lumio-spatial 的逻辑与公开 API；lumio-codec / lumio-diagnostics 不动。
- 不改架构仓与任何上层仓的文件；不恢复任何已删的 Schema / Fixture / 镜像工具链；不为内核错误码在 native-abi.json 加状态码。
- 不写「先兼容、以后再清」的中间态，不留别名、不留兜底常量；历史 ADR 正文不改写。
- 不在提交里夹带 .DS_Store 或本任务之外的文件。
```
