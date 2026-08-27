---
status: completed
---

# 建立 workspace、工具锁与基础门禁（LCE-P0-001 / Workflow R-00011）

一句话：创建可解析但不假装已有实现的 15-crate Cargo workspace，固定 Rust/工具版本与许可证/source 门禁。来源规格：`docs/LumioCoreEngine_Framework_Scaffolding_Spec_v1.0.md` §3、§18 LCE-P0-001（基线 commit f3c9920）。

## 涉及范围

- 根：`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`rustfmt.toml`、`clippy.toml`、`deny.toml`、`about.toml`、`about.hbs`、`nextest.toml`、`.cargo/config.toml`、`justfile`
- 工具锁：`tools/tools.lock.toml`、`tools/checksums.sha256`、`tools/verify-tool-lock.sh`
- 15 个 crate 的 `Cargo.toml` 与 `src/lib.rs`（composition/root-abi×3/platform×3/manifest/signing×4/loader/diagnostics/smoke），7 个 CLI `src/bin/*.rs`
- 本任务卡自身

## 验收标准

- [ ] `cargo metadata --locked --format-version 1` 成功，15 个 package 均可发现
- [ ] `cargo check --workspace --all-targets --locked` 成功（退出码 0）
- [ ] `cargo deny check` 成功：无未锁定 git/registry；许可证策略拒绝 GPL/AGPL 等强传染许可证并显示「需法务审核」
- [ ] `tools/verify-tool-lock.sh` 成功（工具版本/SHA-256/许可证策略完整性校验）
- [ ] `Cargo.lock` 已生成并留在工作区待提交（提交动作归主 loop）
- [ ] 7 个 CLI 二进制任一调用均以非零退出码结束，stderr 含 `BlockedOnArchitectureGate`
- [ ] library 入口只有 crate 文档，无虚假成功 API

## 依赖

无

## 证据记录（在途；验收勾选归主 loop/reviewer）

实现前 Red（基线 f3c9920，2026-08-27）：

- `cargo metadata --locked` exit 101（无 Cargo.toml）；`cargo check --workspace --all-targets --locked` exit 101；`cargo deny check` exit 1；`tools/verify-tool-lock.sh` exit 127。

实现后 Green（工具链 1.89.0-x86_64-pc-windows-msvc，rust-toolchain.toml 锁定）：

- `cargo metadata --locked --format-version 1` exit 0；`packages=15`、`workspace_members=15`，crate 名与规格 §3.1 逐一一致，bin 7 个（compose/root-abi-generator/platform-build/manifest/evidence-generator/signer-tool/smoke）。
- `cargo check --workspace --all-targets --locked` exit 0；`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` exit 0；`cargo fmt --all --check` exit 0。
- `cargo deny check` exit 0（advisories/bans/licenses/sources 全 ok；空依赖图下白名单未命中仅为 warning）。`grep -c 'source = ' Cargo.lock` = 0（无 git/外部 registry 来源），锁内 15 package。
- `tools/verify-tool-lock.sh` exit 0：4 工具（cargo-deny 0.20.2 / cargo-about 0.9.2 / cargo-nextest 0.9.114 / just 1.58.0）本机二进制 SHA-256 全部比对通过；许可证策略完整性通过并显示「拒绝（需法务审核）」。
- `just check`（fmt-check+clippy+deny+about+tool-lock）exit 0；`just about` 生成报告（15 crate，Apache-2.0）。
- 7 个 CLI：本机无 MSVC/SDK/WSL/docker，无法链接原生二进制；以 wasm32-wasip1（rust-lld 自包含）构建后经 wasmtime 执行：7/7 非零退出 + stderr 含 `BlockedOnArchitectureGate`（代码路径设 `ExitCode::from(5)`；wasmtime 下观察到非零 1，原生退出码 5 未在本机演示）。
- lib.rs 公共 API 扫描 `grep -rn '^pub '` = 0 处。

Negative：

- 篡改 tools.lock.toml（清空 artifact_sha256）→ 脚本 exit 1「字段缺失」；篡改 checksums.sha256（翻转 1 hex）→ exit 1「二进制 SHA-256 漂移」；deny.toml 白名单注入 "GPL-3.0" → exit 1「白名单混入传染许可证」。恢复后全绿（备份 diff 校验）。
- 许可证探针（临时 scratch crate + 本仓 deny.toml）：GPL-3.0-only / AGPL-3.0-only / MPL-2.0 → cargo-deny exit 4，诊断显示被拒许可证 ID 与 Copyleft 标注；MIT 对照 exit 0。
- 浮动 git 依赖（branch 引用 serde）→ cargo-deny sources exit 8 `error[source-not-allowed]: detected 'git' source not explicitly allowed`。
- 篡改 Cargo.lock（version 改 0.2.0）→ `cargo metadata --locked` exit 101；恢复后 exit 0。
- `just compose bad-profile` → exit 1「未知 profile」。

收口门槛：

- `node .spec/tools/spec-lint.mjs` → OK（exit 0）。
- `node --test .spec/tools/spec-lint.test.mjs` → 全部用例失败（EPERM symlink，Windows 无符号链接权限）；**预先存在**：对 pristine HEAD（git archive f3c9920）运行同样失败（fixture 需 fs.symlinkSync）。非本卡改动引入，属宿主环境已知缺口。

备注（非本卡改动/提交注意）：

- 会话开始后 `.agents/skills`、`.claude/agents`、`.claude/skills` 被外部进程（宿主初始化）重建为绝对路径目标，`git status` 显示 D；spec-lint 通过；本卡未触碰，处置归主 loop。
- `.gitignore` 未覆盖 `target/`（`build/`、`dist/` 同理，规格 §3.3 规定不提交）——`.gitignore` 不在本卡文件集，主 loop 提交前须补齐，**不得 `git add -A`**。
- `tools/verify-tool-lock.sh` 已 `chmod +x`；git 提交时需保留执行位（建议 `git update-index --add --chmod=+x tools/verify-tool-lock.sh` 或 `core.filemode` 处理）。
- `just nextest` 与规格 §20.1 命令差一个 `--config-file nextest.toml`：nextest ≥0.9.100 不再自动发现仓库根 nextest.toml（改查 `.config/nextest.toml`）；本仓按规格 §3.1 保留根文件布局。
- 本机（无链接器）`cargo build`/`nextest` 会因 MSVC link.exe 缺失失败，属宿主环境限制；CI/Linux 主机不受影响。
