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

## 2 · 当前状态(截至交接时刻)

### 各仓 origin/main

| 仓 | HEAD |
|---|---|
| LumioGameEngineArchitecture | `4303949` |
| LumioNativeCore | `e192459` |
| LumioVoxelEngine | `0466ffd` |
| LumioCoreEngine | `25808f3` |
| LumioGameRuntime | `6a2ab80` |
| LumioServer | `506bca9` |
| LumioClient | `219e1a4` |
| LumioGame | `39f88c9` |

### Workflow 卡态(八室)

| Room | 仓 | 总数 | done | 待核销 | backlog |
|---|---|--:|--:|--:|--:|
| RM-00001 | Architecture | 11 | 9 | 0 | 2(R-00009 需 V1.5 跃迁 / R-00269 V1.5 规划) |
| RM-00002 | NativeCore | 68 | 66 | 1(R-00083) | 1(R-00007 蓝图) |
| RM-00003 | Voxel | 55 | 14 | **26** | 15 |
| RM-00004 | CoreEngine | 40 | 7 | 4 | 29 |
| RM-00005 | Runtime | 34 | 0 | 4 | 29(+1 in_progress R-00138) |
| RM-00006 | Server | 67 | 0 | 8 | 59 |
| RM-00007 | Client | 14 | 0 | 6 | 8 |
| RM-00008 | Game | 2 | 0 | 2 | 0 |

工作项:done 3 / review 5(T-00001/2/6/7/8)/ todo 4(T-00003/4/5/9)。

**待核销清单**(状态 acceptance 或 in_review,等你核验后流转 done):
- NativeCore:R-00083 —— **现已可解锁**(ADR-040 §7.1 已发 capability 常量三形态,其 BLOCKED 依据的原句已被取代)
- Voxel:R-00002 / 00034 / 00041 / 00045 / 00047 / 00066 / 00068 / 00070 / 00071 / 00073 / 00076 / 00078 / 00080 / 00081 / 00093 / 00096 / 00104 / 00116 / 00119 / 00121 / 00134 / 00135 / 00136 / 00137 / 00142(25 张 in_review)+ **R-00203**(SV-α 判 3/4,第 1 项不通过:mvp-review.md 未逐张覆盖全体 P0 卡,需在绿基线上补一轮含全 P0 对照的复审)
- CoreEngine:R-00265 / R-00016 / R-00017 / R-00018
- Runtime:R-00112 / R-00127 / R-00131 / R-00133
- Server:R-00186 / R-00188 / R-00190 / R-00200 / R-00209 / R-00260 / R-00270 / R-00272
- Client:R-00001 / R-00019 / R-00031 / R-00065 / R-00067 / R-00256(+ 5 个 review 态工作项)
- Game:R-00259(模块脚手架设计)/ **R-00283(美术比稿——用户终裁,不要动)**

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
2. **核销 50 张待核销卡**。方法:每仓派一个 QA 会话(写的人 ≠ 判的人),按「锚点在 origin + 门禁实跑 + 验收项逐条判定 + 每卡一条证据评论」核销,卡保持验收中,**你复核后流转 done**。Voxel 26 张量最大,可拆两道(奇/偶卡号,今日用过该切法,互斥且好核对)。
3. **R-00083 解锁收敛**(NativeCore):ADR-040 §7.1 已发布 capability 常量,其原 BLOCKED 依据已被取代;派 NativeCore 会话按新发布物补「生成转换」半边(既有 crate-private 交付无需推倒),然后收口 R-00007 蓝图卡。

### P1 · 各仓下一批实现
4. **CoreEngine**:R-00020(Root ABI 运行时绑定)→ R-00021(platform build)→ R-00022;**R-00021 卡面已写入信任根裁决**(见 §5.3),派活时让它读卡面。R-00266(P2 守护小卡)仍未动。
5. **Server**:R-00271(契约镜像,**已 BLOCKED 于 ADR-045 漂移,裁决已落卡评论**)→ R-00273 起 wave 2..8;13 张 A1-α 卡(R-00270..R-00282)已全部挂 MS-00001。
6. **Runtime**:26 张 C 类卡已逐卡登记 BLOCKED 引用 R-00267/R-00268——**这两张现已 done**,可解冻;复工前先重 pin 架构镜像(上游已前进,identity 变了),再核对各卡验收项是否仍字面成立。T-00138 残项(S03/S06/S07)同理。
7. **Client**:**R-00291(vendor/mirror 消费通道)是硬前置**——T-00004/T-00005 在它落地前不开工。顺序:R-00291 → T-00003(WSS adapter,卡面已补三条必做项)→ T-00004 → T-00005。
8. **新立的 8 张既有缺陷卡**(用户已授权,尚未派活):R-00284(供应链+生成契约闸门接入 CI,**P0**)、R-00285、R-00286、R-00287(CI 补 dotnet test,**P0**)、R-00288、R-00289、R-00290、R-00291。

### P2 · 收尾
9. **R-00269**:V1.5 跃迁批规划 + D-1 ADR 草案(同批:R-00009 枚举对齐、ADR-040..048 Draft 转 Accepted、OperationId namespace 发布、D-5 冻结点 tag、trust 两条 P2)。**D-1 方向已定音**(见 §5.1),只差执行。
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

1. **仓库设置:分支保护 / required check**。CoreEngine 与 Client 的 `main` 都未启用,今日做出的门禁全部**只在本地跑、红了也不阻断合并**。CoreEngine 会话连报四次,后果已具体化:`check-generated` / `check-contracts` / `nextest` 不在任何 workflow(规格 §20.1「CI 重新生成并要求工作树零差异」无机器执行者)。R-00284 / R-00287 两张卡能补 CI 调用,但**「必须绿才能合并」只有仓库设置能给**。
2. **美术三方向终裁**(R-00283)。材料已备齐(LumioGame `39f88c9`),推荐 **B > C > A**,含 C 的升位条件与 A 的回看条件。按 ADR 0007 拍板应在出图 + 四关筛选之后;若跳过出图直接定,须新记 ADR 取代 0007 该条。**本卡不要动,留用户。**

## 8 · 工作方式备忘

- **派活**:`mcp__ccd_session__spawn_task` 新开会话(不要用 SendMessage 派新活,那会打断对方);提示词只做**指路 / 立规 / 设禁区**,工作内容本体留在 Workflow 卡上。回报走 SendMessage。
- **公共纪律**(嵌进每份派活提示词):领卡先流转「实现中」;证据只引用已推送 origin 的提交号;测试证据必须是链接执行的真实输出;交付 = 改动清单 + 验证证据 + known gaps + 沉淀落点;公共契约缺口 → 停,标 BLOCKED 上报,不本地绕过;只动本仓文件;做完流转「验收中」,`done` 归总调度;**动手前必开隔离 git worktree**(多会话共用工作区)。
- **一条今天学到的**:同一 worktree 内主会话与 reviewer 并发跑 .NET 构建会互锁(MSBuild 节点复用 + `obj/` 争用)。派审后主会话停手,或给 reviewer 独立 worktree。`.spec/AGENTS.md` 现在只要求「并行 worker 各在独立 worktree」,**没说 reviewer 也要**——这条规则有缺口,值得补。
- **收口门槛**(架构仓):`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && python3 -m py_compile tools/lumio_contract.py && python3 tools/lumio_contract.py validate`;涉基线还须复现 `.github/workflows/repository-policy.yml` 的 Hash/文件检查。**`generate` 需 python3.11**(3.9 会 TypeError),`validate` 3.9 可跑。
- 执行会话拒绝改卡面 description 是**正确**的(`workflow-execute` 的授权例外只覆盖状态流转与证据回写);卡面修订由你做。
