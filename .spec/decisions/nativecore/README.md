# Decisions(决策记录 · ADR)

本目录是 **NativeCore(`engine/native/`)内部实现决策**的子命名空间,编号自 `0001` 起,与上级目录的 `ADR-NNN` 是两套编号、互不占号。

> 全仓决策记录的权威索引是上级 [`.spec/decisions/README.md`](../README.md);本目录只是它的一个分区。沿用 `000N-` 文件名是因为 `engine/native/` 下 51 个源码 / 配置文件以「ADR-0006」这类写法引用它们,改号无收益。

## 怎么写一条 ADR

- 一个决策 = 一个文件 `NNNN-<slug>.md`,编号从 `0001` 递增;写完在下方索引加一行。
- **一旦记录不改写**:被推翻就新增一条,把旧的状态标成「被 NNNN 取代」,历史留痕。
- 无 frontmatter。格式照抄:

      # NNNN · <一句话决策>

      - 日期:YYYY-MM-DD
      - 状态:生效 | 被 NNNN 取代

      ## 背景
      面对什么问题。

      ## 决策
      定了什么。

      ## 后果
      接受了什么代价。

## 索引

| 编号 | 决策 | 状态 |
|------|------|------|
| [0001](0001-build-orchestration-boundary.md) | composition 只产不可变 BuildPlan，platform 是唯一构建执行入口 | 生效 |
| [0002](0002-supply-chain-domain-split.md) | signing 内部按四个安全域分拆，运行时只发布 runtime-verifier | 生效 |
| [0003](0003-observation-validation-planes.md) | diagnostics 收窄为观测适配平面，smoke 定位为验证平面 | 生效 |
| [0004](0004-workspace-runtime-boundary.md) | 固定单 Cargo workspace 15 crate，运行时发布闭包按规格 §3.7 白名单冻结 | 生效 |
| [0006](0006-internal-build-plan-freeze.md) | BuildPlan 定为仓内确定性 JSON（plan_format_version=1），sidecar Digest 原子冻结，platform 只读 | 生效 |
| [0007](0007-composition-config-toml-parser.md) | compose 配置解析选定 `toml` crate 并精确锁版，不自研 TOML 子集解析器 | 生效 |
| [0008](0008-opened-artifact-set-construction-inversion.md) | `OpenedArtifactSet`/`MappedNativeImage` 用构造反转保持私有构造器，feature gate 不用于跨 crate 可见性 | 生效 |
| [0009](0009-root-abi-generator-adapter-boundary.md) | root-abi generator 摘要链每环须有外部锚点，descriptor 不得自证；报告不得声称没做过的检查 | 生效 |
| [0010](0010-root-abi-runtime-unfrozen-semantics-seams.md) | 上游明确未冻结的语义（capability 位映射、per-table version）保持缺位，`capability_bits` 只做不透明相等校验 | 生效 |
