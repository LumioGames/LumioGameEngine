# 2026-08-29 · `LGE-V1.5` 跃迁批规划(执行编排 + 验收判据)

> **本文是规划,不是执行。** 落地本文任何一项都需要用户对跃迁批的终批(R-00269 卡面「边界」明写)。
> 批内容由 [`2026-08-29-contract-surface-adjudication.md`](2026-08-29-contract-surface-adjudication.md) §裁决三与 [`2026-08-29-td-handoff.md`](2026-08-29-td-handoff.md) §4/§5.1 固定,**本文不增删方向**,只补「改动面 / 顺序 / 判据 / 编排」。
> 度量锚点:`origin/main = 3287bba`(2026-08-29)。下述所有计数均为本文写作时的实测值,执行前须重测——它们会随 additive 增补腐烂,判据一律写成「存在性 + 身份」而非硬编码计数(承 TD 交接 §5.3 Server ErrorCode 条的已定口径)。

## 0 · 为什么是一批而不是七次

`schemas/README.md` 的变更规则:改必需字段、**枚举**、revision 语义、相位、ID 布局或兼容规则,须 ADR + 正反 Fixture + 重生成物 + **新 BaselineId** + 七仓镜像同步。

批内**六项里有三项各自独立触发这条规则**(D-1 的 MessageType 增值与 body 必需集扩字段、R-00009 的 `loadBackend` 枚举与 `packaging` 概念置换、ADR-040..048 转 Accepted 后的公共构造再登记)。若各跳各的,下游要吃三次「全量重新 pin」;D-11 的裁决口径是**只跳一次基线**,故合批。

**基线跃迁的实测钉死面**(`grep -rl "LGE-V1.4-2026-08-27"`,`origin/main = 3287bba`):

| 面 | 计数 | 性质 |
|---|--:|---|
| `schemas/*.json` 的 `"const": "LGE-V1.4-2026-08-27"` | 11 | 手改 |
| `schemas/index.json` `baselineId` | 1 | 手改 |
| `ids/index.json` `baselineId` | 1 | 手改 |
| `fixtures/**/*.json` | 37 | 手改 |
| `packages/**/*.json` | 31 | **生成物,禁手改**,随 `generate` 重发 |
| `tools/lumio_generate.py:12` `BASELINE` | 1 行 | 手改(生成源) |
| `tools/lumio_contract.py:441` `_CURRENT_BASELINE` | 1 行 | 手改(校验源) |
| `.github/workflows/repository-policy.yml:37-38` grep | 2 行 | 手改 |
| `docs/architecture/.baseline.sha256` | 1 行 | 随正文重算 |
| 架构正文 | 新文件 `LumioGameEngine_Architecture_v1.5.md` | 沿 v1.1→v1.4 惯例逐版新建,**CI 的 `test -s` 与四条 `^## N.` 章节 grep 必须同步改指向** |
| `README.md` | 1 处 | 手改 |

> **两条最容易漏的**:① CI 的 `grep -q '^# LumioGameEngine V3 (v1.4)'` 与 `test -s docs/.../v1.4.md` 是**文件名与标题双重钉死**,新建 v1.5 正文而不改 CI = 绿着骗人;② `packages/` 31 件是 `generate` 的产物,`command_generate()` 会 `rmtree` 输出目录——PR #27 (`3287bba`) 刚修完 `Cargo.lock` 被删的同族问题,跃迁批重生成前须确认该修复在场。

## 1 · 批内六项:改动面 / 依赖 / 判据

### 项 1 · D-1 状态载荷与输入承载(**批内唯一新公共语义**)

- **ADR**:本批同时交付的 [`ADR-049`](../../.spec/decisions/ADR-049-replication-state-payload-and-input-command.md)(Draft,本卡产出)。
- **改动面**:
  - Schema:`replication-envelope.schema.json` —— `messageType` 枚举增 `InputCommand`;`FullSnapshot.body` 必需集增 `stateBlocks`、`Delta.body` 必需集增 `changedBlocks`;新增 `if/then` 分支约束 `InputCommand`。新增 `input-envelope.schema.json` 并登记进 `schemas/index.json`。
  - ID:`ids/index.json` 的 `MessageType` namespace(owner GameRuntime)增 `InputCommand`,`numeric = 9`(现最高 8,**不复用、不回填**,承 `ids/README.md` 的不复用规则)。
  - Fixture:正向 `replication/full-snapshot-state-blocks`、`replication/delta-changed-blocks`、`input/command`;失败向至少 `replication/state-block-payload-hash-mismatch`、`replication/state-block-order-violation`、`input/unregistered-target`(逐条对应 ADR-049 §失败语义,一条判据一条反例——承 lessons「判据与它的反例测试同一提交内同时诞生」)。
  - 生成物:八类本体里的 `ReplicationEnvelope` 随 schema 重发;`compilerHash` 全量移动。
  - README/Baseline:`schemas/README.md` 无需改文,但 BaselineId 全面跃迁。
  - 七仓镜像:GameRuntime(MessageType owner)、Server、Client 三仓的 replication 面;CoreEngine 契约镜像 required paths。
- **顺序依赖**:**必须最先落 ADR-049 转 Accepted**(它是本批唯一引入新语义的项);其 `stateBlocks` 编码依赖 ADR-047(LumioBinV1)已 Accepted,故**项 3 的 ADR-047 转 Accepted 必须先于或同刻于本项**。
- **验收判据**:
  1. `messageType` 枚举、`ids/index.json` 的 MessageType 值集、fixture 实际用到的 messageType 三者**同集**(`lumio_contract.py` 已有此断言,ADR-028 立);
  2. 每条新增必需字段有**至少一条正向 + 一条失败** fixture,失败 fixture 移除该字段后 `validate` EXIT≠0、恢复后 EXIT=0(对照组探针,不接受「validate 通过」作为守护生效的证据);
  3. `stateBlocks` / `changedBlocks` 的 `payloadHash` 由门禁**重算**而非只校验格式(承 ADR-047 §3「vector 不能腐烂成谎言」的同款要求);
  4. A1-β 验收项「另一个客户端看到方块被挖掉」在本项落地后**才**从 BLOCKED 解冻——它是本批的下游解阻信号,不是本批的判据。

### 项 2 · R-00009 三轴枚举对齐(TargetProfile / LoadBackend / PackagingProfile)

- **实测漂移**(承 D-11,本文复测 `origin/main = 3287bba` 仍成立):
  - `schemas/target-profile.schema.json:22` `loadBackend = ["StaticLinked","DynamicLibrary"]`;架构正文 §10 写 `DynamicLibrary`/`StaticLink`/`NoNative` —— **改名 + 缺一值**;
  - Schema 的 `packaging` 是打包细节对象 `{libraryFileName, debugSymbolFormat, archiveFormat}`,正文的 `PackagingProfile` 是三值枚举 `LooseFiles`/`Archive`/`EmbeddedInApp` —— **是两个不同概念,不是拼写差**。
- **改动面**:ADR-020 的 refine ADR(新号,**不改写 Accepted 的 ADR-020 正文**——`Accepted` 后不可改写是 `decisions/README.md` 的硬规则,只能新增 ADR 并双向记录取代/refine 关系)+ `target-profile.schema.json` + 正文 §10 + `fixtures/valid/target-profile-linux-server.json` 及新增各目标平台正反 profile + `targetProfileDigest` Golden(ADR-041 的 `TargetProfileV1` 摘要域已就绪,**直接复用,不新增摘要域**) + 七仓镜像(CoreEngine 为主要消费方)。
- **顺序依赖**:与项 1 **无依赖,可并行**(文件集不重叠:项 1 动 replication/input,项 2 动 target-profile)。但两者都动 `fixtures/index.json` 与 `schemas/index.json` —— **索引文件是共享热点,两项的索引条目必须串行合入或由同一执行者一次写入**。
- **验收判据**:① 全仓无 `StaticLinked`/`StaticLink` 双拼写(`git grep` 零命中其一);② `packaging` 与 `PackagingProfile` 概念在 Schema 与正文里指同一物或显式分列两字段并各自定义;③ R-00009 卡面原验收项逐条成立:「CoreEngine 不含 alias/双枚举」「所有 P1 config 只通过单一生成类型」—— 后者须由 CoreEngine 侧出证据,**本仓无法自证**,属跨仓验收项。

### 项 3 · ADR-040..048 九张 Draft 转 Accepted

- **范围**:ADR-040 / 041 / 042 / 043 / 044 / 045 / 046 / 047 / 048,共九张(D-11 当时只数到 040–044 五张;此后 045–048 陆续以 Draft 落地,本批一并处理)。
- **改动面**:每张 ADR 的 `Status` 行 + `decisions/README.md` 索引表状态列 + 索引前言的「随哪个基线接受」叙述句 + `docs/adr/` 软链接完整性(**已知缺口:ADR-045 缺软链接**,见 TD 交接 §附录 P2-5,本批一并补)。**ADR 正文其余部分不动**——转 Accepted 是状态迁移,不是改写。
- **顺序依赖**:ADR-047 必须与项 1 同刻或更早(项 1 的编码引用它);其余八张与项 1/2 无顺序耦合。**但九张一起转,不拆批**——拆批就等于「先转一部分 Accepted、剩下的等下一次基线」,而 D-2 排序约束 3 明写:把 Draft 公共构造钉进 CoreEngine 只读镜像,等它转 Accepted 时会再触发一次全量重新登记。拆批 = 多一次全下游 churn。
- **验收判据**:① 九张 `Status` 均为 `Accepted` 且标注基线 `LGE-V1.5-<date>`;② `decisions/README.md` 索引表状态列与正文一致(`spec-lint` 校验 status 枚举);③ `docs/adr/` 软链接对 `.spec/decisions/` **全覆盖无缺口**(本批须把这条补进 `spec-lint`,否则 ADR-045 那类缺口会再犯——**这是本批唯一建议新增的机器检查**);④ ADR-015 保持 `Reserved`,不被批量误转。

### 项 4 · OperationId namespace 发布 —— **已裁决:出批,改为记录裁决**

> **TD 总调度裁决(2026-08-29,终局)**:**采纳下述「优先选项」——项 4 出批,不在 V1.5 批内发布 OperationId namespace。**
>
> **裁决依据(三条,均经第一手核实)**:
> 1. **ADR-040 §7 的否决带条件从句**,原文第 119 行:「There is no `OperationId` namespace, none is reserved, and none is required **while the dispatch surface stays blocked**.」——条件是「dispatch 面仍被挡着」。
> 2. **该条件至今成立**:`packages/index.json` 的 `blocked` 列表实测为 `[{"id":"D-009","reason":"protocol-dispatch not frozen"},{"id":"D-011","reason":"Auth wire not frozen"}]`。**D-009 未解冻**,故 ADR-040 §7 的结论仍然有效。
> 3. **发布它等于抢跑 D-009**,与本日已定的同型裁决一致:ADR-048 §2 明写 validator「只校验已注册、不校验角色权限」,理由正是「架构源无 role→message 权限表,发一个就是发明公共合同并抢跑 D-009」。同一条红线在此复用。
>
> **附加理由**:项 4 与项 3 在同一批内自相矛盾(既把「没有 OperationId namespace」冻成 Accepted,又发布该 namespace);保留项 4 的代价(先解冻 D-009 + 新 ADR 取代 ADR-040 §7)比本批其余五项之和更大,与「只跳一次基线」直接相悖。
>
> **由此确定的终态口径(下游据此执行,不再当缺口上报)**:NativeCore 的 `ArchitectureOperationId` / `operation_ids()` **空 seam 是符合规范的终态**,不是待补缺口。已同步:① 修订 `docs/reviews/2026-08-29-nativecore-closeout.md` §4 的残留漂移句;② 已 SendMessage 通知在途的 NativeCore R-00083/R-00007 会话。
>
> **重新开启的唯一条件**:D-009(protocol-dispatch)解冻。届时须新增一张 ADR 取代 ADR-040 §7 该条,不得靠「批内顺带发布」绕过。

- **实测冲突**:批清单写「OperationId namespace 发布(需 NativeCore 提值)」,但 [ADR-040 §7](../../.spec/decisions/ADR-040-root-abi-generated-bundle.md) 第 119 行明写:「可调用操作的公共身份是 (`apiTable[].name`, `slots[].slotIndex`) 这一对,已发布在 bundle 里并被布局 Golden 断言。**没有 `OperationId` 命名空间,不保留、也不需要保留**,只要 dispatch 面还被 D-009 挡着。」`docs/reviews/2026-08-28-nativecore-abi-adjudication.md` §请求 4 已把它裁为「不适用」,理由是「这不是注册表缺失,是概念不存在」。
- **含义**:项 4 与项 3(ADR-040 转 Accepted)**直接冲突**——同一批里既要把「没有 OperationId namespace」冻成 Accepted,又要发布 OperationId namespace。
- **处置(已裁决,见上方裁决框)**:
  - ~~优先选项~~ → **已采纳**:项 4 出批,记录裁决;`docs/reviews/2026-08-29-nativecore-closeout.md` §4 的残留漂移句已随本次裁决一并修订。
  - ~~若要保留项 4~~ → **已否决**:前置是 D-009 先解冻 + 新增 ADR 取代 ADR-040 §7,动作大于本批其余五项之和。
- **验收判据**:全仓关于 OperationId 的表述**单一口径**,`git grep -n "OperationId"` 的每一处命中要么指向 ADR-040 §7 的裁决,要么是历史 review 文档的原文引用并已标注「已被裁决取代」。
- **判据执行结果(本次已完成)**:`git grep -n "OperationId" -- '*.md'` 全量命中逐条分类处置——
  - **源头**(`ADR-040:119`):不动,它就是裁决依据本身。
  - **三处「活的」待办已收敛**:`2026-08-29-contract-surface-adjudication.md:25`、`2026-08-29-td-handoff.md:103`、`2026-08-28-stepwise-convergence-roadmap.md:48` 原本都把「OperationId namespace 发布」列为待执行项,现已就地标注「已裁决出批 / 不适用」——**这三处是真实的口径冲突源,不处理就会有人照旧执行**。
  - **`2026-08-29-nativecore-closeout.md:39`**:残留漂移句「待上游发布后随小卡收敛」已改写为终态口径(该文件虽属 review,但那句是**面向未来的承诺**而非事实记录,故必须改)。
  - **历史 review / audit 快照不改写**(`2026-08-28-nativecore-abi-adjudication.md`、`2026-08-28-nativecore-convergence-audit.md`、`2026-08-28-nativecore-convergence-dispatch.md`):它们记录的是当时的事实与判断,且方向与本裁决**一致**(前者已裁为「不适用」,后两者写「未发布」「不得发明」)。审计报告是时点快照,改写会破坏其证据性质。

### 项 7 · `canonical_object_pairs` 删除与类型化编码器发布 —— **已裁决加入本批(2026-08-29)**

> **本项是对本批的一次扩容**,授权来自 [`2026-08-29-canonical-object-pairs-adjudication.md`](2026-08-29-canonical-object-pairs-adjudication.md)。本规划扉页禁止自行扩容,故此处显式记录裁决出处。

- **问题**:已发布的 `packages/rust/lumio-gen-contract-runtime/src/lib.rs:40` 的 `canonical_object_pairs` key/value 均不转义、value 不加引号直接拼接、不拒重复 key,**可构造指纹碰撞**(三条独立路径),且同时违反本仓已发布的 `CANONICAL_ENCODING=AsciiEscaped` 与 `CANONICAL_DUPLICATE_MEMBERS=Reject` 两条冻结条款。
- **改动面**:**删除**该函数;发布**自有 formId**(建议 `CanonicalObjectV1`)的**类型化构造式编码器**(值自持,非法状态不可表达);两语言对称发布(当前 C# 侧无对等物,已违反 ADR-039:16 的 `identical observable behavior` 要求);新增 ADR;新增向量表含**构造 X**(C# 孤代理 → strict UTF8 + `LoneSurrogate`)与**构造 Y**(`astral-vs-bmp` 跨语言排序)。
- **为何必须跳基线(即为何必须在本批)**:ADR-041:100 当初保基线的理由是「Nothing was removed」,**该前提已被本次删除推翻**;`baselineId` 是 `const`。**若被当成普通 fix 批合入,下游会得到一个 baselineId 没变、公共面却不兼容的 artifact。**
- **不得声称 `CanonicalJsonV1`、不得扩 ADR-041 §78 绑定面清单**:ADR-041:22 要求成员名匹配 `^[A-Za-z][A-Za-z0-9]*$`,而该 helper 的全部真实 key(`txn_id`、`c:0:0:0`)**一个都不匹配**。盖假合规章是 K[28] 同型错误。
- **顺序依赖**:与项 3(ADR 转 Accepted)同刻或更早;**七仓重 pin(W3)必须在本项落地之后**,否则重 pin 白做。
- **验收判据**:① 旧函数在两语言公共面**均不存在**;② 新编码器两语言**行为等价**并由跨语言向量表机器验证;③ 构造 X / Y 各有一条按其构造的**失败**用例;④ 对照组探针**自动化进 CI**(不接受只在写 ADR 那天人工跑一次);⑤ 10 条既有 golden **逐字节不变**(本项不触碰 CanonicalJsonV1)。

### 项 5 · D-5 冻结点 tag

- **问题原文**(D-5):`compilerHash` 一天四变,下游没有可引用的稳定点;「请给 tag 或 artifact digest,**不要 branch name**」。PR #23 之后又变了一次(全 12 件 `compilerHash` → `870e8635…`),证明问题仍活跃。
- **改动面**:**零 Schema / 零 Fixture / 零 ID**。产出是:① 跃迁批合入后在该提交上打 **annotated tag `LGE-V1.5-<date>`**;② 一份下游 pin 指引(落 `README.md` 的「Published artifacts」段或独立 `docs/architecture/PINNING.md`),写清「pin tag 或 pin `packages/index.json` 的 artifact digest,**不 pin branch**」;③ 七仓的 `.baseline.sha256` / lock / 契约镜像改引用 tag。
- **顺序依赖**:**必须是本批最后一步**——tag 打在批的终态提交上。
- **验收判据**:① tag 存在且为 annotated、指向本批合入后的 `origin/main`;② 至少一个下游仓(建议 CoreEngine,契约镜像最重)按 tag 重 pin 成功、`check-contracts` 绿;③ 指引文档明写「不 pin branch name」。
- **本批不做的两个 D-5 子项**(D-5 原文含三问,只答第一问):`compilerHash` 是否拆成「生成器版本号 + 内容哈希」、`tools/**` 是否该进契约镜像 —— 两者都是**独立设计决策**,塞进跃迁批会把批的验证面撑大一倍。建议出批另立卡。

### 项 6 · trust 两条 P2(`signedAt` preimage、时间窗比较)

- **两条原文**(TD 交接 §附录 P2 台账 2/3):
  - **P2-2**:`signedAt` 不在签名 preimage 内,时间窗检查不受密码学保护 —— **Test 域可接受,Production 域冻结前须 ADR 显式处置或 preimage v2**;
  - **P2-3**:时间窗比较是**字典序**(`lumio_contract.py:990`),分数秒时间戳会误判 —— 潜伏缺陷。
- **改动面**:
  - P2-2 若选「ADR 显式处置」= 在项 3 的 ADR-042 转 Accepted 时**附一节裁决记录**说明该限制与其适用域边界(不改 preimage,零 Schema 改动);若选「preimage v2」= 改 `trust-profile.schema.json` 的 preimage 布局 + 全部 trust 向量重算 + 七仓 verifier 同步 —— **这是一次密码学构造变更**,批内做会让本批的验证面从「枚举与字段」扩到「签名字节」。
  - P2-3 是**纯实现缺陷修复**(比较前解析时间戳,或收紧 `common.schema.json` 的 `timestamp` def 禁止分数秒),零公共语义变更,**可在批外任意时点单独修**,不必等基线。
- **顺序依赖**:P2-2 的「ADR 显式处置」路径与项 3 同刻(同一份 ADR-042 转 Accepted);P2-3 **无依赖**。
- **裁决(TD 总调度,2026-08-29,终局)**:**采纳建议——P2-2 走「ADR 显式处置」路径,P2-3 出批单修。**
  - **P2-2 不做 preimage v2**:那是一次**密码学构造变更**,会把本批的验证面从「枚举与字段」扩到「签名字节」,失败面不可控;而 `trustDomain` 实测为 `Test`(`packages/index.json` 的 `trust.trustDomain`),P2-2 原文本身就写明「Test 域可接受」——当前域下不构成必须在本批解决的风险。**执行**:在项 3 的 ADR-042 转 Accepted 时附一节裁决记录,写清 `signedAt` 不在签名 preimage 内、该限制的适用域边界、以及 **Production 域冻结的前置条件必须包含 preimage v2 或等效处置**——把约束钉在未来那次域切换上,而不是留成无主待办。
  - **P2-3 出批单修**:纯实现缺陷(`lumio_contract.py` 时间窗字典序比较),零公共语义变更,不必等基线,任意时点可修。
  - **共同理由**:批的目的是「只跳一次基线」,不是「把所有待办清空」。凡「能独立成批且不改公共语义」的,一律出批——这条同时适用于本批其余待办的取舍。
- **验收判据**:① ADR-042 转 Accepted 的正文里有一节明写 `signedAt` 不受签名保护、其适用域限于 Test、Production 冻结的前置条件是什么;② P2-3 修复后有一条**分数秒时间戳的失败 fixture**——不是「改了比较函数」,是「有一条按旧比较逻辑会误判、按新逻辑会正确的对照用例」。

## 2 · 只跳一次基线的执行顺序(公共语义变更顺序的批级展开)

规范顺序是 **ADR → Schema/ID → 正向与失败 Fixture → README/Baseline → 七仓镜像**。批级展开后是七道,**道内可并行、道间严格串行**:

| 道 | 内容 | 并行性 | 出口判据 |
|---|---|---|---|
| **W0** | 用户对跃迁批**终批**;总调度裁决项 4(OperationId)与项 6(P2-2 路径)的处置 | — | 两项裁决成文落库 |
| **W1** | **ADR 层**:项 1 的 ADR-049 终稿;项 2 的 ADR-020 refine 新 ADR;项 3 的九张状态迁移;项 6 的 ADR-042 附节 | ADR-049 与 refine ADR 可并行(不同文件);项 3 需在两者定稿后统一编号与基线标注 | `spec-lint` 绿;`decisions/README.md` 索引与正文一致;`docs/adr/` 软链接全覆盖 |
| **W2** | **Schema/ID 层**:项 1 的 replication + input schema 与 MessageType 增值;项 2 的 target-profile 三轴 | 两者文件集不重叠 → **可并行**;但 `schemas/index.json` / `ids/index.json` 是共享热点 → **索引条目串行写入** | 结构校验通过;枚举三方同集断言通过 |
| **W3** | **Fixture 层**:两项各自的正向 + 失败 fixture;`targetProfileDigest` Golden | 可并行,`fixtures/index.json` 同为共享热点须串行写入 | **每条新判据都有一条按该判据构造的失败用例**;对照组探针实录(制造违规→红→移除→绿) |
| **W4** | **基线跃迁**:`_CURRENT_BASELINE` / `BASELINE` 两行 + 11 处 schema `const` + 三个 index + 37 个 fixture + `README.md` + 新建 `LumioGameEngine_Architecture_v1.5.md` + `.baseline.sha256` 重算 + **CI 的 `test -s` 与 grep 全部改指向 v1.5** | **不可并行**——这一道是全仓单点 | `python3 tools/lumio_contract.py validate` EXIT=0;`.github/workflows/repository-policy.yml` 的每条检查在本地复现通过 |
| **W5** | **生成物重发**:`generate` 重出 `packages/` 31 件;`compilerHash` / `outputHash` 全量移动并记录 | 不可并行 | `generate` 后 `git status` 除预期文件外零漂移;**`packages/rust/Cargo.lock` 仍在**(PR #27 修复的回归面);`outputHash` 稳定性检查(CI 的 publish 步)通过 |
| **W6** | **七仓镜像 + tag**:七仓按新基线重 pin;最后打 annotated tag(项 5) | 七仓**可并行**(各仓独立 worktree / 独立会话);tag 在全部镜像绿后打 | 每仓 `check-contracts` / `.baseline.sha256` 绿;tag 指向终态提交;**上游同步度按已定裁决走报告项而非硬 fail**(TD 交接 §5.3),故某仓滞后不阻塞打 tag,但须登记 |

**回滚**:W1–W3 的产物是纯增量,`git revert` 即可。**W4 是不可逆点**——基线字符串一旦跃迁,七仓的 pin 全部指向新值;W4 之后回滚等于再跳一次基线。故 **W4 前必须完成一次 reviewer 对抗审查**(批级,不是逐项),这是本批唯一的强制审查点。

## 3 · 七仓波及矩阵

| 仓 | 项 1 D-1 | 项 2 R-00009 | 项 3 转 Accepted | 项 5 tag | 动作 |
|---|:--:|:--:|:--:|:--:|---|
| NativeCore | — | — | ADR-040/046 | ✓ | 重 pin `.baseline.sha256`;`ArchitectureOperationId` 依项 4 裁决**保持空 seam** |
| CoreEngine | — | **主要消费方** | ADR-040..044 全部 | ✓ | 契约镜像全量重登记(`tools/**` 在 required paths → 生成器改动必触发);三轴枚举去 alias |
| GameRuntime | **MessageType owner** | — | ADR-045/047/048 | ✓ | MessageType 增值由本仓 owner 确认;replication 面重实现;持久化可依 LumioBinV1 去 MessagePack |
| Server | ✓ 入站/出站 | — | ADR-045 | ✓ | Delta 合法集须继续接受 `gapDetected`/`resyncReason`(已定裁决);新 body 必需集接入 |
| Client | ✓ 出站 InputCommand | — | ADR-048(双目标) | ✓ | 上行输入承载;`netstandard2.1` 面已由 D-4 解 |
| VoxelEngine | 间接(payload 编码) | — | ADR-035/047 衔接 | ✓ | LumioBinV1 是其 ADR-035 payload 一直假定的 primitive 层 |
| Game | ✓ 域 payload schema | — | ADR-047 | ✓ | 域 payload 按 LumioBinV1 声明序编码 |

**dry-run 说明**:本卡**未执行**任何七仓预演——卡面边界明写「只规划与草案,不动基线」。W6 的每仓动作在批终批后由各仓会话按各自卡执行,本表只给波及面与动作性质,不构成已验证的迁移路径。

## 4 · 已知风险与未决

1. **项 4 与 ADR-040 §7 冲突**(§1 项 4)—— 未决,需总调度裁决,是本批唯一的方向性未决项。
2. **项 6 的 P2-2 路径选择** —— 未决;本文给出建议(ADR 显式处置)与理由,不代裁。
3. **计数会腐烂** —— §0 的所有计数是 `3287bba` 的实测值;W4 执行前须重测,判据不得硬编码计数。
4. **本批不含 D-5 其余两问与 D-6 残项** —— 显式出批,建议另立卡;塞进来会让批的验证面失控。
5. **CI 的 v1.4 文件名/标题双钉死** —— W4 最容易漏的一处,漏了会「绿着骗人」(CI 仍在校验旧正文,新正文无人校验)。
6. **9 周时间盒下本批是唯一大动作**,预算一周窗口(承裁决文档 §执行编排的风险自检)。
