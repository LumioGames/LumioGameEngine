# Decisions(决策记录 · ADR)

用 ADR(Architecture Decision Record)记录决策:为什么这样调度、为什么定这种结构、为什么划这条边界。**本目录是全仓决策记录的唯一落点**——功能内决策与框架级决策都记这里,feature 文档只描述设计现状,不留决策记录。

> 跨仓公共语义的决策只在 `LumioGameEngineArchitecture` 维护；本目录仅记录 CoreEngine 内部实现决策，并从 `0001` 开始编号。

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
