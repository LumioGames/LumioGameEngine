---
status: in_progress
---

# 为生成的 SHA-256 实现补 known-answer 测试与 Rust ⇄ C# ⇄ Python 三方一致性断言，并纳入 CI 必跑集合

`K[28]` 错常量（修复 `bcc8eb9`）自引入起让 Rust `ContractRuntime` 对任意输入算出错误摘要，却活过了全部 CI。根因已定位且可复核：`.github/workflows/repository-policy.yml` 对 Rust 侧只跑 `cargo check` 与 `cargo tree`，**从不执行 `cargo test`**，因此 `packages/rust/lumio-gen-contract-runtime/tests/chain.rs` 里已有的两个测试在 CI 中从未运行；而这两个测试即便运行也抓不到本缺陷——`chain_round_trip` 只断言同一个坏 hasher 自洽，不比对任何外部已知值。同时 workflow 里没有 `setup-dotnet`，C# 与 Rust 两个生成实现之间从无一致性比对。

## 涉及范围

手工修改：

- `tools/lumio_generate.py`——在生成 `tests/chain.rs` 的模板中加入 KAT 用例；`compiler_hash()` 的输入就是本文件与 `tools/lumio_contract.py`，故本次改动**必然移动 compilerHash**。
- `.github/workflows/repository-policy.yml`——新增 `cargo test --manifest-path packages/rust/Cargo.toml`；新增 `actions/setup-dotnet`（当前 workflow 无 dotnet）与三方一致性步骤。
- `tools/lumio_kat.py`（新建）——三方一致性驱动：取同一组向量，分别经 Python `hashlib`、`cargo run`/`cargo test` 暴露的 Rust 结果、`dotnet run` 暴露的 C# 结果求值并两两比对，不一致即非零退出。

经 `python3 tools/lumio_contract.py generate` 重新发布、**不得手改**：

- `packages/rust/lumio-gen-contract-runtime/tests/chain.rs`
- `packages/index.json` 及 `packages/**/artifact.descriptor.json` 全集（compilerHash 字段随生成源移动；`contract-runtime-rust` 的 outputHash 因新增测试文件一并移动，`dir_output_hash` 只跳过 `.descriptor.json`）

## 验收标准

- [x] 生成的 `tests/chain.rs` 含三条 KAT，期望值为 FIPS 180-4 公布值：`sha256("")` = `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`；`sha256("abc")` = `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`；一条长度 > 55 字节因而跨两个压缩块的向量（取 FIPS 180-4 的 `"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"` 之 448-bit 双块串，或等价的自定义向量并在注释中记录其 Python `hashlib` 求值来源）。
- [x] 把 `tools/lumio_generate.py` 的 `SHA256_RS` 中 `K[28]` 改回 `0xc6eabbdc` 后重新 `generate`，`cargo test --manifest-path packages/rust/Cargo.toml` 失败；改回 `0xc6e00bf3` 后通过——证明该测试确实守护此常量（先失败后通过，见 `test-driven-development`）。
- [x] `tools/lumio_kat.py` 在三方一致时退出 0；人为篡改任一侧常量后退出非 0。
- [x] `.github/workflows/repository-policy.yml` 中 `cargo test` 与三方一致性步骤均为必跑步骤（非 `continue-on-error`），且在 workflow 的 `baseline` job 内。
- [x] `packages/` 与 `generate` 一致：CI 现有的 outputHash / compilerHash 稳定性步骤通过。
- [x] 收口门槛全绿：`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && python3 -m py_compile tools/lumio_contract.py && python3 tools/lumio_contract.py validate`。
- [x] BaselineId 保持 `LGE-V1.4-2026-08-27` 不变；无 Schema / ID / Fixture 变更。
- [ ] compilerHash 与 `contract-runtime-rust` outputHash 的新旧值写入交回物，并按 `cross-repo-delivery` 向七仓发一次协调 re-pin 通知——本次 churn 是有正当理由的一次性移动，须与 `LumioVoxelEngine` 已完成的镜像同步（其 `main` = `4ced801`）协调，不得静默发布。

## 依赖

无。
