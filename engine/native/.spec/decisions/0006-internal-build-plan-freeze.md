# 0006 · BuildPlan 冻结为仓内确定性 JSON v1（plan_format_version=1），sidecar Digest + 原子目录发布，全消费者只读

- 日期:2026-08-28
- 状态:Accepted（生效）

## 背景

ADR 0001 已冻结所有权边界（composition 只产不可变 BuildPlan，platform 是唯一构建执行入口且不得反向修改），规格 §7 已给出 BuildPlan 的字段面，但**字节级协议没有任何仓级决定**：编码格式（Cargo/JSON/CBOR/自研）、键与集合顺序、sidecar Digest、原子发布、环境变量白名单、版本迁移全部未决。按规格 §16.2 依赖序，LCE-P0-004（composition 编码/冻结实现）与 LCE-P0-008（platform 消费计划）都固定依赖本决策；两卡在不同 wave，若各自作答必然漂移。

未决协议下的具体冲突案例：

1. **半计划**：compose 直接写 `build-plan.json`，进程写中途崩溃——platform 消费到半个 JSON，或消费到完整 JSON 但 sidecar 是旧文件的，摘要链静默断裂。
2. **覆盖发布**：重复 compose 到已存在计划目录直接覆盖——违反 ADR 0001「一经发布不可变」，且下游已登记的 `build_plan_digest`（ProvenanceRecord、执行记录、摘要链）失配无处报警。
3. **原地补写**：platform 发现缺平台特化参数（rustflags/布局），「顺手」改计划再执行——ADR 0001 明令禁止，但没有字节级判据（sidecar 是否必须存在、谁来验、何时验）就无法机器拦截。
4. **双解析器漂移**：composition 与 platform 各写一套序列化/反序列化，键序、转义、数字规则不同——同一语义输入两次 compose 字节不同，规格 §7.5「同输入精确字节相同」的验收间歇性失败。

定位（非目标的对偶）：BuildPlan 是**仓内版本化执行输入**，不是公共架构 Schema——架构源 CanonicalSerializer（AG-005 域）管公共载荷（CoreEngineManifestBody 等）的互操作；BuildPlan 不注册进架构源，其确定性规则由本仓决定，与架构源实现、Golden 不共享。

编号说明：0005 由 LCE-ADR-005（P0 Linux 同对象加载）预留（规格 §16.2 垂直段，LCE-P0-013 之后落卡）；本 ADR 按规格 §1.4 预登记提前使用 0006，序列暂缺 0005 不表示遗漏。

## 决策

### 1. 编码候选与选定：仓内确定性 JSON

| 候选 | 结论 | 理由 |
| --- | --- | --- |
| Cargo 原生（不落独立计划文件，Cargo.toml/lock + 命令行即执行输入） | 否决 | 装不下 Source Lock、Architecture 锁、PackageLayout、环境白名单等字段；argv 无版本化自描述；ADR 0001 的 Source → BuildPlan → ArtifactIndex 摘要链断链；「已发布不可覆盖」无从表达。 |
| **仓内确定性 JSON**（typed 结构 + 紧凑规范编码） | **选定** | 可 diff、可人工审计（`compose print`、dist/records 留档）；serde_json 成熟、MIT/Apache-2.0 在 ADR 0004 许可证白名单内；字节规则可完全钉死并以 Golden 锁定；计划为 KB 级，体积不是约束。 |
| CBOR（ciborium/serde_cbor） | 否决（保留退出路径，见第 10 条） | 字节省、解析快，但不可直接 diff/审计，篡改排查成本高；P0 无体积/性能压力；多引入一个依赖面。 |
| 自研二进制格式 | 否决 | Unicode、转义、数字、键序极易漂移（规格 §4 对 Canonical Serialization 的同一结论）；无差异化收益，手写解析器是漂移与注入温床。 |

不复用架构源 CanonicalSerializer：它是公共载荷的互操作契约（AG-005 域），BuildPlan 是仓内输入（见「非目标」），两者不共享实现与 Golden。

### 2. V1 规范编码规则（字节级）

- UTF-8、无 BOM；紧凑 JSON（无缩进、无多余空白）；文件恰以一个 LF 结尾。
- 数字只允许无符号整数与布尔，**禁止浮点**（字段面只有 u32/bool/String/Vec/Map（规格 §7.3，无 usize 字段））。
- 字符串最小转义：控制字符用 `\u00XX`（小写十六进制），不转义 `/`，非 ASCII 以 UTF-8 原样输出（锁定 serde_json 默认行为，Golden 固定）。
- **对象键按结构体字段声明序发出**（规格 §7.3 的字段序即规范序）；解码端 typed 结构 + `deny_unknown_fields`，不依赖键序、拒绝未知键——Schema 演进只走版本号，不做前向兼容。
- 语义集合入计划前按 UTF-8 字节序排序并去重：feature enabled/disabled、rustflags、environment 键（BTreeMap 天然有序）。rustflags 去重后若存在同键不同值（如两个 `-C opt-level`）的顺序敏感冲突，compose 期拒绝，不允许顺序语义潜入计划。
- `build_invocations` 按 (source_component, package) 固定排序；`source_lock.repositories` 固定 [LumioNativeCore, LumioVoxelEngine] 声明序（由 §7.3 的定长数组与 SourceComponent 声明序蕴含）。
- 计划内不得出现时间戳、生成耗时、CI run ID、随机数、主机名、用户目录等非确定字段。
- 实现载体：typed serde 结构 + serde_json 紧凑序列化；serde/serde_json 由 LCE-P0-004 按 ADR 0004 第 5 条经 `[workspace.dependencies]` 精确锁版引入（本 ADR 不锁具体版本号）；LCE-P0-004 落地时以仓内 Golden 字节 Fixture 锁定本条全部规则，Golden 变化即编码漂移即版本事件。
- 同一语义输入（map/set/feature 顺序置换）必须产出字节一致的计划——`reproducible_plan.rs` 属性测试的直接依据（LCE-P0-004）。

### 3. plan_format_version 唯一性（= 1）

- 当前唯一合法值是 **1**（`plan_format_version: u32`，规格 §7.3）。
- 唯一写方 composition 只产出 1；一切读方（root-abi generator、platform-build、manifest、evidence-generator、`compose verify`）在**任何其他解析之前**先比对版本，≠ 1 一律整体拒绝（fail-closed），错误落在各自模块自己的仓内错误面（composition 侧 `InvalidConfiguration` 家族），不占公共 ErrorCode（规格 §6.2）。
- composition/platform 测试（LCE-P0-004 `reproducible_plan`/`feature_resolution`、LCE-P0-008 `plan_immutability`/负向篡改）以本 ADR 为唯一出处引用常量 `plan_format_version == 1`，不得各自再定义。

### 4. 绝对路径处理

- 计划内一切路径必须是 `WorkspaceRelativePath`：UTF-8、正斜杠、非空、无盘符/NUL/`.`/`..`/重复分隔符、相对 workspace root（与规格 §6.3 PackagePath 同型的仓内不变量）。
- `ComposeRequest` 输入侧的绝对路径（workspace root、checkout、lock 文件）由 compose 期换算为 WorkspaceRelativePath；无法相对化（workspace 之外）即 `InvalidConfiguration`。
- `ArchitectureDocumentRef.source_path` 是相对**架构源提交树**的路径（由 `architecture_source_commit` 锁定），不是本 workspace 路径，两者不得混用。
- 工具只记 `ToolReference`（tool_id/version/executable_sha256）；主机绝对工具路径**绝不写入计划**——platform 执行期从 `tools/tools.lock.toml` 解析并复核 SHA-256（规格 §7.4）。
- 消费者用自身 CLI 显式传入的 workspace 根解析计划内相对路径；计划不自带 workspace 根。

### 5. 环境变量白名单（封闭清单）

- `BuildInvocation.environment` 只允许出现封闭清单内的键；V1 清单只有一个：

```text
CARGO_NET_OFFLINE = "true" | "false"
```

- 配置或计划中出现清单外任何键（PATH、HOME、USER、TMPDIR、locale、RUSTFLAGS、其他 CARGO_*）= compose 期 `InvalidConfiguration` 拒绝（退出码 2，规格 §7.4）。
- 值必须是字面字符串，禁止引用或展开 ambient 环境；compose 不得读取 ambient 环境来生成计划字段。
- 语义参数走结构化字段而非环境透传：rustflags → `BuildInvocation.rustflags`，feature → `feature_set`，profile → `build_profile`。
- platform 执行子进程时的 ambient 继承与 PATH 策略归 LCE-P0-008（原则不变：argv/env 完全来自已验证计划）；本 ADR 只冻结「计划内允许出现什么」。
- 白名单演化：新增键（不改变既有 v1 计划字节）不升版本；删键或收紧值域会使既有 v1 计划读回失败，必须升 `plan_format_version`（见第 10 条）。

### 6. sidecar Digest 与摘要链

- 冻结产物是计划目录内三个文件：`build-plan.json`、`build-plan.sha256`、`provenance.json`。
- `build-plan.sha256` 内容 = build-plan.json 全字节的 SHA-256，64 位小写十六进制 + 恰一个 LF，无文件名、无其他字节。
- `build_plan_digest` = 上述同一摘要值，登记进 ProvenanceRecord 与下游摘要链（Source → BuildPlan → ArtifactIndex，ADR 0001）。
- `inputs_digest` = 对「省略 inputs_digest 字段后的 BuildPlan 规范编码字节」的 SHA-256（自排除，避免自引用）；使无 sidecar 也能检出计划内容篡改。
- `provenance.json` 用同一确定性 JSON 规则编码，含 `build_plan_digest` 与 `build_recipe_digest`；`build_recipe_digest` 的投影输入集是 composition 私有细节，但必须满足同一确定性规则。V1 不给 provenance 单独 sidecar（完整性由 `build_plan_digest` 引用与目录级原子发布锚定）。
- 一切消费者（root-abi generator、platform-build、manifest、evidence-generator）必须先验 sidecar 再消费计划（规格 §10.4）；验证与使用针对同一份已验证字节，不得验证后再换路径重读。

### 7. 冻结协议：temp / fsync / rename，不可覆盖

发布单元是**整个计划目录**（三文件全有或全无），P0 Linux 流程：

1. 编码、摘要全部在内存完成后才开始写盘。
2. 在目标目录的父目录（同文件系统）创建一次性随机名临时目录 `.build-plan.tmp-<nonce>/`，三文件写入其中。
3. 每个文件 write → flush → fsync(文件) → close；随后 fsync(临时目录)。
4. 以 no-replace 语义把临时目录 rename 到最终计划目录路径（P0 Linux 用 `renameat2(RENAME_NOREPLACE)`；syscall 不可用即报 `AtomicPublishFailed`，不做 check-then-rename 竞态降级）；目标已存在即 `OutputAlreadyExists`（CLI 退出码 4），**任何情况下不覆盖已发布计划**。
5. rename 成功后 fsync(父目录)。
6. 任何一步失败：删除临时目录及其全部内容，不留下可发现的临时物；绝不出现部分发布的 `build-plan.json`（规格 §7.4）。
7. 重复 compose 到已存在计划目录 = 稳定拒绝；可复现性比对走「重复执行到不同空目录并 cmp」（LCE-P0-004 验收）。

### 8. platform 不得修改 BuildPlan（显式条款）

- BuildPlan 是 composition 单写的冻结产物（ADR 0001）；platform 与所有消费者**只读**：
  - 只能经 `composition::verify_frozen_plan`（或等价只读 API）取得 `FrozenBuildPlan`；不存在接受可变 `BuildPlan` 的执行 API（规格 §7.5 完成条件的落地）。
  - 不得对 `build-plan.json` / `build-plan.sha256` / `provenance.json` 做写、改名、删除、重新冻结或原地补写（包括「补平台特化参数」）；任何此类操作使 sidecar digest 立即失配并被稳定拒绝。
  - 缺平台特化参数的唯一合法路径是回到 compose 重新生成（规格 §7.4「platform 专属参数缺失时重新 compose；platform 不得补写」）。
  - platform 执行期 argv/env 必须完全来自已验证计划（LCE-P0-008），不得把执行期解析结果回写计划。
  - 篡改负向测试（改 build-plan.json 任一字节后 build-staging）必须稳定失败——LCE-P0-008 `plan_immutability` 以本条为判据。

### 9. 维护 owner

- 格式、编码、白名单、冻结协议、版本迁移的代码唯一落点：`lumio-core-composition`（model.rs / encode.rs / freeze.rs / provenance.rs，规格 §7.2）；解析与验证唯一入口是 composition 公开 API，消费者不得自建第二套解析器（ADR 0004 已冻结 platform-build / root-abi-generator / manifest / evidence-generator → composition 依赖边，本 ADR 复用该边、不加新边）。
- 文档唯一落点是本 ADR；规则变化以新 ADR 取代，不改写。
- 维护角色：程序·协议/公共。

### 10. 迁移与退出路径

- **必须升 `plan_format_version` 的变化**：字段集增删；键序/转义/数字等编码规则变化；`GitObjectId` 从 40 位小写 SHA-1 迁到新 git object format（规格 §7.3 已预告此路必须经本 ADR 迁移版本）；白名单删键/收紧值域；`WorkspaceRelativePath` 不变量收紧——一句话：任何使既有 v1 计划读回失败或语义改变的变更。
- **不升版本的加法**：白名单新增键、追加不影响既有字节的语义校验。
- **升级流程**：新 ADR 定 vN 规则 → 仓内单次原子切换（编码器/解码器/测试/Golden 同一提交更新）→ 旧 v1 计划一律作废并重新 compose。BuildPlan 是确定性派生物，**迁移 = 重新生成**：不提供跨版本迁移工具，不做双版本兼容期。
- **格式整体更换**（如迁 CBOR）：新 ADR 取代本 ADR；`inputs_digest`/`build_plan_digest` 摘要链与消费者 API 面不变。
- 「退出冻结」不存在：不可变性由 ADR 0001 冻结，本 ADR 只能被更具体的协议取代，不能被「可变计划」取代。

### 非目标

不把 BuildPlan 注册进架构源 Schema/Fixture；不影响 CoreEngineManifestBody、SignatureEnvelope、PackageIdentity 与公共 ErrorCode；不实现 compose/freeze 本身（LCE-P0-004）；不改变 ADR 0001—0004 的任何边界；不引入 serde/serde_json 之外的任何新依赖决策。

## 后果

- LCE-P0-004（encode/freeze 实现）与 LCE-P0-008（plan 消费与篡改拒绝）有了唯一可引用协议：验收项「composition/platform 测试可引用唯一 plan_format_version=1」由第 2、3 条成立，「ADR 明确 platform 不得修改 BuildPlan」由第 8 条成立。
- 参数变化的代价：任何平台特化参数缺失都要回 compose 重生成（ADR 0001 已接受此流转成本；本 ADR 把它落成字节级判据）。
- 版本演进都要走新 ADR + 仓内原子切换，流程成本前置；换来摘要链可机器验证、无静默覆盖。
- 计划体积比 CBOR 大（KB 级，可忽略）；消费者对 composition 解析器形成硬依赖（ADR 0004 依赖边既有，无新增）。
- 本 ADR 生效即约束在途卡：LCE-P0-004/LCE-P0-008 的实现与测试不得偏离第 2—8 条；偏离 = 退回。
