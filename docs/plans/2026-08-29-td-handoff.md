# 2026-08-29 · TD 总调度交接(上下文续接用)

> 本文是架构仓 TD 会话的完整交接。读完即可接手,不需要前一个会话的上下文。
> 事实截止时刻:2026-08-29 傍晚。**所有引用的提交号都在 origin,可 `git branch -r --contains` 复核。**

## 0 · 你的角色

你是 **LumioGameEngineArchitecture 仓的 TD 总调度**(引擎总监职能)。职责:
- 盘点七仓进度、对账 Workflow 与仓库真实状态;
- **代用户裁决架构问题**(用户 2026-08-29 明确委任:「你有自主决策的权利,加快开发效率」),裁决须**成文落库**(确认书 / ADR / DECISIONS_PENDING / 卡面修订)并事后报备;
- 派活给各实现仓会话、核验交回物、流转卡状态(`done` 只由你流转);
- **新建 Workflow 卡 / Room / 里程碑仍需用户逐次授权**——委任的是裁决权,不是落卡权。对外发布(push / PR / merge)在收敛任务语境下已获持续授权。

工作流规范在 `.spec/`(`AGENTS.md` 调度核心、`rules/system.md` 红线、`knowledge/` 规范与 lessons)。跨仓派活用 `skills/cross-repo-delivery`,盘点用 `skills/td-progress-audit`。

## 1 · 真值分层(第一原则)

| 问题 | 唯一真值 |
|---|---|
| 需求要做什么、到哪个状态 | Workflow 需求单(项目 `lumiogamesengine`) |
| 公共契约长什么样 | 架构仓 Baseline / Schema / ID / Fixture / `packages/` |
| 代码实际做没做 | **已推送 origin 的提交** |
| 测试过没过 | **链接执行的真实输出**(`cargo check` / `dotnet build` 不算测试证据) |

Workflow API:token 在 `~/.config/workflow/config.toml` 的 `[profiles.lumiogamesengine]`,base 加 `/api/v1`。
**踩过的坑**:流转端点是 `POST /requirements/{id}/transition`(**单数**;复数路径只有 GET);`/search` 今日多次 500,改用 `GET /requirements?roomId=&limit=250` + cursor 全量枚举;响应偶发 chunked 截断,**失败后必须先读回确认是否已落库再重发**(否则重复评论)。

## 2 · 当前状态:**自己拉,不要读死数字**

**本节刻意不写卡数与 HEAD。** 今天的教训之一就是「计数会随 additive 增补必然腐烂」(见 §5.3 ErrorCode 条);同一天里各仓 HEAD 移动过五次以上、卡态被多个并行会话推进,任何写死的快照在你读到时已经是错的。

**开工第一件事跑这个**(约 1 分钟,八室 + 工作项 + 八仓 HEAD):

```bash
cd /Users/cui/LumioGames/LumioGameEngineArchitecture
CONFIG="$HOME/.config/workflow/config.toml"
BASE=$(sed -n "/^\[profiles\.lumiogamesengine\]$/,/^\[/s/^base_url = \"\(.*\)\"$/\1/p" "$CONFIG" | head -1)
export WORKFLOW_TOKEN=$(sed -n "/^\[profiles\.lumiogamesengine\]$/,/^\[/s/^token = \"\(.*\)\"$/\1/p" "$CONFIG" | head -1)
export WORKFLOW_API_BASE="$BASE/api/v1"
# 逐室 cursor 全量枚举(/search 会 500,别用它);各仓 git fetch --prune 后取 origin/main
```

**Room ↔ 仓映射(这个是稳定的)**:

| Room | 仓 | roomId(UUID) |
|---|---|---|
| RM-00001 | Architecture | `01a04225-4fc2-737e-afb3-8aaa8ba80754` |
| RM-00002 | NativeCore | `01a04225-58b8-7abc-84e1-da1e60e58102` |
| RM-00003 | VoxelEngine | `01a04225-6499-71ad-8548-5807eb51f421` |
| RM-00004 | CoreEngine | `01a04225-6ce8-75fd-bd3e-7535e9b232fd` |
| RM-00005 | GameRuntime | `01a04225-7526-70be-8950-32f83dd061fd` |
| RM-00006 | Server | `01a04225-7dfa-7968-8f03-fdff153fb727` |
| RM-00007 | Client | `01a04225-86b1-7be9-870a-adcecb10807c` |
| RM-00008 | Game | `01a04225-8f05-7e9a-aa9d-a0d7c5a685c6` |

**验收项判定用的 passed statusId**:`astat_ee91c1c5812a3def044bc2688f459241`(项目自定义值,变了就现查:从任一已通过卡的 `acceptance-items` 里读 `systemSemantic=="passed"` 那条的 `statusId`)。

**几条到交接时刻为止的结构性观察**(方向性参考,数字自己核):
- **NativeCore 已全部完成**,该室不再需要派活。
- **CoreEngine / Architecture / Runtime / Server 的待核销队列都很短**,多数卡已流转 done——并行会话推进得比交接文档写作时快得多。
- **Voxel 的 `in_review` 队列最长**,是当前最大的核销面;其中 **R-00203** 是唯一有明确退回理由的(SV-α 判 3/4:`mvp-review.md` 未逐张覆盖全体 P0 卡,需在绿基线上补一轮含全 P0 对照的复审)。
- **Game 两张都在 `acceptance`**:R-00259 待核,**R-00283 是美术终裁、留用户、不要动**。
- 工作项只剩 T-00003(review,待与 Client 其余卡一并核销)与 T-00004 / T-00005 / T-00009(todo)。

## 3 · 在途项状态(已闭环,无需接手)

**PR #23 已审已合入** —— `origin/main` = **`6637541`**。

内容:修复 `tools/lumio_generate.py` 的 `emit_canonical()`(写完 Rust `lib.rs` 后又继续追加 ADR-041 常量却没写回,导致 **Rust crate 漏发摘要域而 C# 侧完整**),并新增 `published_canonical_surface_errors()` 门禁对两语言同时断言。

**独立对抗审查结论:放行,无 P0/P1**(审查评论已贴在 PR #23)。关键核实:
- **变动面干净**——把 22 个 JSON 展平成路径→值逐字段比对,全仓**仅四类字段变动**(12 件 compilerHash `e401077a…`→`870e8635…`、`canonical-serializer-rust` 的 outputHash `9daf9c20…`→`4a524594…`、root-abi-bundle 的 compiler digest、index.json 的 bundleDigest),`baselineId`/`schemaEpoch`/`inputHash`/所有 artifact 内容字段**均未出现在差分里**。没有任何真实契约变更被夹带。
- **缺陷是孤例**——reviewer 自写静态检测器,修复后全仓 `emit_*` 零命中。
- **新门禁不空转**——4 个扰动各自只点名自己,端到端 `validate` EXIT=2 精确点名、还原 EXIT=0。
- 门槛全绿:validate 201 fixtures 0 失败、clippy `-D warnings` 0 行、6 csproj 双目标 0 Error、KAT 三方一致、零漂移。

> ⚠️ **下游消费方需要重新 pin**:全 12 件 compilerHash 已变动。CoreEngine 的契约镜像、NativeCore 的 `.baseline.sha256`、Runtime 的 generated manifest 都会因此报"上游已前进"。按已定裁决,这类应走**报告项而非硬 fail**(见 §5.3 Runtime 条)。

**审查发现的 5 条 P2(不阻塞,待授权立卡)**:
1. **门禁只校验域的存在、不校验域的内容**(`lumio_contract.py:1091-1101`)——把 `lib.rs` 里 `CapabilitySetV1` 的 `normalization` 改成 `&[]`,`validate` 仍 EXIT=0。**这恰是 ADR-041 自己 Verification 段写明的那个缺陷**(`canonical/missing-normalization`),最值得补。
2. **任一发布文件整体缺失 → 门禁静默放行**(`:1058`):移走整个 `lib.rs`,validate EXIT=0。属既有惯例(五个兄弟门禁同款写法)。
3. `errors[:6]` 截断(`:1111`)使双语断言的报告面打折:Rust 侧 ≥6 条时 C# 侧错误被整体挤掉。
4. **`generate` 会删掉受版本控制的 `packages/rust/Cargo.lock`**(`:2363-2365`):就地重生成再提交会静默丢掉 lockfile。
5. 交付方声称漏列 `packages/index.json /rootAbi/bundleDigest`(纯表述 nit,值本身正确)。

## 4 · 待办清单(建议优先级)

### P0 · 解阻与核销
1. ~~接手 PR #23~~ —— **已完成**(审查放行、已合入 `6637541`);其 5 条 P2 见 §3,**P2-1 与 P2-4 值得立卡**(待用户授权)。
2. **核销待核销队列**(先跑 §2 的脚本拿当前清单)。方法:每仓派一个 QA 会话(写的人 ≠ 判的人),按「锚点在 origin + 门禁实跑 + 验收项逐条判定 + 每卡一条证据评论」核销,卡保持验收中,**你复核后流转 done**。**Voxel 队列最长,拆两道**(奇/偶卡号,今日用过该切法,互斥且好核对)。R-00203 是唯一带明确退回理由的,按其复审要求单独处理。
3. **R-00083 解锁收敛**(NativeCore):ADR-040 §7.1 已发布 capability 常量,其原 BLOCKED 依据已被取代;派 NativeCore 会话按新发布物补「生成转换」半边(既有 crate-private 交付无需推倒),然后收口 R-00007 蓝图卡。

### P1 · 各仓下一批实现
4. **CoreEngine**:R-00020(Root ABI 运行时绑定)→ R-00021(platform build)→ R-00022;**R-00021 卡面已写入信任根裁决**(见 §5.3),派活时让它读卡面。R-00266(P2 守护小卡)仍未动。
5. **Server**:R-00271(契约镜像,**已 BLOCKED 于 ADR-045 漂移,裁决已落卡评论**)→ R-00273 起 wave 2..8;13 张 A1-α 卡(R-00270..R-00282)已全部挂 MS-00001。
6. **Runtime**:26 张 C 类卡已逐卡登记 BLOCKED 引用 R-00267/R-00268——**这两张现已 done**,可解冻;复工前先重 pin 架构镜像(上游已前进,identity 变了),再核对各卡验收项是否仍字面成立。T-00138 残项(S03/S06/S07)同理。
7. **Client**:**R-00291(vendor/mirror)与 T-00003(WSS adapter)均已交付**,T-00004 / T-00005 的阻塞**已由总调度核验解除**(镜像内 `replication-envelope.schema.json`、21 个 replication fixture、`ContractBodies.cs`、`ProtocolGate.cs`、`lumio-bin-profile.json` 齐备;`upstreamCorpusPin` 已 `mirrored`;pin 是 commit sha)。解除评论与开工注意已落两卡。顺序:T-00004 → T-00005;T-00009 仍 BLOCKED 于 R-00255(Unity 版本未锁,**不是** R-00268)。
8. **既有缺陷卡**(用户已授权立卡;部分已被并行会话完成,开工前先查状态):R-00284(供应链+生成契约闸门接入 CI,**P0**)、R-00285、R-00286、R-00287、R-00288、R-00289、R-00290、R-00291,以及后立的三张——**R-00292**(Client 架构测试枚举改**版本库口径**消除 worktree 假红,**P0**)、**R-00293**(架构仓 canonical 门禁扩到**域内容**一致性)、**R-00294**(Client WSS 工厂迁出 `Internal/`)。

### P2 · 收尾
9. **R-00269**:V1.5 跃迁批规划 + D-1 ADR 草案(同批:R-00009 枚举对齐、ADR-040..048 Draft 转 Accepted、~~OperationId namespace 发布~~**(已裁决出批)**、D-5 冻结点 tag、trust 两条 P2)。**D-1 方向已定音**(见 §5.1),只差执行。
10. 清理本会话遗留 worktree:`/private/tmp/lge-{lessons,lessons2,nc-audit,p2-confirm,report,s4-adj,handoff}` 与仓内 `.claude/worktrees/distracted-chandrasekhar-edbf54`(其分支已合入)。

## 5 · 今日已定裁决(必须继承,不要重开)

### 5.1 契约面四裁(文档:`docs/plans/2026-08-29-contract-surface-adjudication.md`,内容提交 `8ab4ec4`)
- **D-9 → 立 `LumioBinV1`**:已落地(`packages/binary/lumio-bin-profile.json`,ADR-047)。
- **D-3 / D-4 / D-015 → generated 面升级**:已落地(`ContractBodies.cs` 1018 行 + `bodies.rs` 八类型本体、`ProtocolGate.cs` 可执行 validator、`packages/csharp/*` 全部 `netstandard2.1;net8.0`、ADR-040 §7.1 capability 常量、ADR-048)。
  **关键限制**:validator **只校验「已注册」,不校验角色权限**——架构源无 role→message 权限表,发一个就是发明公共合同并抢跑 D-009,ADR-048 §2 明写到此为止。
- **D-1 方向定音、执行押后**:下行 = ReplicationEnvelope typed body 扩展(FullSnapshot 增 `stateBlocks`、Delta 增 `changedBlocks`,按 mappingSet 声明序以 LumioBinV1 编码、`payloadHash` 绑定);上行 = 新 MessageType `InputCommand` + 独立 input envelope schema。**属基线事件,归 V1.5 批,只跳一次基线。**

### 5.2 Voxel P2 决策门(确认书 `LGE-V1.4-VOX-D-P2-2026-08-29`,`997117e`)
不冻任何数值、不选候选;实测不变量升为 binding(同 cut 三run 字节确定性 / 过期 pin 发布空集 / 可见写后不可恢复 / **缓存键须含 World + Revision** / Host 拥有 DAG-fsync-指针交换);数值轴 adapter-internal 并给定解锁条件;VOX-D-007 差分维持依赖缺口(NativeCore kernel artifact 未发布)。

### 5.3 其他已落卡面的裁决
- **CoreEngine R-00021**:`verify_frozen_plan` **不构成防篡改边界**(sidecar / 重编码 / inputs_digest 全是计划内容的纯函数)。**V1 信任根 = 带外登记的 `build_plan_digest`**;`plan_immutability` 验收必须写成「篡改后因**带外 digest 不匹配**而失败」。已写进 R-00021 卡面。
- **CoreEngine python3**:属**宿主基础设施**,不进 tools.lock、不另立卡(与 git / node 同类,后两者也未登记;一致性即判据)。已落 R-00018 评论。
- **Server ErrorCode 43→53**:所有计数断言改为「**存在性 + 身份**」断言(SchemaId 在册 + BaselineId 相等),**不硬编码任何计数**——计数会随 additive 增补必然腐烂。已落 R-00271 / R-00273 评论。新增 10 个全是内核状态码、无凭据类语义,故「认证失败用 close 1008 不发 Envelope Error」仍成立。
- **Server ADR-045 漂移 5 项**:①镜像纳入 **10 条** replication invalid fixture 全集 + 三个 schema;②`MappingSetHash` = `a805f7c841f708981cc82a93047d7b0c8e6bf923f3dba18e179036741a6d2ea7`(**不是 64 个 0**,ADR-045 明文否决 sentinel);③`length` = **声明上界**(不是字节数);④结构层校验器**必须支持 if/then**,`SchemaValidatorRejectsUnsupportedConstructTest` 换构造;⑤`absences.json` 两条错误记录经**重开 R-00270** 修正(已完成)。**要害**:Delta 合法集 = required + `{gapDetected, resyncReason}`,入站必须接受这两个额外成员。
- **Client**:A1-α 是 **LumioServer 自闭环**的(两个进程都是它自己的),客户端接入是之后的独立跨仓卡;A1-α **全程明文 `ws://127.0.0.1`**,`wss://` 是独立后续卡,故 **R-00253 不是 T-00003 的 A1 前置**。授权 T-00003 内把 `TcpClient` / `NetworkStream` 加进 `eng/BannedSymbols.txt`(现有禁表有洞)。
- **全室 RED 口径**:历史「实现前失败」无实录且锚点不可回退复现的,以**变异式失效证明**替代(单点最小变异→真实失败输出→恢复→通过,两段实录),评论须如实注明「不代表按 TDD 时序留有原始 RED 实录」;变异后仍不失败的空洞测试**不判过**,列差异清单。今日 57 个变异全部被测试杀死,零空洞。

## 6 · 今日沉淀的纪律(`.spec/knowledge/lessons.md`,已合入)

**第一条:「有一份看起来在守护的东西」必须用对照组探针证明它真的会响。** 一天六例四仓:SHA-256 K[28] 常量错、generated manifest 判据哈希经 attribute 转换的字节、LumioClient 与 LumioServer **各自独立**踩中 BannedApiAnalyzers 只认 `BannedSymbols.txt` / `BannedSymbols.*.txt`(叫别的名字则整份禁令静默忽略)、Runtime 供应链脚本不在 CI 准入路径、Client 凭据遏制测试结构上不可能失败。规避五条:
1. 验收证据必须是对照组探针(制造违规→看它红→移除→看它绿),「build 通过 / 测试通过」不构成守护生效的证据;
2. 记录型守护锚定**已提交对象**(`git archive` 优于 `git worktree add`,后者会向被并发编辑的仓写注册记录),并把「产物未被手改」与「与上游同步」**拆成两条独立检查**,后者只报告不 fail;
3. C# 仓 BannedApiAnalyzers 的 AdditionalFile 必须命名为 `BannedSymbols.txt` 或 `BannedSymbols.<描述>.txt`;
4. **探针要打在「判据本身会不会变」上**,不能只打在「改坏了会不会被抓」上(改 manifest / 填假 commit 那类探针全绿,却测不到行尾转换那一面;真正抓住它的是注入 `core.autocrlf=true` 看结论动不动);
5. **闭合到哪一步就只声称到哪一步**(修完行尾层后极易立刻写出「判据是 commit 的纯函数」,而全局 `~/.gitattributes` 的 `* eol=crlf` 仍能压过 `-c core.eol=lf`;正确表述是「`(commit, attribute 栈)` 的函数」)。

**第二条:SPIKE / 审查结论文档里的 P0 行动项必须当轮落单**,否则躺文档过夜被另一会话重复发现(LumioClient 的 SPIKE-HYBRIDCLR-63 §4.7 昨天就记录了闸门空转,行动项没落单,今天两个仓各自重新发现一遍)。

**两条来自执行会话、尚未升格进架构仓的规避(建议你补进 lessons 或 standards)**:
- **任何写进代码注释 / 文档 / 卡面的「由 X 覆盖 / 由 X 保证」声称,必须在同一提交内 grep 验证 X 真实存在**,并把 grep 输出留进证据评论。(它只能证明「X 存在」,证明不了「X 覆盖范围 == 声称范围」——后半段仍需对抗审查。)
- **判据与它的反例测试同一提交内同时诞生**:新增一条 ADR 判据或「由 X 保证」声称时,同提交内必须有一条按该判据构造的**失败**用例。今日两次「同提交内违反自己刚写的规范」(ADR 0007 明写用 toml crate 而实现是手写扫描;ADR 0009 写「被背书者与背书者必须不同源」而同提交让 descriptor 自报字段参与重建)。
- 附:**反例的构造方式决定覆盖面**——往文件追加一个字节必然被抓,替换某字段的值则不会;同一测试意图两种构造盲区完全不同。

## 7 · 需要用户处理(你办不到,别自己动)

1. **美术三方向终裁**(R-00283)。材料已备齐(LumioGame `39f88c9`),推荐 **B > C > A**,含 C 的升位条件与 A 的回看条件。按 ADR 0007 拍板应在出图 + 四关筛选之后;若跳过出图直接定,须新记 ADR 取代 0007 该条。**本卡不要动,留用户。**

### 已裁决,不要再当阻塞项上报

**分支保护 / required check:用户 2026-08-29 决定敏捷期推迟,不加。** 理由(讨论后确认):
- 项目已有两道人工闸补位——所有改动走 PR、高风险面派独立 reviewer(今日七仓皆如此,reviewer 确实抓出过 P0);机器闸在有人工闸时边际价值低。
- 已定裁决降低了传播风险:**上游同步度做成报告项而非硬 fail**(见 §5.3 Runtime 条),架构仓改动不会通过 pin 链打断下游,一次错误合并的传播被降级成通知。
- 重估时机:MVP 收口,或第一次有外部消费者接入。届时 required check 列表里已有可选项(见下),是一次配置动作而非又一轮工程。

**但「把检查接进 CI」是另一件事,仍是 P0,必须做**——两者今天曾被混为一谈:
- 现状是 `check-generated` / `check-contracts` / `nextest` **不在任何 workflow**,Client 的 CI **没有任何 job 跑 `dotnet test`**。即使开了分支保护,required 列表里也没有那些检查可选,因为它们从未在 CI 上运行。
- 后果比「没有闸」更重:今日七仓做出的门禁(K[28] KAT、闸门锚点修复、banned-api 激活、runtime-deps 断言、tools.lock 二进制校验)**只在写它的那台开发机上跑过一次,此后永不执行**。Client 那 3 个架构测试在 macOS 恒红长期无人发现,根因正是 CI 不跑测试。
- **接 CI 不阻断任何人**:PR 页面显示红/绿,照样能合。这正是敏捷期该有的形态——**红了能看见,但拦不住你**。对应卡:**R-00284**(供应链 + 生成契约闸门接入 CI)、**R-00287**(CI 补 dotnet test job),均保持 P0。

## 8 · 工作方式备忘

- **派活**:`mcp__ccd_session__spawn_task` 新开会话(不要用 SendMessage 派新活,那会打断对方);提示词只做**指路 / 立规 / 设禁区**,工作内容本体留在 Workflow 卡上。回报走 SendMessage。
- **公共纪律**(嵌进每份派活提示词):领卡先流转「实现中」;证据只引用已推送 origin 的提交号;测试证据必须是链接执行的真实输出;交付 = 改动清单 + 验证证据 + known gaps + 沉淀落点;公共契约缺口 → 停,标 BLOCKED 上报,不本地绕过;只动本仓文件;做完流转「验收中」,`done` 归总调度;**动手前必开隔离 git worktree**(多会话共用工作区)。
- **一条今天学到的**:同一 worktree 内主会话与 reviewer 并发跑 .NET 构建会互锁(MSBuild 节点复用 + `obj/` 争用)。派审后主会话停手,或给 reviewer 独立 worktree。`.spec/AGENTS.md` 现在只要求「并行 worker 各在独立 worktree」,**没说 reviewer 也要**——这条规则有缺口,值得补。
- **收口门槛**(架构仓):`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && python3 -m py_compile tools/lumio_contract.py && python3 tools/lumio_contract.py validate`;涉基线还须复现 `.github/workflows/repository-policy.yml` 的 Hash/文件检查。**`generate` 需 python3.11**(3.9 会 TypeError),`validate` 3.9 可跑。
- 执行会话拒绝改卡面 description 是**正确**的(`workflow-execute` 的授权例外只覆盖状态流转与证据回写);卡面修订由你做。
