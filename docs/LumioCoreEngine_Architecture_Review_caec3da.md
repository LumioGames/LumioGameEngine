# LumioCoreEngine 模块化架构对抗式 Review

> **证据范围说明**：本次评审固定在远端提交 `caec3da2c485bc96180497b8c3fc95346732f9f5`，即 `docs(core): add modular module readmes`。当前环境不能读取 `/Users/cui/LumioGames/LumioCoreEngine` 的本地工作树，因此不对本地未提交修改或本地分支同步状态作断言。该提交对应的 `Repository Policy` 工作流已成功执行，其工作流包含 `spec-lint`、测试、内容规则和架构基线 Hash 检查；这证明政策检查通过，但不等于架构契约已经完整。([GitHub commit](https://github.com/LumioGames/LumioCoreEngine/commit/caec3da2c485bc96180497b8c3fc95346732f9f5))

---

## 1. Executive Verdict

## **基本合理，但需先修正；当前不应进入可被其他仓库消费的生产实现**

可以继续进行公共 Schema、Fixture、ADR 和本仓文档收敛，但不建议立即实现正式 Root ABI、生产 Loader 或签名链路。

最关键的三个原因：

1. **ABI、CoreEngineManifest、签名载荷和 Loader Package Identity 尚未形成可执行的公共契约。**  
   当前公共 Schema 登记中只有一个粒度较粗的 Native/Managed ABI Schema，没有 CoreEngineManifest、包索引、签名 Envelope 或 Loader 状态契约；这会迫使实现者在本仓自行创造公共语义，违反唯一架构源原则。([Public schema registry](https://raw.githubusercontent.com/LumioGames/LumioGameEngineArchitecture/main/schemas/index.json))

2. **当前模块图混合了源码依赖、产物流、验证流和诊断事件流。**  
   公共基线明确规定 `A -> B` 表示 A 消费或依赖 B，而本仓模块图主要按“生产者到消费者”绘制。按照公共语义解读，甚至会得到“生产模块依赖 smoke”的错误结论。([modules/README.md](https://github.com/LumioGames/LumioCoreEngine/raw/refs/heads/main/modules/README.md))

3. **`smoke`、`diagnostics` 和 `signing` 的执行平面与所有权尚未收敛。**  
   公共架构第 16 章列出的 CoreEngine 首批模块只有六个，本仓新增的 smoke 和 diagnostics 尚未回写公共模块地图；同时 diagnostics 已进入队列、应急落盘和阻塞入口策略，越过了 CoreEngine Adapter 边界。([Architecture baseline](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/docs/architecture/LumioGameEngine_Architecture_v1.0.md))

### 总体边界判断

当前 README **没有明显把 VoxelWorld、Gameplay、Session、Server World 或 Client World 状态倒灌进 CoreEngine**。CoreEngine 仍被描述为 Native 组合、ABI、包、加载、校验和供应链层，这一总体方向正确。([README.md](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/README.md))

问题不在于“仓库领域方向错误”，而在于：

- 公共契约尚不足以进入 ABI 和供应链实现；
- 模块之间的生产阶段、运行阶段和验证阶段尚未明确分层；
- 若现在实现，极易产生一套只能在本仓自洽、却无法被 Runtime、Server、Client 稳定消费的事实标准。

---

# 2. Findings

## P0 Findings

### [CE-001] [P0] CoreEngineManifest、Artifact Hash 与签名载荷存在循环和未绑定证据风险

**文件：**

- `README.md`
- `modules/manifest/README.md`
- `modules/signing/README.md`
- 公共架构 Schema Registry

**行号或章节：**

- `README.md:96-105`
- `modules/manifest/README.md:6-21`
- `modules/signing/README.md:6-23`
- 公共架构 Schema 索引

**证据：**

根 README 将 `Signature`、`SBOM`、`License` 列为 CoreEngineManifest 的最低字段；但模块产物链同时规定 manifest 先生成并交给 signing，而 signing 又负责生成签名、SBOM 和 License 证据。manifest README 只说明非确定时间字段不进入 Artifact Hash，没有定义它们是否进入 Manifest Hash、签名载荷或独立 Attestation。公共 Schema Registry 中也没有登记 CoreEngineManifest、ArtifactIndex 或 SignatureEnvelope Schema。([README.md](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/README.md))

**影响：**

可能出现以下不可接受情况：

- Manifest 包含 Signature，而 Signature 又签 Manifest，形成自引用；
- SBOM 或 License 在签名后被替换，但签名仍然通过；
- Artifact Hash、Manifest Hash 和 Package Hash 指向不同字节域；
- Loader 验证的是构建阶段产生的旧 verification result，而非当前实际打开的包；
- 时间戳、签名证书链、透明日志证明破坏可复现性；
- 同一个包在不同机器上得到不同 Manifest Hash。

**建议：**

在公共架构源中至少拆出以下正式契约：

1. `CoreEngineManifestBody`：确定性、可复现、无自引用；
2. `ArtifactIndex`：每个文件的规范路径、类型、大小和 Digest；
3. `EvidenceSet`：SBOM、License、Provenance 的 Digest 和媒体类型；
4. `SignatureEnvelope`：与 ManifestBody 分离；
5. `VerifiedPackageDescriptor`：运行时验证器输出，Loader 只能消费这一结果。

必须明确：

- Canonical Serialization 算法及版本；
- Unicode、数字、路径分隔符和字段顺序规则；
- Hash Algorithm 和 Domain Separation；
- 签名覆盖的精确字节；
- Evidence 是否直接进入 ManifestBody，或通过 Digest 绑定；
- 未知字段、可选字段和必需字段策略；
- 时间戳、证书链和透明日志证明位于可复现 Body 之外。

**是否需要更新公共架构源：** 是，属于跨仓包格式和供应链安全契约。

**是否需要新增 ADR：** 是。先新增公共 ADR；之后本仓可再新增签名 Provider、Canonicalization Library 和原子文件读取方式的实现 ADR。

---

### [CE-002] [P0] Root ABI 的唯一公共 Schema 不足以生成安全、稳定的 C ABI

**文件：**

- `modules/root-abi/README.md`
- 公共 `native-managed-abi` Schema
- `README.md`

**行号或章节：**

- `modules/root-abi/README.md:6-25`
- `README.md:48-56`
- 公共 ABI Schema 全文

**证据：**

root-abi README 要求 Header、Managed Adapter 和 P/Invoke 全部由唯一 Schema 生成，并声明稳定 Error Code、Opaque Handle、API Table 和 Capability。但公共 ABI Schema 当前主要描述 API Table 名称、版本和函数数量，以及少量概括性的 Ownership、Threading、Panic、Exception 和 Load Policy 字段；它没有定义足以生成二进制布局的函数 Slot、签名、Calling Convention、Alignment、Buffer Contract 或 Handle 生命周期。部分关键语义还是可选字段。([root-abi README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/root-abi/README.md))

**影响：**

第一批实现者只能二选一：

- 在 CoreEngine 本仓补写实际 ABI 语义，形成第二套事实来源；
- 根据个人理解手写 Rust、C Header 和 C# P/Invoke 布局。

两者都会破坏“唯一 Root ABI”和“生成物不可手改”约束，并可能产生：

- `stdcall`、`cdecl`、AOT 调用约定不一致；
- 32/64 位结构大小和对齐差异；
- Rust panic 或 C# exception 穿越 ABI；
- Buffer 或错误详情的释放方不一致；
- Handle 重复释放、悬空句柄和跨线程误用；
- API Table 尾部扩展破坏旧 Host。

**建议：**

公共 ABI Schema 至少必须冻结：

- 唯一入口符号、符号前缀和可见性；
- Calling Convention；
- API Table Header、Slot 顺序和 Slot Stable ID；
- Major/Minor 兼容规则、`struct_size` 和尾部扩展规则；
- Alignment、Packing、Endian、Pointer Width；
- 每个函数的参数、返回值、Nullability；
- Buffer 的 `ptr/len/capacity`、分配器和释放函数；
- 每类 Handle 的创建方、销毁方、线程归属和 Generation 校验；
- ErrorCode Registry、错误详情编码及其生命周期；
- Panic/Exception 永不穿越 ABI；
- Reentrancy、Owner Thread、Worker Callback 禁止规则；
- Cancellation Token 和 Destroy/Cancel 并发规则；
- Capability Namespace、未知 Capability 的处理方式；
- Reserved Slot 必须为零等前向兼容规则。

**是否需要更新公共架构源：** 是。不能由本仓 README 自行冻结。

**是否需要新增 ADR：** 是，公共 ABI ADR；本仓仅可为生成器工具和输出目录新增内部 ADR。

---

### [CE-003] [P0] Loader 的“单进程单组合”、验证责任和卸载语义尚不安全

**文件：**

- `README.md`
- `modules/loader/README.md`
- `modules/signing/README.md`
- 公共架构第 8 章

**行号或章节：**

- `README.md:48-56`
- `modules/loader/README.md:6-25`
- `modules/signing/README.md:17-23`
- 公共架构 Native Loader 约束

**证据：**

Loader 一方面被描述为执行 Signature Preflight，另一方面又把 `signing verification result` 作为输入；没有规定运行时必须重新验证包，还是可以信任构建阶段结果。根 README 表述为拒绝第二个“**不兼容**”包，而公共约束是一个进程只能有一套 Native 组合。Loader 状态机仍标记为“建议状态”，并包含 Unload，但没有 Lease、Quiescence、活跃 API Table 指针或 Native Worker 停止协议。([loader README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/loader/README.md))

**影响：**

- 第二个版本号相同但内容不同的包可能被当成“兼容”而加载；
- 不同路径指向同一文件或同一路径被原地替换时身份判断不可靠；
- 验证和 `dlopen/LoadLibrary` 之间存在 TOCTOU；
- 卸载后 Runtime 仍持有 API Table 或 Opaque Handle，产生 UAF；
- 并发 Acquire 可能重复映射 Library 或创建两个 Worker Pool；
- 部分加载失败后残留符号、文件句柄或全局 Registry 状态；
- 静态链接的 iOS/移动端无法套用动态加载状态机。

**建议：**

冻结以下公共语义：

- 进程级 `PackageIdentity`，至少包含 Manifest Digest、Artifact Set Digest、ABI Identity、Target Profile；
- 第一次成功 Acquire 后锁定唯一 PackageIdentity；
- 同一身份重复 Acquire 为幂等 Lease/引用计数；
- 任意不同身份一律返回稳定 `PackageConflict`，不判断“看起来兼容”；
- Loader 必须对实际打开的同一组文件句柄完成 Hash 和 Signature 验证；
- 包内 Trust Root 不得成为自身信任来源；
- 动态和静态加载使用两个 Backend，但共享同一逻辑状态机；
- V1 优先考虑“进程生命周期内不物理卸载”，避免跨语言指针 UAF；
- 若支持物理卸载，必须有 Host Quiescence、Handle Drain 和 Worker Join；
- 定义并发 Acquire、Timeout、Cancel、OOM、Partial Mapping 和 Retry 行为；
- Loader 输出应是 `LoaderLease + RootApiTableView`，不是裸 Library Handle。

**是否需要更新公共架构源：** 是，属于跨仓可观察的运行时契约。

**是否需要新增 ADR：** 是。公共 ADR 冻结状态和行为；本仓 ADR 选择 OS Loader Backend、Registry 实现和是否采用 V1 No-Unload。

---

## P1 Findings

### [CE-004] [P1] 当前模块图不是一张语义明确的依赖 DAG

**文件：**

- `modules/README.md`
- 公共架构第 2 章

**行号或章节：**

- `modules/README.md:21-40`
- 公共架构依赖图语义定义

**证据：**

公共基线规定 `A -> B` 表示 A 消费、编译依赖或读取 B 的公开 Schema/Artifact。本仓图则使用相同箭头表达从 composition 到 root-abi、再到 platform、manifest、signing、loader 的生产流程，并写出“所有生产模块 -> smoke”。这同时混入了源码依赖、构建产物流、验证消费和诊断事件输出四种不同关系。([modules/README.md](https://github.com/LumioGames/LumioCoreEngine/raw/refs/heads/main/modules/README.md))

**影响：**

- 可能把 smoke 建成生产 Library 依赖；
- Loader 可能编译依赖 Manifest Generator 或 Signing Tool，而不是只消费 Schema 和运行时 Verifier；
- 构建工具链与运行时依赖无法裁剪；
- 图面虽看似无环，但无法证明真实源码依赖无环；
- diagnostics 的箭头可能被误解为 Loader 依赖诊断队列，或 diagnostics 依赖 Loader 内部实现。

**建议：**

至少拆成四张图，并在每张图旁定义箭头：

1. Source/Compile Dependency；
2. Build Artifact Flow；
3. Runtime Call/Ownership；
4. Validation 与 Observability Event Flow。

**是否需要更新公共架构源：** 本仓图可先本地修正；若修改公共仓库拓扑或公共依赖方向，则必须同步公共架构源。

**是否需要新增 ADR：** 否，除非要改变公共依赖方向。

---

### [CE-005] [P1] P0/P1 优先级不能形成一条可执行的 Vertical Slice

**文件：**

- `README.md`
- `modules/README.md`
- 各模块 README

**行号或章节：**

- `README.md:21-32`
- `modules/README.md:8-18`
- `modules/loader/README.md:17-18`
- `modules/manifest/README.md:17-18`

**证据：**

当前标记为 P0 的 Loader 需要签名验证结果，但 signing 为 P1；P0 manifest 需要 platform 产物，而 platform 为 P1；P0 smoke 又要求验证 platform、signing、loader 等全部产物。由此，所谓 P0 不能独立达到自己的验收条件。([README.md](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/README.md))

**影响：**

实现阶段很可能出现以下临时绕过：

- P0 Loader 暂时跳过签名；
- 使用未登记的 Development Unsigned Package；
- platform 逻辑临时塞进 composition；
- smoke 只能验证 Happy Path，无法覆盖真实包；
- P0 完成状态与真实可运行状态不一致。

**建议：**

将“公共契约优先级”和“完整自动化优先级”分开：

- **P0 Contract/Minimal Slice**：一个明确 Target Profile、SignatureEnvelope Schema、测试 Verifier、Loader State Machine、基础 Diagnostic Event Schema；
- **P1 Expansion**：完整平台矩阵、生产 Key Rotation、远程签名、完整 SBOM/License 自动化、诊断批处理；
- smoke 不应作为产品模块参与 P0/P1 产品依赖，应作为每个阶段的验证门。

**是否需要更新公共架构源：** 若优先级属于公共第 16 章实现节奏，需更新；纯仓内实施排期可在本仓调整。

**是否需要新增 ADR：** 否，优先级和里程碑应进入实现计划；Unsigned Development Profile 若被允许则必须有 ADR。

---

### [CE-006] [P1] composition 与 platform 同时拥有平台矩阵和构建产物

**文件：**

- `modules/composition/README.md`
- `modules/platform/README.md`

**行号或章节：**

- `modules/composition/README.md:6-26`
- `modules/platform/README.md:6-26`

**证据：**

composition 声称负责平台矩阵、Toolchain、构建参数和“平台构建产物”；platform 同时负责 Target、SDK、Compiler、链接方式、目录布局，并输出平台包。两个模块都可以被理解为实际构建执行者和平台矩阵所有者。([composition README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/composition/README.md))

**影响：**

- Compiler Flag、Feature 和链接参数可能有两个来源；
- Manifest 无法判断 Build Recipe 来自 composition 还是 platform；
- 同一 Target 可能产生不同目录布局和 Artifact Hash；
- CI 与本地构建可能调用两套入口；
- 模块内部实现无法独立替换。

**建议：**

重新冻结边界：

- `composition`：只负责 Source Lock、Feature Resolution、Build Recipe、输入 Hash 和 Provenance Plan；
- `platform`：只负责 Target Profile 规范化、平台 Backend、实际构建/链接、产物布局和 ArtifactIndex；
- 实际编译只能有一个权威执行入口；
- platform 消费 composition 产生的不可变 BuildPlan，不反向修改它。

不建议将两者合并，因为“组合计划”和“平台执行”具有不同替换边界，但必须消除双重所有权。

**是否需要更新公共架构源：** 若 BuildPlan、Target Profile 或 Manifest 字段跨仓消费，需要更新公共 Schema；纯工具实现边界可本仓处理。

**是否需要新增 ADR：** 是，本仓 Build Orchestration ADR。

---

### [CE-007] [P1] signing 顶层模块混合了四种不同安全边界

**文件：**

- `modules/signing/README.md`
- 公共架构 RACI
- 公共 Fixture 规则

**行号或章节：**

- `modules/signing/README.md:6-26`
- 公共架构 CoreEngine 所有权章节
- 公共 Fixture README 的签名说明

**证据：**

当前 signing 同时负责：

1. 包签名；
2. 运行时签名验证；
3. Trust Root 与 Key Rotation 元数据；
4. SBOM 和 License 生成。

这些职责分别属于离线/CI 私钥域、运行时只读验证域、信任策略域和供应链证据生成域。公共架构将签名归入 CoreEngine，但 Fixture 说明又使用“Release Toolchain implementation”措辞，没有明确该 Toolchain 是 CoreEngine 内部工具还是独立发布工具。([signing README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/signing/README.md))

**影响：**

- Loader 运行时包可能误链接 Signer 或私钥 Provider；
- 生产私钥可能被误认为 CoreEngine Runtime 配置；
- Verifier 与 Signer 被迫共同升级；
- SBOM 失败与签名失败混用同一状态机；
- Key Rotation 政策与具体签名 SDK 紧耦合。

**建议：**

顶层可改名为 `supply-chain` 或保留 signing 聚合名，但内部必须拆成：

- `evidence-generator`
- `signer-tool`
- `runtime-verifier`
- `trust-policy`
- `attestation-model`

运行时发布包只能包含 verifier 和只读 trust metadata，不应包含 signer 或私钥访问代码。

**是否需要更新公共架构源：** 是，需要明确 Signing、Release Toolchain 和 Runtime Verifier 的 RACI。

**是否需要新增 ADR：** 是，本仓需记录内部拆分和部署边界；签名算法及信任模型属于公共 ADR。

---

### [CE-008] [P1] smoke 和 diagnostics 与公共模块地图不一致

**文件：**

- `modules/README.md`
- `modules/smoke/README.md`
- `modules/diagnostics/README.md`
- 公共架构第 16 章

**行号或章节：**

- `modules/README.md:8-18`
- 公共架构 CoreEngine 首批模块地图

**证据：**

公共架构第 16 章列出的 CoreEngine 首批模块为 composition、root-abi、loader、manifest、signing、platform。本仓将 smoke 定义为 P0 支撑模块、diagnostics 定义为 P1 模块，但没有对应公共架构变更记录。([modules/README.md](https://github.com/LumioGames/LumioCoreEngine/raw/refs/heads/main/modules/README.md))

**影响：**

- 公共架构源和实现仓的模块地图出现漂移；
- smoke 可能被视为生产构件；
- diagnostics 的公共事件职责可能与 Server Observability 重叠；
- 后续其他仓库无法判断这两个目录是否可被引用。

**建议：**

- `smoke` 改为 `tests/validation`、`validation-plane` 或明确的非生产支撑目录；
- `diagnostics` 改为 Diagnostic Adapter/Contract 支撑平面；
- 若坚持把二者作为公共一级模块，必须先更新公共模块地图、RACI 和依赖图。

**是否需要更新公共架构源：** 保留为一级公共模块时必须更新。

**是否需要新增 ADR：** 若仅调整本仓目录类型，不必；若定义新的公共执行平面，需要公共 ADR。

---

### [CE-009] [P1] diagnostics 越过 Adapter 边界，开始拥有 Host 队列和持久化策略

**文件：**

- `modules/diagnostics/README.md`
- `README.md`
- 公共架构第 12 章
- 公共 Logging Event Schema

**行号或章节：**

- `modules/diagnostics/README.md:6-26`
- `README.md:57-66`
- 公共架构日志、Metrics、Trace 与审计章节

**证据：**

diagnostics 声称拥有有界异步队列、批处理、队列满载策略、应急落盘，并可以阻塞新入口；同时又声明不拥有最终 Sink 和 Audit 规则。公共架构将 Diagnostic Log、Audit Log、Txn Journal 和 Command Log 分开，公共 logging-event Schema 的 Owner 也不是 CoreEngine。([diagnostics README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/diagnostics/README.md))

**影响：**

- Loader 可能因诊断队列满载而死锁；
- CoreEngine 无法在不拥有 Host 生命周期的情况下安全阻塞“新入口”；
- Server 和 CoreEngine 可能各自维护一套异步队列；
- 签名/SBOM 验证证据可能被当成普通 Diagnostic Event 而丢失；
- Audit 持久化责任变得不清楚。

**建议：**

CoreEngine diagnostics 应只提供：

- 稳定的 Diagnostic Event Contract；
- 同步返回的 `VerificationResult`；
- Host 注入的 EventSink Adapter；
- Failure Evidence Fragment；
- 不依赖具体 Sink 的 Metrics/Trace 属性。

队列、批处理、采样、磁盘 Spool、跨进程传输和 Durable Audit 应由 Host/Server Observability 实现。供应链准入结果必须是 Loader 返回值和审计输入，不能仅靠异步日志保证。

**是否需要更新公共架构源：** 是，需确认 Logging Event、Audit Evidence 和 Failure Bundle 的 RACI。

**是否需要新增 ADR：** 本仓可为 Adapter 行为新增 ADR；全局队列和持久化策略不应在本仓 ADR 中决定。

---

### [CE-010] [P1] 平台分类、链接 Backend 和 Host Profile 尚未正交化

**文件：**

- `modules/platform/README.md`
- `README.md`
- 公共架构第 10 章

**行号或章节：**

- `modules/platform/README.md:6-26`
- `README.md:77-94`
- 公共架构 Host Profile、Platform 与 Capability 章节

**证据：**

当前列表混合了 Linux、Windows Server、Desktop、iOS、Android：有的是 OS，有的是产品角色，有的是设备类别。没有冻结 Target Triple、CPU Architecture、libc/C Runtime、Minimum OS、动态/静态链接 Backend 或移动端部署限制。根 README 又把 PureHeadless 列入统一 Loader 覆盖范围，但公共架构中 PureHeadless 是 ReferenceVoxelPort 路径，可以完全不加载 Native 包。([platform README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/platform/README.md))

**影响：**

- “Windows Server”和“Desktop Windows”可能被错误建成两个 ABI；
- Linux glibc 与 musl 被错误视为同一 Target；
- iOS 静态链接无法执行动态 Loader Preflight；
- Android ABI Split 和 NDK 版本无法进入 Manifest；
- Host Profile、Capability 与 Platform 被重复编码；
- PureHeadless 被迫依赖 CoreEngine Loader。

**建议：**

将以下维度分开：

- `TargetProfile`：OS、Arch、ABI、libc、Minimum OS、SDK、Compiler；
- `PackagingProfile`：文件名、目录布局、Debug Symbol、Archive Format；
- `LoadBackend`：StaticLinked、DynamicLibrary、ProcessAbsent；
- `HostProfile`：NativeHeadless、LocalEmbedded、LocalSplitProcess、RemoteDS、MobileLocal、PureHeadless；
- `CapabilitySet`：运行时功能，不等同于平台。

PureHeadless 应明确为 CoreEngine 缺席或 No-Native 路径，而不是 Loader 的一种普通包加载模式。

**是否需要更新公共架构源：** 是，Target/Host/Capability 都会被多个仓库消费。

**是否需要新增 ADR：** 平台字段和兼容策略需公共 ADR；目录实现和 SDK 探测可由本仓 ADR 决定。

---

### [CE-011] [P1] 公共 Error、Capability 和 Failure Fixture 登记不足以支撑 README 承诺

**文件：**

- 公共 Schema Index
- 公共 Fixture Index
- 公共 ID Registry
- `modules/loader/README.md`
- `modules/smoke/README.md`

**行号或章节：**

- 公共 Registry 全文
- `modules/loader/README.md:21-25`
- `modules/smoke/README.md:6-25`

**证据：**

当前公共 ID 登记中，与 CoreEngine 直接相关的稳定错误主要只有 `NativeAbiMismatch`；Fixture 主要覆盖 ABI 正例、Pointer Width 反例和通用 Failure Bundle Hash。README 中要求的 Hash Mismatch、Signature Failure、Symbol Collision、Duplicate Library、Repeated Release、OOM 和 Loader Timeout 尚无对应公共错误登记和正/负 Fixture。([fixture registry](https://raw.githubusercontent.com/LumioGames/LumioGameEngineArchitecture/main/fixtures/index.json))

**影响：**

- 每个消费者可能自行映射错误码；
- Loader 无法稳定区分“不兼容”“损坏”“不可信”和“资源不足”；
- smoke 只能依赖字符串消息断言；
- Failure Bundle 无法按稳定错误分类；
- 上层重试策略可能把永久错误当成瞬态错误。

**建议：**

在公共源登记最少以下错误族：

- `ManifestMalformed`
- `ManifestUnsupportedVersion`
- `ArtifactMissing`
- `ArtifactHashMismatch`
- `SignatureMissing`
- `SignatureInvalid`
- `TrustRootUnknown`
- `TrustPolicyRejected`
- `TargetMismatch`
- `AbiMajorMismatch`
- `CapabilityMissing`
- `SymbolMissing`
- `SymbolCollision`
- `PackageAlreadyLatched`
- `PackageIdentityConflict`
- `LoaderTimeout`
- `LoaderOutOfMemory`
- `PartialLoadRolledBack`
- `InvalidHandle`
- `DuplicateRelease`

每个 P0 错误至少一组正例和一组负例 Fixture。

**是否需要更新公共架构源：** 是。

**是否需要新增 ADR：** Error 分类及稳定性需要公共 ADR；本仓无需再定义第二套编号。

---

## P2 Findings

### [CE-012] [P2] 生命周期章节存在，但不足以指导实现

**文件：**

- 所有模块 README
- 尤其 `modules/loader/README.md`
- `modules/smoke/README.md`
- `modules/diagnostics/README.md`

**行号或章节：**

- 各模块“生命周期与失败行为”
- `modules/loader/README.md:21`
- `modules/smoke/README.md` 全文

**证据：**

多数模块只给出一行阶段序列和若干失败名称，没有冻结 Owner Thread、并发调用、Timeout 起点、OOM 行为、Partial Rollback、Safe Retry、Idempotency 和残留资源检查。Loader 状态仍是“建议状态”。smoke README 没有与其他模块一致的“生命周期与失败行为”章节。([composition README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/composition/README.md))

**影响：**

当前 README 足以解释“模块是做什么的”，但不足以指导两个独立团队实现出行为一致的组件。

**建议：**

每个模块增加规范化状态表：

| 字段 | 必须描述 |
|---|---|
| Owner | 谁创建、谁销毁 |
| Entry Thread | 允许在哪个线程调用 |
| State | 稳定状态与瞬态状态 |
| Timeout | 从何时计时、超时后资源状态 |
| OOM | 返回码、诊断、是否可重试 |
| Partial Failure | 已产生文件、句柄或 Registry 如何回滚 |
| Retry | 同输入是否幂等 |
| Recovery | 是否可恢复、重放或只能重新构建 |
| Diagnostic Fields | PackageId、ArtifactHash、TraceId、State、ErrorCode |
| Acceptance | 状态和资源不变量 |

**是否需要更新公共架构源：** 对外可观察的 Loader、ABI、包验证行为需要；内部构建步骤可留在本仓。

**是否需要新增 ADR：** Loader、Signer/Verifier 和 Platform Backend 建议新增。

---

### [CE-013] [P2] Source Commit、Input Hash、Output Hash 尚未贯穿整个产物链

**文件：**

- `modules/composition/README.md`
- `modules/root-abi/README.md`
- `modules/platform/README.md`
- `modules/manifest/README.md`
- `modules/signing/README.md`

**行号或章节：**

- 各模块输入/输出章节

**证据：**

composition 明确锁定 Source Commit 和构建参数，root-abi 声明 Compiler/Input/Output Hash，manifest 声明 Artifact Hash；但 platform 输出和 signing 输出没有统一要求携带上一阶段 Input Hash、当前 Output Hash 和工具版本。也没有规范化的逐文件 ArtifactIndex。([composition README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/composition/README.md))

**影响：**

即使最终二进制 Hash 正确，也无法回答：

- 它来自哪一个 Build Recipe；
- ABI Header 使用了哪个 Schema 和 Generator；
- Debug Symbols 是否匹配；
- SBOM 是否对应同一个 Artifact Set；
- Compiler、Linker、SDK 或 Feature 是否在中途变化。

**建议：**

冻结如下 Digest Chain：

```text
SourceTreeDigest
  -> BuildRecipeDigest
  -> AbiSchemaDigest + GeneratorDigest
  -> TargetProfileDigest
  -> PlatformArtifactSetDigest
  -> EvidenceSetDigest
  -> CanonicalManifestDigest
  -> SignatureEnvelope
```

每一阶段输出必须记录前一阶段的输入 Digest，不能只在最终 Manifest 中事后拼接来源描述。

**是否需要更新公共架构源：** 是，Digest 字段和链路属于包格式。

**是否需要新增 ADR：** Hash 算法和 Canonicalization 属于公共 ADR；本仓可为缓存键和本地目录新增 ADR。

---

### [CE-014] [P2] Failure Bundle 同时被 smoke 和 diagnostics 声称生产，公共 Schema 又偏向游戏快照故障

**文件：**

- `modules/smoke/README.md`
- `modules/diagnostics/README.md`
- 公共 Failure Bundle Schema
- 公共 Schema Index

**行号或章节：**

- `modules/smoke/README.md:18-23`
- `modules/diagnostics/README.md:17-22`
- 公共 Failure Bundle Schema

**证据：**

smoke 输出 Failure Bundle，diagnostics 又负责组装 Failure Bundle；公共 Schema 将 Failure Bundle 所有权放在 Architecture，同时现有 Schema 要求 `snapshotId`。发生 Manifest Parse、Signature Failure 或 `dlopen` 前失败时，通常不存在游戏 Snapshot。([smoke README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/smoke/README.md))

**影响：**

- 两个模块可能生成不同 Bundle 格式；
- CoreEngine 预加载故障无法合法填写 `snapshotId`；
- smoke 测试报告和生产事故 Bundle 混在一起；
- Host 无法统一聚合 Server、Runtime 和 CoreEngine 的现场证据。

**建议：**

- 公共架构继续拥有 Failure Bundle Schema；
- CoreEngine 模块只输出 `FailureEvidenceFragment`；
- smoke 输出测试报告和 Fixture Reference，不直接定义生产 Bundle；
- Host 或独立诊断工具负责最终 Bundle Assembly；
- Schema 增加 `incidentKind`，并使 Snapshot 仅对相关事故必需；
- CoreEngine Fragment 至少包含 PackageIdentity、ManifestDigest、ArtifactDigest、TrustDecision、LoaderState、TargetProfile、TraceId 和稳定 ErrorCode。

**是否需要更新公共架构源：** 是。

**是否需要新增 ADR：** Bundle Assembly 的公共所有权需要公共 ADR；本仓无需另立格式。

---

### [CE-015] [P2] Manifest 与上层 ReleaseManifest 的文字边界清楚，但缺少机器可验证的关联

**文件：**

- `README.md`
- `modules/manifest/README.md`
- 公共 ReleaseManifest Schema
- 公共架构第 13 章

**行号或章节：**

- `README.md:77-94`
- `modules/manifest/README.md:12-13`
- 公共 Release 章节和 Schema

**证据：**

本仓明确排除了产品级 Release 路由和业务 Migration，这一职责边界正确；但公共 ReleaseManifest 目前只有较粗的 CoreEngine ABI/Signature 信息，没有冻结它通过哪个 PackageIdentity、ManifestDigest 或 ArtifactSetDigest 精确引用 CoreEngine 包。([README.md](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/README.md))

**影响：**

上层 ReleaseManifest 可能声明 ABI 版本正确，却实际选中另一个同 ABI、不同 Feature 或不同 Artifact Hash 的包。

**建议：**

ReleaseManifest 不复制 CoreEngineManifest 内容，但必须通过以下不可变键精确引用：

- CoreEngine Package Identity；
- CoreEngine Manifest Digest；
- Artifact Set Digest；
- ABI Identity；
- Target Profile；
- Capability Set Digest；
- Signature/Attestation Reference。

**是否需要更新公共架构源：** 是，属于 Game Release 与 CoreEngine Package 的跨仓链接。

**是否需要新增 ADR：** 公共 ADR。

---

## P3 Findings

### [CE-016] [P3] 文档格式和自动校验覆盖存在低风险不一致

**文件：**

- `modules/README.md`
- `modules/smoke/README.md`
- `modules/diagnostics/README.md`
- `.spec/tools/spec-lint.mjs`
- GitHub Workflow

**行号或章节：**

- 模块图和各模块章节
- spec-lint 链接扫描范围

**证据：**

smoke 缺少与其他模块统一的生命周期标题；diagnostics README 将 smoke 事件列为输入，但模块图没有相应关系；root-abi 明确产生 ABI 诊断，但图中 diagnostics 入口主要列 loader/manifest/signing/platform。当前提交的政策工作流通过，但现有 spec-lint 的重点是 `.spec`、根规则和受管文档，并不能作为 `modules/**` 与 `docs/architecture/**` 全量链接和术语一致性证明。([smoke README](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/modules/smoke/README.md))

**影响：**

风险主要是后续文档漂移，不直接造成运行时错误。

**建议：**

- 统一 8 个 README 标题和字段顺序；
- 增加 Module Type：Production、Build Tool、Runtime Adapter、Validation Plane；
- 明确 `Current Fact`、`Normative Requirement`、`Pending Decision`；
- 对 `README.md`、`modules/**/README.md`、`.spec/**/*.md` 和 `docs/architecture/**/*.md` 增加全量相对链接检查；
- 增加术语检查：`Artifact Hash`、`Manifest Hash`、`Package Hash` 不得混用。

**是否需要更新公共架构源：** 否。

**是否需要新增 ADR：** 否。

---

# 3. 模块逐项评估

| 模块 | 单一职责 | 边界清晰度 | 依赖合理性 | 内容完整度 | 结论 |
|---|---|---|---|---|---|
| `composition` | 中 | 中：与 platform 重叠 | 中 | 中低 | **调整职责**。保留为 Source Lock、Feature Resolution、BuildPlan 和 Provenance Planner；不直接拥有平台最终产物 |
| `root-abi` | 高 | 高：没有明显领域状态倒灌 | 中：依赖的公共 Schema 过弱 | 低 | **保留**。不建议再拆一级模块；先补公共 ABI Schema 和生成器契约 |
| `loader` | 高 | 中：Verifier、Trust 和 Unload 边界不清 | 中低 | 低 | **保留并补充文档**。内部可拆 Preflight、Registry、Static/Dynamic Backend；必须冻结状态机 |
| `manifest` | 高 | 中：与 signing 的产物顺序不清 | 中 | 低 | **保留**。不必拆模块，但必须拆数据模型：ManifestBody、ArtifactIndex、SignatureEnvelope |
| `signing` | 中低 | 低：Signer、Verifier、Trust、SBOM、License 混合 | 低 | 低 | **改名并内部分拆**。建议作为 `supply-chain` 聚合，运行时只发布 Verifier |
| `platform` | 中 | 中低：与 composition 重叠，Target 与 Host 混合 | 中 | 低 | **调整职责**。保留 TargetProfile、Build Backend、Layout 和 ArtifactIndex；完整平台矩阵可延后 |
| `smoke` | 高，作为验证平面 | 高 | 低：当前图容易被理解为生产依赖 | 中 | **改为测试/支撑平面**，不是生产模块；放入 `tests/validation` 或明确的 validation plane |
| `diagnostics` | 中 | 低：越过 Adapter 进入 Queue/Sink/Audit 策略 | 中低 | 低 | **改为适配/支撑平面**；只拥有 CoreEngine Event Adapter 和 Failure Evidence Fragment |

### 逐模块补充判断

- 当前八个目录都不是完全没有边界的“伪模块”；每个 README 至少定义了目的、输入、输出和非职责。
- 但 `smoke` 的问题是**类型错误**，不是职责虚假：它是真实验证能力，不应被建模为生产模块。
- `diagnostics` 的问题是**范围过大**：Event Contract 和 Failure Evidence 是 CoreEngine 必需能力，队列、磁盘 Spool 和全局 Audit 则不是。
- `signing` 的问题是**部署安全边界混合**：Signer Tool 与 Runtime Verifier 绝不能因为同属一个目录就被打入同一运行时产物。
- 除 root-abi 外，目前各模块实现都还不能保证“内部替换而不影响消费者”，因为正式输入输出 Schema 尚未冻结。

---

# 4. 依赖和产物链评估

## 4.1 当前 DAG 是否无环

结论分两层：

- **作为作者意图中的产物流，当前图表面无环。**
- **作为公共架构定义的 Source Dependency DAG，当前图语义无效，不能据此证明无环。**

此外，当前 manifest/signing 顺序存在潜在数据自引用：

```text
Manifest 声称包含 Signature/SBOM/License
          |
          v
Signing 又以 Manifest 为输入，并生成 Signature/SBOM/License
```

这不是代码级循环依赖，但属于必须在实现前消除的**产物定义循环**。

## 4.2 建议的 Source/Schema 依赖图

以下统一采用公共基线语义：

> `A -> B` 表示 A 消费 B 的公开 Schema、Artifact 或 Contract。

```text
root-abi
  -> Architecture::NativeManagedAbiSchema
  -> NativeCore/VoxelEngine::PublishedAbiSourceSchema

composition
  -> NativeCore/VoxelEngine::PublishedSourceDescriptor
  -> Architecture::TargetProfileSchema

platform
  -> composition::BuildPlan
  -> root-abi::GeneratedAbiArtifacts

manifest
  -> platform::ArtifactIndex
  -> composition::ProvenanceRecord
  -> root-abi::AbiDescriptor

supply-chain tools
  -> manifest::CanonicalManifestBody
  -> platform::ArtifactIndex
  -> EvidenceSet

loader
  -> manifest::PackageSchema
  -> signing::RuntimeVerifierInterface
  -> root-abi::RootApiTableContract
  -> platform::LoadBackendContract

smoke
  -> 所有公开 Contract 和 Artifact

diagnostic-adapter
  -> Architecture::LoggingEventSchema
  -> Architecture::FailureBundleSchema
```

关键要求：

- 生产模块不依赖 smoke；
- Loader 不依赖 Manifest Generator 或 Signer Tool 的内部实现；
- Loader 只依赖公开 Manifest Schema、Verifier Interface 和 Load Backend；
- diagnostics 不读取 Loader 私有状态，只接收公开事件和状态快照。

## 4.3 建议的 Artifact Flow

下图采用生产者到消费者语义：

```text
Source Lock + Public ABI Schema
        |
        v
BuildPlan + Generated Header/Bindings
        |
        v
PlatformArtifactSet + ArtifactIndex
        |
        v
SBOM / License / Provenance Digests
        |
        v
Canonical CoreEngineManifestBody
        |
        v
Detached SignatureEnvelope
        |
        v
VerifiedPackageDescriptor
        |
        v
LoaderLease + RootApiTableView
```

此顺序避免：

- Signature 自己签自己；
- Manifest 在签名后被补写 SBOM；
- Loader 直接信任离线 verification result；
- Hash 覆盖范围不一致。

## 4.4 产物生产者和消费者

| 产物或契约 | 唯一生产者 | 消费者 | 依赖类型 | 当前状态 |
|---|---|---|---|---|
| Source Lock / BuildPlan | composition | platform、manifest、smoke | Schema/Artifact | 基本存在，但字段未冻结 |
| Root ABI Schema | 公共架构源 | root-abi generator、Native/Voxel Schema Publisher | 公共 Schema | 粒度不足 |
| Generated C Header / C# Binding | root-abi generator | Native Build、Managed Adapter、smoke | Generated Artifact | 原则正确，生成规则不足 |
| Platform Artifact Set | platform | manifest、supply-chain、loader、smoke | Artifact | 与 composition 所有权重叠 |
| ArtifactIndex | platform | manifest、verifier、smoke | Artifact Schema | 当前缺失 |
| ManifestBody / ManifestDigest | manifest | signer/verifier、loader、release tooling | Artifact Schema | 当前无公共 Schema |
| SBOM / License / Provenance | supply-chain evidence generator | manifest/signature、audit、smoke | Evidence Artifact | 与 signing 聚合过紧 |
| SignatureEnvelope | signer tool | runtime verifier、smoke、release tooling | Artifact | 载荷未定义 |
| VerifiedPackageDescriptor | runtime verifier | loader、audit adapter | Runtime Contract | 当前仅笼统称 verification result |
| LoaderLease / API Table View | loader | Runtime Host Adapter | Runtime Contract | Lease 和卸载行为未定义 |
| Diagnostic Event | 各生产模块 | Host Diagnostic Adapter | Event Contract | 所有权需要收窄 |
| Failure Evidence Fragment | 各生产模块 | Host/Tool Bundle Assembler | Artifact Fragment | 当前与 smoke/diagnostics 重叠 |
| Smoke Report | smoke | CI、Release Gate、开发者 | Test Artifact | 不应被生产模块消费 |

## 4.5 隐性反向依赖

当前主要隐性反向依赖风险有：

1. **Loader → signing implementation**  
   Loader 当前消费“签名验证结果”，若不是稳定 Verifier Interface，会依赖离线工具内部格式。

2. **composition ↔ platform**  
   两者都拥有构建和平台矩阵，容易形成 BuildPlan 双向修改。

3. **Production → smoke**  
   当前图的箭头语义可能使生产模块依赖测试模块。

4. **Loader → diagnostics queue**  
   diagnostics 若能够阻塞新入口，Loader 生命周期就会被 Host 日志队列反向控制。

5. **Manifest → signing → Manifest**  
   若 Signature/SBOM 被直接写回 Canonical ManifestBody，会形成产物循环。

---

# 5. 必须补齐的契约

## P0：实现前必须冻结

### 1. Root ABI Schema

最低内容：

- 唯一入口符号；
- Calling Convention；
- API Table Header、Slot 和扩展规则；
- Handle、Buffer、Allocator、Error Detail；
- Thread、Reentrancy、Cancellation；
- Panic/Exception 转换；
- ABI Major/Minor、Capability 和 Error Registry；
- C Header、Rust Adapter、C# Binding 的生成规则。

### 2. CoreEngine Package 与 Manifest Schema

最低内容：

- `schemaVersion`
- `architectureBaselineId`
- `packageId` / `packageVersion`
- `manifestCanonicalization`
- `artifactIndex`
- Source Commit 和 Source Digest
- Build Recipe Digest
- Compiler、Linker、SDK、Generator 版本
- Feature Set
- Target Profile
- Root ABI Identity
- Capability Set
- Artifact Set Digest
- SBOM/License/Provenance Digest
- SignatureEnvelope Reference
- 未知字段处理和大小上限

### 3. 签名载荷与 Trust Policy

必须冻结：

- 签名覆盖的精确字节；
- Domain Separation；
- Algorithm 和参数；
- KeyId、TrustDomain；
- Trust Root 来源；
- Key Rotation、Revocation、Expiry；
- Offline Signer 与 Runtime Verifier 分离；
- 测试密钥与生产密钥隔离；
- 包内数据不得自举为信任根；
- 同一打开文件上的 Hash、Verify、Map 原子性要求。

### 4. Loader 状态机

建议至少：

```text
Uninitialized
  -> Preflighting
  -> Verified
  -> Binding
  -> ApiReady
  -> Leased

任意瞬态状态
  -> FailedRolledBack

Leased
  -> Quiescing
  -> Released
```

必须另外定义：

- 并发 Acquire；
- 同一包重复 Acquire；
- 不同包冲突；
- StaticLinked Backend；
- DynamicLibrary Backend；
- Timeout、Cancel、OOM；
- Partial Rollback；
- 是否物理 Unload；
- Host Owner Thread；
- Worker Join 和 Tick Barrier 的边界；
- Stable Error Mapping。

### 5. Package Identity 与兼容策略

`PackageIdentity` 不能只使用版本号，应至少绑定：

```text
ManifestDigest
ArtifactSetDigest
AbiIdentity
TargetProfileDigest
CapabilitySetDigest
```

兼容不等于允许同进程加载第二包。V1 应坚持：

> 同一进程只能锁定一个精确 PackageIdentity。

## P1：第一条 Vertical Slice 前完成

### 6. Platform Matrix

至少登记：

- OS
- CPU Architecture
- ABI / libc / C Runtime
- Minimum OS
- Compiler、Linker 和 SDK
- Dynamic/Static
- Symbol Visibility
- Library Naming
- Debug Symbol Format
- AOT Restrictions
- Loader Backend
- 支持的 Host Profile
- 必需和可选 Capability

### 7. Failure Fixture Matrix

至少覆盖：

| 分类 | 正例 | 负例 |
|---|---|---|
| ABI | 正确 Table/Layout | Major、Size、Pointer Width、Slot、Calling Convention 错误 |
| Manifest | Canonical Valid | 缺字段、重复路径、未知必需字段、非规范序列化 |
| Hash | 全部匹配 | 文件缺失、大小变化、单字节篡改 |
| Signature | 可信签名 | 错 Key、过期、撤销、Payload 不匹配 |
| Platform | 匹配 Target | OS、Arch、libc、Link Backend 错误 |
| Loader | 首次 Acquire | 并发第二包、符号缺失、Partial Load、Timeout |
| Handle | 合法释放 | 重复释放、跨 Package Handle、Stale Generation |
| Diagnostics | 正常发射 | Sink 拒绝、队列满载、Failure Fragment 不完整 |

### 8. Diagnostics Event Schema

至少定义统一字段：

- `timestamp`
- `traceId`
- `spanId`
- `packageIdentity`
- `manifestDigest`
- `artifactDigest`
- `module`
- `phase`
- `loaderState`
- `targetProfile`
- `errorCode`
- `retryability`
- `trustDecision`
- `evidenceReference`

### 9. 模块版本与兼容策略

必须区分：

- ABI Version
- Manifest Schema Version
- Package Format Version
- Capability Registry Version
- Error Registry Version
- Tool Version
- Build Recipe Version
- Signature Envelope Version

不能只使用一个泛化的 CoreEngine Version 替代全部维度。

---

# 6. 文档与架构源同步建议

## 6.1 可以直接在本仓 README 修正

这些内容不改变跨仓公共语义，可以在本仓修正：

- 把模块图拆成 Source、Artifact、Runtime、Validation、Observability 五个视图；
- 明确每张图的箭头语义；
- 将 composition 和 platform 的构建所有权写清；
- 把 smoke 标记为 Validation Plane；
- 把 diagnostics 标记为 Adapter/Support Plane；
- 修正 PureHeadless 与 CoreEngine Loader 的关系；
- 区分 `Current Fact`、`Design Requirement`、`Pending Decision`；
- 补齐每个模块的 Owner、State、Retry、Rollback、Timeout、OOM 和 Acceptance；
- 新增根级 Artifact Producer/Consumer 表；
- 新增仓内 Failure Matrix 文档；
- 增加 `modules/**` 与 `docs/architecture/**` 的全量链接检查；
- 明确当前全部模块仍为设计状态，避免将 README 验收要求写成“已经实现”。

当前文档已经明确这些模块属于设计文档而非已实现代码，因此没有发现“README 虚假声称实现完成”的问题。([modules/README.md](https://github.com/LumioGames/LumioCoreEngine/raw/refs/heads/main/modules/README.md))

## 6.2 应在 `.spec/decisions/` 新增的本仓 ADR

仅限不会改变公共语义的实现选择：

1. Build Orchestration：composition BuildPlan 如何由 platform Backend 执行；
2. Loader Registry 的并发实现；
3. Linux/Windows 动态加载库选择；
4. StaticLinked Backend 的适配方式；
5. V1 是否采用 No-Physical-Unload；
6. Canonicalization Library 的实现选择；
7. Signer/Verifier Provider 的代码组织和部署隔离；
8. 本地 Artifact Cache 与目录布局；
9. Diagnostic Adapter 如何接收 Host Sink。

其中“是否允许同进程多包”“错误码”“ABI 布局”“签名载荷”不是本仓内部实现选择，不能只写本仓 ADR。

## 6.3 必须回到 LumioGameEngineArchitecture 修改

以下属于公共契约：

- 完整 Root ABI Schema；
- CoreEngineManifest Schema；
- ArtifactIndex 和 PackageIdentity；
- SignatureEnvelope 和 Trust Policy；
- Loader 公共状态、错误和兼容行为；
- Target Profile / Load Backend / Host Profile 映射；
- CoreEngine ErrorCode 和 Capability ID；
- CoreEngine 正负 Fixture；
- ReleaseManifest 对 CoreEngine Package 的精确引用；
- Failure Bundle 对无 Snapshot 故障的支持；
- smoke、diagnostics 若保留一级模块时的公共模块地图和 RACI；
- Signing、Release Toolchain、Runtime Verifier 的责任划分。

公共规则已经明确：公共 ABI、Manifest、状态机、字段、错误码、ID、版本和依赖图的变化必须先回到唯一架构源，而不是在实现仓镜像中创造新语义。([repository architecture standard](https://raw.githubusercontent.com/LumioGames/LumioCoreEngine/caec3da/.spec/knowledge/standards/repository-architecture.md))

## 6.4 只能同步镜像、不能直接在镜像中决策

正确流程应是：

```text
LumioGameEngineArchitecture
  -> 公共 ADR / Schema / Fixture / ID Registry 更新
  -> 新 Baseline 或明确兼容更新
  -> 发布新的 Baseline Hash
  -> 同步 LumioCoreEngine 的 v1.0 镜像
  -> 更新 docs/architecture/.baseline.sha256
  -> 运行 Repository Policy
```

历史 `v0.3` 文件只是 Deprecated Compatibility Pointer，明确指向 v1.0 基线，并禁止在该文件中加入新架构决策。

---

# 7. 实现前阻断项

以下十项是真正阻断正式实现的事项：

1. **补齐可生成实际二进制布局的 Root ABI Schema。**  
   否则第一行 ABI 代码就会在本仓创造第二套公共语义。

2. **冻结 CoreEngineManifestBody、ArtifactIndex 和 Canonical Serialization。**  
   否则 Hash、缓存、签名和 Loader 无法使用同一份包身份。

3. **冻结 SignatureEnvelope、签名载荷和 Trust Policy。**  
   否则存在未绑定证据、自引用和自信任风险。

4. **冻结 Loader 状态机、PackageIdentity 和单进程锁定规则。**  
   否则可能发生重复加载、符号冲突和 API 指针 UAF。

5. **明确 Runtime Verifier 与 Offline Signer 的安全边界。**  
   否则生产 Loader 可能携带签名能力或错误信任离线结果。

6. **拆分依赖图、产物流、验证流和诊断事件流。**  
   否则生产代码依赖关系没有可靠基线。

7. **消除 composition 与 platform 的构建所有权重叠。**  
   否则 Source/Compiler/Feature/Output Hash 无法形成单一来源链。

8. **定义一个最小 P0 Target Profile 和 Static/Dynamic Load Backend。**  
   否则 Loader、Manifest 和 Smoke 没有可执行的第一目标平台。

9. **登记稳定 ErrorCode、Capability 和 P0 正负 Fixture。**  
   否则跨仓错误处理和回归验证无法稳定。

10. **收窄 diagnostics，并明确 Audit 与 Failure Bundle 所有权。**  
    否则 Loader 准入证据可能通过普通诊断队列丢失或反向阻塞运行时。

---

# 8. 最终建议

## 8.1 现在是否可以开始实现

**不建议开始正式生产实现。**

可以立即开始的工作：

- 公共 ADR；
- Schema；
- ID Registry；
- 正负 Fixture；
- 本仓模块图和 README 收敛；
- 纯验证性 Generator Prototype，但不得作为公开 ABI 基线发布。

不应立即开始：

- 对外发布的 Root C ABI；
- 可被 Runtime/Server 正式引用的 P/Invoke；
- 生产 Loader；
- 生产签名和信任根；
- 多平台包发布。

## 8.2 推荐的修正顺序

```text
1. 公共 Root ABI ADR + Schema
2. 公共 CoreEngine Package/Manifest/Signature ADR + Schema
3. 公共 Loader State/Error/PackageIdentity ADR
4. 公共 Target Profile 与 Capability 更新
5. 公共 ID Registry + Failure Fixtures
6. 更新公共模块地图和 RACI
7. 发布新 Baseline/Hash
8. 同步本仓只读镜像
9. 新增本仓内部实现 ADR
10. 再开始 Vertical Slice
```

## 8.3 修正后建议首先实现的 P0 能力

建议第一批实现顺序：

1. `root-abi`
   - Schema Validator；
   - C Header Generator；
   - Rust/C# Binding Generator；
   - Layout Fixture。

2. `composition`
   - Source Lock；
   - BuildPlan；
   - Input Digest；
   - Provenance Record。

3. `platform` 的最小 P0 子集
   - 只实现一个明确的 Linux Server Target Profile；
   - 一个明确的链接和加载 Backend；
   - ArtifactIndex；
   - Debug Symbol 映射。

4. `manifest`
   - Canonical ManifestBody；
   - Artifact Set Digest；
   - Schema Validation。

5. `signing` 的最小 P0 子集
   - 测试 Signer；
   - Runtime Verifier；
   - Detached SignatureEnvelope；
   - Tamper Fixtures。  
   生产 Key Management 和 Rotation 可延后。

6. `loader`
   - Process Package Latch；
   - Preflight；
   - Same-Package Idempotent Acquire；
   - Different-Package Rejection；
   - V1 No-Physical-Unload；
   - Stable Errors。

7. `smoke` 验证平面
   - 贯穿以上模块的一条端到端 Slice。

## 8.4 可以延后到 P1 的内容

- 完整 Windows/Desktop/iOS/Android 平台矩阵；
- MobileLocal 的 AOT、Unity、HybridCLR 适配；
- 生产远程签名和 HSM/KMS Provider；
- 完整 Key Rotation、Revocation 和透明日志；
- 完整 SBOM/License 自动发现；
- 诊断批处理、磁盘 Spool 和上传；
- 完整 Failure Bundle UI；
- 物理 Native Library Unload；
- 多种 Compiler/SDK 并存；
- Release Pool 和产品级 Migration——这些继续由上层仓库负责。

## 8.5 第一条 Vertical Slice 前必须验证的风险

第一条 Slice 不应只验证“能加载”，而应同时验证：

1. Rust、C Header、C# Binding 的结构大小、偏移和 Calling Convention 完全一致；
2. Panic/Exception 不穿越 ABI；
3. 错误详情的内存生命周期明确；
4. Hash、Signature 和动态加载针对同一组已打开字节；
5. 包中任何一个字节变化都会导致验证失败；
6. 同一包并发 Acquire 只映射一次；
7. 不同包无论 ABI 看似是否兼容，都被同一进程拒绝；
8. Partial Load、Symbol Missing、OOM 和 Timeout 不留下半初始化 Registry；
9. Loader 初始化发生在规定 Host Thread；
10. Native Worker 不回调 Managed 热路径，Tick Barrier 仍由其真正所有者管理；
11. StaticLinked 与 DynamicLibrary Backend 产生一致的 PackageIdentity 和 RootApiTable Contract；
12. CoreEngine 故障在没有 Game Snapshot 时仍能生成合法 Failure Evidence；
13. LocalEmbedded 可以拥有多个独立 World，但进程仍只有一套 Native Package；
14. PureHeadless 可以完全绕过 CoreEngine，而不是加载一个空包。

---

## 最终结论

**CoreEngine 的领域定位正确，六个核心生产模块的基本方向也成立；问题集中在公共契约未冻结、模块执行平面混合，以及供应链和 Loader 安全语义不足。**

因此本次结论不是“推翻模块化设计”，而是：

> **保留 composition、root-abi、loader、manifest、signing/supply-chain、platform 的总体框架；把 smoke 降为验证平面，把 diagnostics 收窄为适配平面；先在公共架构源补齐 ABI、Manifest、Signature、Loader、Platform、Error 和 Fixture 契约，再进入实现。**
