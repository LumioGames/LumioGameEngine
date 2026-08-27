# 0004 · 固定单 Cargo workspace（15 crate 显式 member），运行时发布闭包按 §3.7 白名单冻结并机器校验

- 日期:2026-08-28
- 状态:Accepted（生效）

## 背景

LCE-P0-001（R-00011 @ `015035b`）已按规格 §3.1/§3.2 建立根 workspace：15 个 member 显式列出、`[workspace.dependencies]` 为空（无第三方依赖）、`publish = false`、统一 toolchain 1.89.0。当前 15 个 crate 的依赖节全部为空，crate 间依赖方向尚未落进任何 `Cargo.toml`——workspace 拓扑与运行时发布边界正处于「骨架已建、边界未冻结」状态。

实现开始前有三个未决问题，任何一张实现卡单独作答都会造成边界漂移：

1. **拓扑**：单 workspace、多 workspace 还是自研打包没有仓级决定，后续任务可能各自选择。
2. **运行时闭包**：ADR 0002 只冻结了 signing 四域的隔离；整个仓的运行时发布闭包（loader 到 contracts）没有白名单。构建期工具、私钥代码、测试密钥若被运行时 crate 引用，CF-05 要求的「runtime 依赖图机器证明不含 signer/私钥/evidence 代码」无法成立。
3. **依赖/许可证政策**：`deny.toml`/`about.toml`（R-00011 已建）只有规则没有决策记录，「为什么白名单制、为什么拒 MPL」缺少 ADR 依据。

冲突案例：`lumio-core-signer-tool` 已带空占位 `test-provider` feature（保证规格 §3.4 `sign-test` 命令可执行到 blocked 守卫，实际 provider 由 LCE-P0-012 实现）。若没有闭包白名单，任何运行时 crate 都可能在后续任务里「顺手」依赖 signer-tool、test-provider 或测试密钥 fixture 而不被机器拦截——ADR 0002 的四域隔离只剩文档约束。

本 ADR 是规格 §1.4 预登记的「Workspace 与运行时闭包」实现决策，只固化本仓代码组织与发布边界；不重开 ADR 0001—0003，不定义任何公共契约（公共语义唯一来源仍是架构源 `LGE-V1.2-2026-08-27`）。

## 决策

### 1. 拓扑：单 Cargo workspace（选定），否决多 workspace 与自研打包

| 候选 | 结论 | 理由 |
| --- | --- | --- |
| 单 workspace、15 member 显式列出 | **选定** | 单一 `Cargo.lock` 保证全仓同一依赖闭包；`[workspace.dependencies]` 单点精确锁版；toolchain/`cargo-deny`/`cargo-about` 单点门禁；15 crate 规模下管理成本最低。member 显式列出、禁 glob/exclude（规格 §3.2）。 |
| 多 workspace（运行时 / 构建工具分仓） | 否决（保留退出路径，见第 6 条） | 编译期物理隔离是真实收益，但代价是多 lock 漂移、跨 workspace 版本不同步、SBOM 需拼接、门禁双份维护；当前规模下用第 2 条闭包检查替代物理隔离，成本更低。 |
| 自研打包（绕开 Cargo 编织依赖） | 否决 | 放弃 Cargo 依赖解析/lock/审计生态；与 ADR 0001「platform 是唯一构建执行入口、内部经 Cargo/rustc 执行」冲突，无对应收益。 |

### 2. 运行时发布闭包白名单（与规格 §3.7 逐项一致）

允许闭包（根为 `lumio-core-loader`）：

```text
lumio-core-loader
  -> lumio-core-runtime-verifier
  -> lumio-core-trust-policy
  -> lumio-core-platform-runtime
  -> lumio-core-platform-contracts
  -> lumio-core-root-abi
  -> lumio-core-contracts
```

这 7 个 crate 是唯一允许出现在运行时发布闭包中的仓内 package。机器判据沿用规格 §3.7：`cargo tree -p lumio-core-loader --edges normal`（`just runtime-deps`，由后续任务接入）中出现下列任一项即失败：

```text
lumio-core-composition
lumio-core-root-abi-generator
lumio-core-platform-build
lumio-core-manifest
lumio-core-evidence-generator
lumio-core-signer-tool        （绝对否——ADR 0002）
lumio-core-smoke
任何 KMS/私钥 SDK
任何 test-provider/test-key package
```

`lumio-core-diagnostics` 不在闭包内也不在拒绝清单里：它由 Host 显式装配，Loader 不依赖它（ADR 0003）；上述 `cargo tree` 检查以 loader 为根，天然不含它。

### 3. crate 间依赖方向冻结（规格 §5.1，本 ADR 不新增、不删减边）

- `root-abi -> contracts`
- `root-abi-generator -> contracts, composition`
- `platform-contracts -> contracts`
- `platform-runtime -> platform-contracts, root-abi`
- `platform-build -> platform-contracts, composition, contracts`
- `manifest -> composition, contracts`
- `evidence-generator -> composition, contracts`
- `signer-tool -> contracts`
- `trust-policy -> contracts`
- `runtime-verifier -> trust-policy, platform-contracts, contracts`
- `loader -> runtime-verifier, platform-runtime, platform-contracts, root-abi, contracts`
- `diagnostics -> contracts`
- `smoke -> loader, diagnostics, platform-build, manifest, evidence-generator, signer-tool`

禁止边（沿用 §5.1/ADR 0001—0003）：`Loader -> Manifest 生成器`、`Loader -> Signer`、`Loader -> Evidence`、生产模块 `-> Smoke`、生产模块 `-> Diagnostics`。

### 4. 现状对照（`015035b` 实测：`cargo metadata --no-deps` 15 package / 7 bin / 0 依赖）

| package | 类型 | 进入运行时闭包 | bin |
| --- | --- | --- | --- |
| `lumio-core-composition` | BuildPlan library + CLI | 否 | `lumio-core-compose` |
| `lumio-core-contracts` | 架构源 ContractTypes 唯一 re-export 面 | 是 | — |
| `lumio-core-root-abi` | Root API runtime view/binder | 是 | — |
| `lumio-core-root-abi-generator` | ABI 生成 Adapter | 否 | `lumio-core-root-abi-generator` |
| `lumio-core-platform-contracts` | LoadBackend/OpenedArtifact 契约 | 是 | — |
| `lumio-core-platform-runtime` | OS LoadBackend | 是 | — |
| `lumio-core-platform-build` | 唯一 build/link/layout/index 执行器 | 否 | `lumio-core-platform-build` |
| `lumio-core-manifest` | ManifestBody 生成/验证 CLI | 否 | `lumio-core-manifest` |
| `lumio-core-evidence-generator` | SBOM/License/Provenance 生成 | 否 | `lumio-core-evidence-generator` |
| `lumio-core-signer-tool` | 离线/CI Signer | **绝对否** | `lumio-core-signer-tool` |
| `lumio-core-runtime-verifier` | 运行时包验证 | 是 | — |
| `lumio-core-trust-policy` | 只读信任策略 | 是 | — |
| `lumio-core-loader` | Loader 状态机/Lease | 是（闭包根） | — |
| `lumio-core-diagnostics` | Host 可选观测 Adapter | 否（Host 显式装配） | — |
| `lumio-core-smoke` | E2E 验证 bin/test | 否 | `lumio-core-smoke` |

member 列表、闭包成员、bin 清单与根 `Cargo.toml`（15 个显式 member）及各 crate manifest 一一对应；现状依赖图为**空**（Current Fact）——第 3 条冻结的是方向与禁边，不是声称已实现，crate 间边由后续任务按此落进 `Cargo.toml`。这 15 个 package 是八个既有模块内部的编译/安全域切分，不是新增第九个模块；模块所有权仍按 `modules/README.md` 八个单元计算。

### 5. 依赖与许可证政策（决策依据 `deny.toml`/`about.toml`，R-00011 已建）

- 直接依赖一律先登记 `[workspace.dependencies]` 精确版本，crate 只能经 `<dep>.workspace = true` 消费；`Cargo.lock` 提交；git 依赖必须固定 40 位 commit，禁 branch/tag 浮动。
- 运行时闭包内 crate 默认 feature 必须最小，禁止 KMS、Signer、测试密钥、网络 client 与 build-tool feature；`signer-tool` 的 `test-provider` 仅限其自身 CLI，任何闭包内 crate 不得引用。
- 许可证白名单制（deny-by-default）：`deny.toml [licenses].allow` = Apache-2.0、MIT、Apache-2.0 WITH LLVM-exception、BSD-2-Clause、BSD-3-Clause、ISC、Zlib、Unicode-3.0、Unicode-DFS-2016、CDLA-Permissive-2.0；GPL/AGPL/SSPL/LGPL/MPL 全系默认拒绝，出现即「需法务审核」，放行只能走 `[[licenses.exceptions]]`（逐 crate、附法务评审与新 ADR），不得扩白名单；`about.toml` 与之保持同一口径。

### 6. 维护 owner 与退出路径

- **维护 owner**：workspace 拓扑、member 清单与运行时闭包边界由「程序·协议/公共」（共享合同维护者）角色维护；根 `Cargo.toml`/`Cargo.lock`/`justfile` 门禁入口是共享热点，改动必须在任务卡显式声明。模块内 crate 归属按 `modules/README.md` 八模块。增删 member、改动闭包或禁边**必须新增 ADR 取代本条**（决策记录不改写），实现卡不得自行扩边。
- **退出路径**（何时以及如何拆多 workspace / 独立 package）：第 2 条白名单就是拆分缝——7 个运行时 crate 可沿 `loader -> … -> contracts` 边界整体抽出为独立 workspace 或独立 package，对外 Root ABI 不变。触发条件示例：P1 平台矩阵使构建期工具显著拖慢 CI、运行时与构建工具需要不同节奏发版、外部消费方需要只取运行时 crate。届时以本 ADR 的允许闭包为新 workspace 的初始成员集，新增 ADR 取代本条，并同步 `deny.toml`/`about.toml`/`Cargo.lock` 三处门禁；`smoke`/`evidence-generator`/`signer-tool` 可先独立成工具仓而不触碰运行时闭包。

## 后果

- 后续任务卡一律引用本 ADR 获取 workspace 拓扑与发布边界，不在各卡重新定义；`just runtime-deps` 闭包检查落地前，白名单靠任务卡纪律与 review 保证，落地后由机器强制。
- 单 workspace 的代价：构建期 crate 与运行时 crate 共享同一 lock 与工具链，构建工具升级会触及全仓 lock；以单点门禁换取隔离。
- 放弃多 workspace 的编译期物理隔离：违规引用要靠 `cargo tree` 检查事后发现，而非物理上不可能。
- 15 member 的增删、闭包成员变化或禁边调整都需要重开 ADR，流程成本前置。
