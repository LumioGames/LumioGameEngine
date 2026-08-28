# 2026-08-28 · P0 Gate 交付结果与待裁决项

> 交付会话：`lumiogameenginearchitecture-6a`。基线 `LGE-V1.4-2026-08-27` 未变。
> 本文分两部分：**已完成**（三张卡，已合入 main）与**待架构所有者裁决**（**十一项**：D-8 已修复落地并移出；新增 D-10 / D-11 / D-12）。

> **【状态更新 · 2026-08-28 · 会话 `practical-lehmann-145b88`】**
> 本文正文是写作当时（`origin/main = a4a7956`）的快照，**正文不改写**，此后变化统一记在这里：
> - `origin/main` 已前进至 **`b8f8c50`**，依次叠加：`bcc8eb9` K[28] SHA-256 修复、`4f36d92` 本文、`7bdad78` ADR-041 normalization 可机器读（D-8）、`b8f8c50` Root ABI consumers 登记。四次 Architecture Policy 均 success。
> - **§三.4 的「尚未合入 main」与其禁令已失效**：K[28] 修复已在 main，该实现现可用于核对 Golden / digest。详见该条的更新标注。
> - **D-8 已落地并经独立复核**：仅凭已发布 profile 数据（不读 ADR 正文）的净室实现，8/8 Golden 的 bytes 与 sha256 全中；此前静默算错的两条 permutation 用例已转通过。
> - **D-2 的排序约束③（「卡② 不得在 K[28] 合入前落地」）已自动解除。**
> - **2026-08-28 收工状态**：`origin/main = c712ff4`，CI completed/success。此后又叠加 `c712ff4` R-00005（ADR-042 Signature/Trust Profile）。
> - **三张 P0 Gate 全部关闭**：R-00003 `44f617b` / R-00004 `a4a7956` / R-00005 `c712ff4`，卡状态均为 `done`；R-00258 亦 `done`。**`LumioCoreEngine` R-00015 的前置至此齐备**（但仍需先升 architecture lock，见 D-2）。
> - **D-8 已修复并合入**（`7bdad78`）：`digestDomains[].normalization` 现为可执行声明，生成器与门禁执行它本身；三个下游仓各自用零仓库知识的净室实现复核 8/8，剥空后精确退回 6/8。**该项已从待裁决移除。**
> - 卡状态流转（D-7）在用户全权授权下执行，**独立审查仍未发生** —— 每张卡的关闭评论都如实记录了这一点，未写成「审过了」。
> - 其余待裁决项（D-1、D-3、D-4、D-5、D-6、D-9）**仍未裁决**。

## 一、已完成并合入 `origin/main`

| 卡 | 提交 | PR | CI | Workflow 状态 |
| --- | --- | --- | --- | --- |
| R-00003 `[LGE-GATE-P0-001]` Root ABI Bundle | `44f617b` | #1 MERGED | success | acceptance |
| R-00258 TransportProfile WebSocket 档登记 | `a738524` | #2 MERGED | success | acceptance |
| R-00004 `[LGE-GATE-P0-002]` Canonical/Digest Profiles | `a4a7956` | #3 MERGED | success | acceptance |

`origin/main = a4a7956`，收口门槛全绿：spec-lint OK / 13 tests pass / **174 fixture 0 失败** / baseline hash OK / `packages/` 与 generate 一致（12 artifacts + rootAbi + canonicalDigest）/ cargo check + clippy `-D warnings` / dotnet build 0 warning / C# no-native grep 无匹配。

### 各卡实质

- **R-00003**：核对判定**不可关闭**——V1.4 已发布的六类 Artifact 无一字节来自 `native-managed-abi.schema.json`。补 ADR-040（Draft）冻结 compiler 身份、输入集合、`typeRef` 19 条产生式 → C/C#/Rust 映射、`linux-x86_64-glibc` 布局、输出文件名；发布 `packages/abi/`（C header + 布局 Golden + Rust/C# 绑定）。三语言 layout 实测通过（clang `-Werror` / cargo const 断言 / `Marshal.OffsetOf` 12/12）。
- **R-00258**：核对判定**无缺口**。WSS 落在既有 `reliability:"Reliable"` 一档，属 D-004 的 adapter 级选择。产出结论文档 + 登记记录 + 一正一反 Fixture。**D-009 / D-011 未触碰。**
- **R-00004**：关闭 ADR-018 留下的四处摘要歧义（`artifactSetDigest` 自引用、index vs set digest、`targetProfileDigest` / `capabilitySetDigest` 输入未定义、规范形式只是散文）。ADR-041（Draft）定义 `CanonicalJsonV1` 与五个摘要域，发布 8 条**自校验** Golden——`validate` 从 input 重新规范化并重新取摘要，Golden 不会腐烂成谎言。

---

## 二、待裁决（按阻塞严重度排序）

### D-1 · 下行状态载荷编码 + 上行输入承载 —— **最高优先级，直接挡住 MVP 验收 1**

**提出**：`lumioserver-2d`（R-00260 设计）。**独立复核**：本会话 + `lumioclient-a9`。

现有公共面**没有任何字段承载世界状态**：

- 8 条正向 replication fixture 的 body 键集恰好等于 `_REPLICATION_BODY_REQUIRED`，全是标识与版本：`FullSnapshot` = `snapshotId/tickId/sessionRevisionVector/schemaEpoch/mappingSetHash`；`Delta` = `baseSnapshotId/fromRevision/toRevision/mappingSetHash/confirmationSequence/tombstones`。
- 本会话另查一层：`replication-mapping.schema.json` 冻结的是**哪些字段复制**（source/target component+field、delivery、visibility、prediction、quantization），Envelope 只带 `mappingSetHash`。**映射描述符冻结了，被映射值的线编码整层未定义。** 所以这不是「body 少一个字段」，补字段不能了结。
- ADR-028（Accepted）的 Alternatives 明文否决 free-form payload，堵死各仓自行加字段的路。`lumioserver-2d` 曾裁决加私有字段 `mvpAuthorityPayload`，经三路对抗审查判为越界并**已整体撤销**，落地断言升级为「出站 body 字段集恰好等于必填集」——本仓要求的「缺口→停、不本地绕过」被正确执行。
- Client 侧佐证（`lumioclient-a9`）：`replica` 只有 `RequiresResync` 类阶段状态，**没有任何被映射值的解码路径**；`mappingSetHash` 能校验映射集一致，但校验通过后没有东西告诉客户端怎么把字节还原成组件值。

**后果**：A1 已被 `lumioserver-2d` 拆成 **A1-α**（WSS 握手 → admission → FullSnapshot → BaselineAck → revision 前进 → DeltaAck → 断连重连 Full Resync；只依赖已冻结面，可交付）与 **A1-β**（「第二个客户端看见方块被挖」，**BLOCKED**）。A1-β 正是 MVP 计划 §2 目标与 §6 验收 1。

**需要决定**：是否立 ADR 冻结状态载荷编码（下行）与客户端输入承载（上行）。这是 MVP 主线的硬前置。

### D-2 · `architecture.lock` 升级 —— CoreEngine 物理上无法开始消费

**提出并算好数字**：`lumiocoreengine-93`。**本会话独立复核，数字逐项一致**：

```
baselineId: LGE-V1.2-2026-08-27   commit: 2d7980d95b16
requiredPaths: 131
top prefixes: {'.spec': 24, 'fixtures': 72, 'ids': 2, 'schemas': 31, 'tools': 2}
any packages/ path: False
```

CoreEngine 不读架构仓工作区，只读钉死的只读镜像。`packages/abi/` 与 `packages/canonical/` **不在 requiredPaths 内**；且其 `sync-architecture.sh` 投影规则只认五个前缀，`packages/**` 会被判为不可投影直接 fail。**本会话此前「BaselineId 未变、直接 pull main 即可」的判断对 CoreEngine 不成立，已向其撤回。**

升级代价 —— **`lumiocoreengine-93` 已更正其初版数字**。初版让 pin 取 `1957c50`，实测该提交**零个 `packages/` 路径**；`packages/abi` 最早出现在 `44f617b`，`packages/canonical` 最早在 `a4a7956`，因此 **pin 必须 ≥ `a4a7956`**。重算后：

| 项 | 卡① 现五前缀（不含 packages/） | 卡② 加 packages/ 后 |
| --- | --- | --- |
| requiredPaths（按 `c712ff4` 重算） | 131 → **277**（新增 147 / 删除 1 / 共有路径漂移 67） | +5（consumers 收敛后的 `packages/`，非全量 62） → **282** |
| 镜像文件数 | 133 → **279** | — |

**已备两张卡草稿（带精确行号、陷阱与验收项），可直接落单：**

- **卡①「升级 lock 到 V1.4」**：改 `tools/sync-architecture.sh:50-51` 两个常量 + `--update-lock` + 镜像目录改名。三个陷阱：唯一删除项 `fixtures/invalid/processor-read-write-conflict.json` 必须与 140 个新增在**同一次** `--update-lock` 里处理；改脚本会变其 SHA，而 `compilerSha256` 与之强耦合，必须重生成 `generation-record.json`；镜像内 `ids/index.json` 的 baselineId、lock 的 `architectureBaselineId`、镜像目录名三处必须同步。**卡面必须写明「本卡不解除任何下游阻塞」** —— R-00015 要的 ContractTypes / ErrorCode / Capability / Schema registry 全在 `packages/` 下，而卡①的五前缀投影不含它。
- **卡②「扩展投影规则纳入 `packages/`」**：改 `project()`（63-69）、`--update-lock` 枚举表（197-198，硬编码五前缀）、守卫串（185）。属 **R-00012 文件集**，不得由 R-00015/R-00018 顺手改。两个待澄清项交架构侧定：`packages/rust/Cargo.lock` 是否进镜像（它会漂移，且架构仓 `generate` 会 rmtree 掉这个已入库文件）；`packages/.gitignore` 与 `packages/README.md` 是契约还是仓务文件（进镜像后架构仓改 README 就会打断 CoreEngine 门禁）。

**三条排序约束（硬）：**

1. **①→② 严格串行**，不可并行 —— 两张卡都改 `sync-architecture.sh`，都触发 `compilerSha256` 重生成。
3. **卡② 不得在 K[28] 修复合入 main 之前落地**，否则会把一个**已知算错摘要**的 sha256 实现字节级冻结进 CoreEngine 只读镜像，并用它的 SHA-256 登记进 lock。若 K[28] 能在卡①之前合，两张卡都取修复后的 commit 最干净；否则卡② 会顺带再动一次 pin，须在卡面写明这是预期行为而非范围蔓延。
4. **ADR-040 / ADR-041 仍是 Draft**（见 D-6）。把 Draft 状态的公共构造冻进只读镜像并钉进 lock，等它们转 Accepted 时会再触发一次 300+ 条路径重新登记。**是否等转 Accepted 再做卡②，请一并裁决。**

**卡② 范围已因 `b8f8c50` 的 consumers 登记大幅收敛**：`rootAbi.consumers = [LumioCoreEngine, LumioNativeCore]`，而 12 个六类 artifact 的 consumers 不含 CoreEngine。因此 CoreEngine 只需镜像 **3 条**（`packages/abi/lumio_core.h`、`packages/abi/root-abi-bundle.json`、`packages/index.json`），不是 62 条；requiredPaths 合计从 332 降到 **274**。`packages/rust/Cargo.lock`（已在 `b8f8c50` 移除）与 `packages/.gitignore` / `README.md` 两个待澄清项因此自动消解。

**`lumiocoreengine-93` 的设计改进，本会话背书**：卡② 的投影规则**不要硬编码 `packages/` 前缀**，而是**按 `packages/index.json` 的 consumers 关系收敛**，验收项写成「镜像内 `packages/` 文件集 == `index.json` 中 consumers 含 `LumioCoreEngine` 的条目所指文件集，多一个少一个都 FAIL」。这把镜像范围变成上游数据的函数，上游以后增删消费面时下游镜像自动跟随，不必再开卡改投影规则。

**由此暴露的 `consumers` 缺口已补齐（本次收工提交）**：`packages/index.json` 原先只有 `rootAbi` 节带 `consumers`，`canonicalDigest` 与 `trust` 两节没有，收敛规则对它们无从判断，实现方只能猜 —— 而「猜」正是这套设计要消灭的东西。现按 ADR-041 / ADR-042 的 Owner 行补为 `["LumioCoreEngine"]`。CoreEngine 侧的镜像因此是 **5 条**而非 3 条。

**pin 目标**：现在应取 **`c712ff4`**（三张 Gate + K[28] + D-8 全在其中）。`lumiocoreengine-93` 将按此重算两张卡的数字。

**需要决定**：是否落这两张卡草稿；上面三条排序约束怎么排。（`consumers` 补齐一项已在收工提交中完成，不再需要裁决。）**并见 D-10：卡面修订必须与这两张卡同批，否则 R-00015 仍不可执行。**

> **注意**：即使 R-00005 关闭、三张 Gate 齐备，CoreEngine 侧仍要先过这道坎才能真正开工。不要以为 Gate 一关下游就自动通了。

### D-3 · generated 面能力边界（含 ADR-022 落差）

**提出**：`lumioclient-a9` + `lumiogameruntime-22`（两仓独立路径撞到同一堵墙）。**本会话确认属实**。

`packages/csharp/` 六个包是 **catalog-only**：发的是字段表、目录、状态机转移表；只有 ContractRuntime 有真实现（SHA-256 + hash chain + 有界缓冲）。**没有 `ReplicationEnvelope` 类型本体，也没有可执行的 Protocol/Permission Validator** ——与 ADR-022「Active 消息必须过架构源工具链生成的 Protocol/Permission Validator」的意图存在落差。

`lumiogameruntime-22` 本机对 `origin/main`（8 文件 437 行）复核：`ConfigTable` / `ProcessorDescriptor` / `TxnJournalRecord` / `CommandLogRecord` / `WalRecordEnvelope` / `EntityIdentity` / `ReplicationEnvelope` / `SessionRevisionVector` 的定义数**全为 0**。Runtime 有 8 条验收项踩在这上面（R-00138 S03/S06/S07、R-00139 S04/S06、R-00141 S02、R-00149、R-00150 S04），6 张 wave 卡已判「前置未满足、未开工」，未用手写 fixture 或自研 validator 顶替。

**需要决定**：generated 面只给目录表，还是给类型本体 + validator/builder。**若裁决是 catalog-only，请一并说明「不得自行发明公共合同」与「必须调用 generated validator」这对约束下游怎么解 —— 尤其 ID ordinal 的权威来源在哪里。**

### D-4 · 面向 `netstandard2.1` 消费方的发布形态

`packages/csharp/*/*.csproj` 六个全是 `<TargetFramework>net8.0</TargetFramework>`（本会话实测）。LumioClient 核心冻结在 `netstandard2.1`（Unity 与 .NET Host 的共同面），**编译期引用不了**。这横跨架构源与至少三个下游仓。

**需要决定**：是否发多目标（`netstandard2.1;net8.0`），或另立面向 Unity 的发布形态。

### D-5 · 下游可 pin 的冻结点

`compilerHash` 今日在 40 分钟内变了三次（三笔合并各重新 generate 一次），三个仓的快照数出三个不同行数。下游没有可引用的稳定点。当前 `origin/main = a4a7956` 已稳定，但 R-00005 会再动一次。

量化（`lumiogameruntime-22`）：`compilerHash` 一天四变 `99a786e7` → `3b4230a3` → `2545ab1c` → `01049476`（当前 main）。**请给 tag 或 artifact digest，不要 branch name。**

附一条改进建议（`lumiogameenginearchitecture-53` 提出）：`compilerHash` 现在把生成器源码与产物身份绑死（SHA-256 的 K 表就写在 `lumio_generate.py` 里，而 `compiler_hash` 正是对该文件算的），**工具改一行就让全下游 churn**。可考虑拆成「生成器版本号（人工递增）+ 内容哈希」。

**新增一条同族问题（`lumiocoreengine-93` 提出，本会话确认）**：`tools/lumio_contract.py` 与 `tools/lumio_generate.py` **都在 CoreEngine 契约镜像的 required paths 里**（`tools/` 是五个投影前缀之一）。含义是**架构仓每次改生成器或校验器，都会打断 CoreEngine 的 `check-contracts`，即使契约本身没变** —— 工具实现变了、契约没变，下游门禁照样红。`tools/lumio_generate.py` 今天漂移了三次（K[28]、D-8、R-00005），频率远高于 schema/fixture，这条耦合是真实且高频的。

同族的还有 `packages/rust/Cargo.lock`（每次 `generate` 必被 `rmtree` 删掉，已在 `b8f8c50` 移除）。**共同的口径问题是：契约镜像收什么、不收什么 —— 凡是「会被上游工具重新生成或删除」的文件，都会把下游门禁绑在上游的操作纪律上。** 建议一次性划清。

**需要决定**：给下游一个冻结引用点（tag / 版本号 / artifact digest）；是否采纳 `compilerHash` 拆分；以及 `tools/**` 是否真该进契约镜像。

### D-6 · 两张 Draft ADR 转 Accepted

- **ADR-040**：`lumio_handle_t` = 16B(4+4+8)/align 8、`lumio_buffer_t` = 24B(ptr+u64+u64)/align 8、root 与 api table 各 16 字节表头、`structSize >= 派生最小值`。**16 字节表头是从 V1.4 ABI 文档两张表的 `structSize` 反推**（48 = 16+4×8、32 = 16+2×8，两表精确吻合），依据与被拒替代已写进 ADR。
- **ADR-041**：`CanonicalJsonV1` 与五个摘要域，外加 §4 的 `normalization` 可执行声明。
- **ADR-042**：`LumioSignatureV1` —— 域分离 preimage 布局、`keyId` 派生规则、五级拒绝优先级。

三者都是**首次冻结的公共构造**，需架构所有者确认后随下一基线转 `Accepted`。注意 D-2 的排序约束 3：把 Draft 状态的公共构造冻进 CoreEngine 只读镜像并钉进 lock，等它们转 `Accepted` 时会再触发一次全量重新登记。

### D-7 · 卡状态 `acceptance` → `done`

`lumiocoreengine-93` 指出：下游卡的「精确前置」是按**卡状态**判定的，不是按 git。R-00003 / R-00004 停在 `acceptance`，R-00015 的前置就仍是 3 缺 3 而非 3 缺 1。

**本会话不自行流转**，两个理由：① 这三张卡的交付是本会话所写，而其 Known gaps 明确请求「CODEOWNERS 补一次独立审查」——一边要求独立审查一边自判 done，闭环是假的；② 本轮纪律要求卡状态流转须逐次确认。

**需要决定**：是否先做独立审查再流转，还是直接流转。

### D-8 · ~~已发布 Profile 的 pre-sort 规则不是机器可读的~~ —— **已修复，合入 `7bdad78`**

**提出**：`lumioclient-a9`（端到端实测）。**本会话独立复现，结果一致。**

`canonical-digest-profile.json` 的 `canonicalForm` 块完整描述了 `CanonicalJsonV1`，`omitMembers` 也是结构化的；但 **per-domain 的 pre-sort 规则只存在于 `sortRule` 自由文本与 ADR-041 正文散文里**。后果实测：

```
只按 canonicalForm 块独立实现 →  6/8 复现，2 条 permutation 用例 BYTES-DIFF，且【不抛任何异常】
加上域内 pre-sort（entries 按 path、capabilities 按码点，在规范化之前）→ 8/8 bytes + sha256 全中
```

失败是**静默**的：一个照着 profile 实现的下游会得到错误摘要而毫无提示，一路带到生产。这直接违背 ADR-041 「Golden 自校验、下游只读发布物即可合规」的意图 —— 摘要配方（无 prefix/salt/length framing）本身已被 8/8 验证成立，缺的只是把排序规则从散文变成数据。

**建议方案**（`lumioclient-a9` 提出，本会话认为可直接采纳）：在 `digestDomains` 每项增加机器可读的 `normalization`，例如
`[{"path":"entries","sortBy":"path","order":"Ascending"}]` / `[{"path":"capabilities","sortBy":"$self","order":"CodePointAscending"}]`，
并把 `artifactSetDigest` 的「省略自身成员」也表述成同一套结构化规则（现为 `omitMembers`，可并入）。

**已修复，无需裁决。** `digestDomains[].normalization` 发布为可执行步骤序列（`path` / `op` / `by` / `collation`），**生成器与门禁执行这份声明本身**，不再走硬编码分支 —— 发布的数据可证明就是实际跑的东西。`sortRule` 降级为人读注释；空数组表示「无规则」，与「作者忘了写」可区分，后者由新增负例 `canonical/missing-normalization` 抓。

纯增量：8 条 Golden 的 `sha256` 与 `canonicalBytes` 逐条不变，已正确实现 sort 的消费方不受影响。三个下游仓各自用零仓库知识的净室实现复核：修复后 8/8，剥空 `normalization` 后精确退回 6/8 且失败集合不变。

### D-10 · R-00015 卡面自相矛盾 —— 三张 Gate 全关也没有解锁它

**提出**：`lumiocoreengine-93`。**本会话独立复核，逐条属实。**

R-00015（LCE-P0-003）的目标是「用一个只读、可审计的运行时 crate 消费架构源 ContractTypes / ErrorCode / Capability / Schema registry」，实现要求写明「generated 文件**逐字节来自上游制品**」。而同一张卡面又写着：

> 本单固定消费 `LGE-V1.2-2026-08-27` / `2d7980d95b163404e33cc6212db13ac948d30d40`。**不得因为 README 或相邻仓已经出现 V1.4 而静默升级** architecture lock、Schema、Fixture 或生成输出。

实测那个被钉死的基线上有什么：

| | `2d7980d9`（卡面钉死的消费基线） | `c712ff4`（当前 main） |
| --- | --- | --- |
| `packages/` 路径 | **0** | 63 |
| `packages/abi/` | **0** | 2 |
| ADR-040 / 041 / 042 | **0** | 3 |

**卡面要求实现方去消费一个在其指定基线上根本不存在的东西。** 照卡面做只能交空壳；违反卡面则是静默升级 lock，而那是卡面明文禁止的。实现方无法自行解决。

真实状态因此是：R-00015 的五个前置（R-00012 / R-00013 / R-00003 / R-00004 / R-00005）**全部 done**，但它**依然不可执行** —— 缺口从「Gate 未关」变成了「卡面基线与交付物不自洽」。

**需要决定**：解法必须三件一起 —— ① 升 lock（D-2 卡①）；② 按 consumers 收敛纳入 `packages/`（D-2 卡②）；③ **同时修订 R-00015 卡面那句消费基线**（`2d7980d9` → 升级后的 pin）。**第三件最容易被漏**：前两张落地后若不改卡面，R-00015 还是卡着，只是理由变了。R-00012 卡面有同款句子，一并检查。

---

### D-9 · `ADR-010:20` 的「the same canonical codec rules」当前无指向物

**提出**：`lumiogameruntime-22`（核 ADR-041 适用范围时发现）。**本会话逐条复核，全部属实。**

```
ADR-010:20  "Domain payload schemas are owned by Voxel/Game and reference the same canonical codec rules."
$ git grep -lniE 'messagepack|msgpack' -- schemas .spec/decisions packages   → 零命中
$ endianness 命中全在 native ABI 语境（ADR-006 / ADR-020 / ADR-040），持久化域零命中
```

架构源当前冻结的 canonical 只有 `CanonicalJsonV1`，它是 **JSON 文本**形态（`encoding = AsciiEscaped`）。而 ADR-035 已经把 voxel payload 的字节冻得很死 —— `chunkOrder = CoordXYZAscending`、`byteOffset`/`byteLength` 连续升序且求和等于 `payloadLength`、`payloadHash` = canonical bytes 的 SHA-256、`determinism = SameCutSameBytes`，并明文「Two encodes of the same cut that differ in bytes are a fatal contract violation」。

**二进制 voxel chunk payload 显然不可能走 `AsciiEscaped` 的 JSON。** 所以 ADR-010 把 domain payload 指向了一套**并不存在**的二进制 canonical codec 规则。ADR-035 定义了域内排序与偏移，但**没有定义 primitive 层字节布局**（整数宽度/端序/变长编码/字符串与字节数组长度前缀）。

**这同时证明两件事**：① Snapshot payload 字节是**公共的**（ADR-035 明文 "every conforming encoder"），不能按仓内私有编码处理；② 公共的东西却没有权威规范。

**需要决定**（`lumiogameruntime-22` 的问法，本会话背书）：
> 补一个二进制 canonical profile（与 `CanonicalJsonV1` 并列），还是把 `ADR-010:20` 这句引用改掉、明确 domain payload 的 primitive 编码归各域 ADR 自定？

前者定了，Runtime 的 `Directory.Packages.props` 里 `MessagePack 3.1.8` 的去留也跟着定；后者定了，R-00141 需要一份「Runtime 持久化域 primitive 编码」的域级 ADR 才能开工。**两条路 Runtime 都能走，但不能没有裁决就自己选一条**——那正是发明公共合同。

**附带**：`snapshot-header.checksum` 的 B 档权威 `CHECKSUM_DOMAIN.md` 只有一行，**没有 Golden、没有 domain tag，也没说与 `checksum` 并列的那个 required `hash` 字段的口径**。比 A 档薄得多，下游照它实现仍有歧义空间，建议一并补。

### D-11 · R-00009 需要 BaselineId 跃迁 —— 一张 P1 卡承担不了

**本会话实测的漂移**（与卡面描述一致）：

```
schemas/target-profile.schema.json  loadBackend = ["StaticLinked", "DynamicLibrary"]
架构正文 v1.4 §10                    LoadBackend = DynamicLibrary / StaticLink / NoNative
schemas/target-profile.schema.json  packaging   = object{libraryFileName, debugSymbolFormat, archiveFormat}
架构正文 v1.4 §10                    PackagingProfile = LooseFiles / Archive / EmbeddedInApp
```

三处差异属实：`StaticLinked` vs `StaticLink` 是**改名**；Schema **缺 `NoNative`**（而 `PureHeadless` preset 明文声明它）；`packaging` 在 Schema 里是打包细节对象、在正文里是三值枚举 —— **两者是不同概念，不只是拼写**。

**为什么本会话不动它**：`schemas/README.md` 的变更规则明写「Changing a required field, **enum** … requires … **a new baseline id**, and synchronized repository mirrors」。改 `loadBackend` 枚举与把 `packaging` 改成 `PackagingProfile` 都是 enum 变更。而 `LGE-V1.4-2026-08-27` 被 `generated-contract-artifact.schema.json` 的 `const`、CI 的多处 grep、七仓镜像与 lock、以及所有在途实现卡的卡面共同钉死。**一次基线跃迁是七仓事件**，不该由一张 P1 卡或一个未获授权的会话发起。

**建议**：把 R-00009 并进一次 **`LGE-V1.5` 跃迁**，与①ADR-020 refine 统一三轴命名、②Schema/Fixture/`targetProfileDigest` Golden 同步（ADR-041 的 `TargetProfileV1` 域已就绪可直接复用）、③七仓镜像同步、④**五张 Draft ADR（040–044）一并转 Accepted** 同批规划。这样**只跳一次基线**，而不是为本卡跳一次、再为 Draft ADR 跳一次。

### D-12 · R-00257 前提不成立 —— Voxel P2 决策门的测量从未执行

**本会话实测**（LumioVoxelEngine HEAD `13d515f`）：

- `docs/evidence/decision-gates/` 的 8 个 `approvalStatus`：4 个 `approved`（VOX-D-001..004，已由 D-013 确认书裁决），**4 个 `blocked`（VOX-D-005..008）**。
- **8 个证据文件全部提到 `link.exe`**。R-00061/R-00062 的交付评论原文：`cargo test -p lumio-voxel-test-support --all-features` → **exit 101, linker `link.exe` not found**；只有 `cargo build --lib` 与 `rustc --crate-type rlib` 通过。
- blockedReason 原文含 **`cargo test unlinked (no link.exe)`**。
- VOX-D-005 证据文档自述「records candidates and a **measurement seam**… does not freeze numeric defaults」，其 §4 标题是 **"Measurement plan (harness seam)"** —— 是计划，不是结果。

R-00257 卡面写「R-00061..R-00064 **已完成测量**、停在验收中等待架构所有者裁决」，**与事实不符**。卡面要求「以 VoxelEngine benchmarks 测量缝的证据为据」逐门裁决 —— **没有测量就没有据**，因此本会话**不签发确认书**，也不做「D-014 临时默认反正说这些 stay implementation-level、照抄一遍」的省事处理：那会把未经测量的结论伪装成经过测量的裁决。

**真正的阻塞点不在架构所有者**，而在 VoxelEngine 的测量从未链接执行 —— 即本仓 `lessons.md` 已收录的「Windows 缺 `link.exe`，2.4 万行代码只过了 cargo check」那条。2026-08-28 的 W0 派活提示词已要求在 macOS 本机首次真实跑通，正是为了打破它。

**需要决定 / 解除条件**：① VoxelEngine 在可链接宿主上真跑 decision-gate 测量，产出真实数字与 host triple；② 四张卡的 evidence §4 从 plan 变成 results；③ 届时按 D-013 模式逐门裁决。在此之前 R-00257 应停留 backlog，**并修正其卡面「已完成测量」一句**，否则下一个接手的人会重复这次核对。

---

## 三、非阻塞但应处理

1. **本会话误判「R-00141 可开工」，已撤回（本会话缺陷）**：我据 A 档结论告诉 `lumiogameruntime-22` 其 R-00141（canonical encode/decode）可以开工。**错了** —— 我按卡名推断，没读卡面。R-00141 要的是二进制线编码（UInt64/端序/长度策略/MessagePack primitive），而 `CanonicalJsonV1` 是 JSON 文本形态，层次错配；且 ADR-041 的 Owner 行列的消费方是 CoreEngine 的 manifest/platform/runtime-verifier，本就不含 Runtime。A/B/C 三档划分本身成立，错的是我把它套到一张没读过的卡上。`lumiogameenginearchitecture-53` 半小时前得出过同一错误结论并已撤回 —— 两个架构仓会话独立踩同一个坑，说明 ADR-041 的适用边界需要写得更显眼。
2. **R-00258 文档措辞越界（本会话缺陷）—— 已修正**：§0 曾写「现有 Envelope…已经完整覆盖 MVP A1 所需的公共语义」，但 12 项核对**只覆盖传输语义**。那句话把「WebSocket 档不需要公共契约变更」这个正确结论，扩写成了「A1 不需要公共契约变更」这个错误结论，由 `lumioserver-2d` 读准并上报。已收窄为「就传输面而言」，并新增 §6b 明确列出未覆盖、但对 A1 是硬前置的两项（即 D-1），附范围更正说明。
3. **`forbiddenDependents` 命名误导**：字面是「依赖它的人」，实际语义是「本 artifact 不得依赖这些仓」。三个下游仓因此误判为「自相矛盾、无法机器判定」并一度停工；经本会话给出 validator + fixture + schema 三处证据后，`lumioserver-2d` 与 `lumioclient-a9` 均已独立复核并撤回。**只当命名改进项，不当缺陷**（改名会动已发布 Schema 枚举，需权衡）。
4. **SHA-256 `K[28]` 错常量**：`packages/rust/lumio-gen-contract-runtime/src/sha256.rs` 第 29 个常量为 `0xc6eabbdc`，FIPS 180-4 规定 `0xc6e00bf3`——该实现**对任意输入都算出错误摘要**。修复在 `fix/sha256-k28-round-constant` @ `c862682`（`lumiogameenginearchitecture-53` 交付，已推 origin，基于 a4a7956，`git merge-tree` 实测无冲突，28 files / +28 / −28）。~~**尚未合入 main**，合并权在用户。在它落地前不得用该实现核对任何 Golden 或 digest。~~ **【已失效 · 2026-08-28】** 该修复已作为 `bcc8eb9` 合入 main（PR #5，rebase 后 SHA 变更，树与 `c862682` 逐字节相同）。独立复核：64 个常量对 FIPS 180-4 立方根推导零不符；5 组标准 KAT（含 1MB 多块）修复前 0/5、修复后 5/5。**禁令解除，该实现可用于核对 Golden 与 digest。** **跨仓证据锚点只能用 `bcc8eb9`,不能用 `c862682`**(2026-08-28 补):`fix/sha256-k28-round-constant` 分支已随 PR #5 删除,`git ls-remote --heads origin` 无此分支、`git branch -r --contains c862682` 为空——本条上文「已推 origin」现已不成立,该 SHA 只存在于开发机本地。`LumioVoxelEngine` 据本条把 `c862682` 当作跨仓证据锚点并回报「架构仓 main 仍是错常量」,即由此而来。判定锚点有效性请用 `git branch -r --contains <sha>`,`git cat-file -t` 对「本地已提交但未推送」会漏报。**遗留**:该常量仍无 CI 守护——本 workflow 对 Rust 侧只跑 `cargo check` 与 `cargo tree`,从不跑 `cargo test`,上文那 5 组 KAT 是一次性人工核验、未入库;补测已立卡 `.spec/tasks/sha256-kat-and-three-way-consistency.md`。
5. **`generate` 会 rmtree 掉已入库的 `packages/rust/Cargo.lock`**（既有行为）。当前树已手工复原。修法：`generate()` 在 rmtree 前保留并回写。 **【已发生 · 2026-08-28】** 本条预测的失效已在 `b8f8c50`（PR #7）真实发生：`packages/rust/Cargo.lock` 被从版本库删除，且该文件未被 `.gitignore` 覆盖。删除是有意还是 rmtree 副作用未经确认——它同时是 D-2 卡②「Cargo.lock 是否进镜像」这项待澄清的对象，**请连同该待澄清项一并裁决**，不要在裁决前擅自恢复或补 ignore。
6. **本机 `python3` 是 3.9，`generate` 需要 3.10+**（`Path.write_text(newline=)`）。本机须用 `python3.11`；CI 不受影响，`validate` 在 3.9 正常。
7. **跨仓核实纪律候选**（`lumioclient-a9` 提出，今日在三个仓同时发生，已越过 `lessons.md` 的「第二次出现」准入线）：**看到可疑数据先找约束它的 validator 与 schema，再下结论；只读值等于只读了一半。** 前两类误报（读自己仓落后的检出、读别人仓移动中的工作区）靠「只读 `origin/<branch>` 已提交对象」能拦，这一类拦不住。
8. **多会话共用工作区**：今日已真实发生一次分支被切换。约定已在各会话间达成——动手前必开隔离 `git worktree`，不在共享工作区切分支或提交。

---

## 四、审查状态

三张卡的交付均由**同一上下文自审**（本会话未启用子代理，按 `AGENTS.md` 宿主差异表的 Inline Fallback 降级执行）。自审确有产出：R-00003 抓到并修复两处（生成器不校验 ABI 文档就发射、已发布 bundle 不校验 compiler digest / inputHash）；R-00004 抓到并修复一处（新规则使 `artifact/duplicate-path` 变成两条错误，削弱了该 fixture）。

但「写 ≠ 审」的独立性缺失属**已知降级**。三张卡的 PR 均无人 review（`reviewDecision` 为空）。请 CODEOWNERS `@Go1c` 补一次独立审查。
