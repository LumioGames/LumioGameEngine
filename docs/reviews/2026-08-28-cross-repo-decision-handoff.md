# 2026-08-28 · 跨仓协作交接与待裁决清单

> **日期**：2026-08-28（晚于同日的 [七仓进度评估与对账报告](2026-08-28-seven-repo-progress-assessment.md)）
> **范围**：SHA-256 K[28] 缺陷的修复与跨仓贯通；七仓并行会话协作中暴露的公共契约缺口
> **方法**：全部结论附命令与输出；跨仓引用一律以 `origin` 上的提交为锚点
> **性质**：交接文档。已落地的部分是记录，待裁决的部分需要架构所有者（用户）决定

## 1. 摘要

一个契约级正确性缺陷已修复并贯通到下游，两条分支均已合入各自 `main`。协作过程中，七个实现仓的并行会话共同暴露出一组**公共契约缺口**，它们不是实现问题，需要 ADR 级裁决——这是本文档的主要交接内容。

| 项 | 状态 |
|---|---|
| SHA-256 K[28] 缺陷修复（架构源） | **已合入** `LumioGameEngineArchitecture` main `bcc8eb9` |
| 下游镜像贯通（LumioVoxelEngine） | **已合入** main `4ced801`（PR #3） |
| 公共契约缺口 | **5 项待裁决**，见 §4 |
| 伪缺陷 | 3 条已排除，见 §5 |

## 2. 已落地：SHA-256 K[28] 缺陷

### 2.1 缺陷

生成的 Rust `ContractRuntime` 中 SHA-256 轮常量 `K[28]` 写作 `0xc6eabbdc`，FIPS 180-4 规定值为 `0xc6e00bf3`。单个轮常量错误会污染每一轮压缩，该实现**对任意输入都算出错误摘要**：

```
sha256("")    得 d86c89fc171387b0a8793333e938280743f338afed0655c7b3b5ca75d34957f1
              应 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
sha256("abc") 得 35ef76aeef15087adaeaa82c1e120b2f60c15085a198e4cb6f7d52402b9e74ab
              应 ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
```

整张 K 表对照 FIPS 180-4 推导（前 64 个素数立方根小数部分前 32 位）逐项核验：**64 项中仅 K[28] 不符**。

这是**跨语言契约分叉**，不是笔误。C# 侧 `ContractRuntime` 用 `System.Security.Cryptography.SHA256.HashData`，即真算法；同一份 hash-chain 契约的两个生成实现对任何输入都不一致，从 `HashChain.Sha256(empty)` 的 genesis 起就分叉。缺陷自引入即存在，因此**本项目历史上每一次经 Rust 侧执行的 artifact hash 校验都跑在坏 hasher 上**。

### 2.2 修复与影响面

修在生成源 `tools/lumio_generate.py` 的 `SHA256_RS` 模板（一行），随后整体重新 `generate`，未手改任何生成物。

- 爆炸半径为**一个 artifact**：`contract-runtime-rust` outputHash `ee8fa744 → 61458289`
- 其余 11 个 artifact 字节不变
- `inputHash` 不变（无 schema / ID / fixture 改动）
- `compilerHash` `01049476 → 3a46fc31`，ADR-040 root ABI `bundleDigest` `146b074f → 25e78226`——两者均派生自生成器源码
- BaselineId 不变：`LGE-V1.4-2026-08-27`

其余六仓已扫描确认无手写 SHA-256 实现（`0x428a2f98` 零命中），无需动作。

### 2.3 下游症状与根因链

`LumioVoxelEngine` 的 `lumio-voxel-contracts` 通过 `lumio_gen_contract_runtime::sha256_hex` 自算 artifact 摘要，因此错常量使其对每个 artifact 都算错，在字母序第一个包上先失败：

```
修复前（纯净 origin/main）  HashMismatch { artifact_id: "canonical-serializer-rust" }
                            test result: FAILED. 1 passed; 2 failed
修复后（合入 main 后复跑）  test result: ok. 3 passed; 0 failed
```

### 2.4 验证证据

架构源（`bcc8eb9`）：

```
spec-lint                         spec-lint: OK
node --test spec-lint.test.mjs    pass 13, fail 0
lumio_contract.py validate        Validated 174 fixture(s), 0 failure(s).
CI outputHash stability           packages/ matches generate: 12 artifacts
cargo check (1.98.0 aarch64)      Finished dev profile
FIPS 180-4 已知向量               sha256("") PASS, sha256("abc") PASS
```

`LumioVoxelEngine`（`4ced801`，本机 aarch64-apple-darwin 复跑）：

```
check_generated_clean.py                              check-generated-clean OK
example check-generated-clean                         check-generated-clean OK
artifact_hashes                                       3 passed / 0 failed
test --workspace --all-features --no-fail-fast        158 passed / 0 failed
clippy --workspace --all-targets --all-features -D warnings   exit 0
```

镜像随之从 52 文件扩到 58 文件：上游 main 已前进至包含 ADR-040 root ABI 与 ADR-041 canonical profile 产物，而 `canonical-serializer-csharp` 与两个 `language-binding` 包各自新增了源文件，其 descriptor 声称的 outputHash 只有镜像带上这些文件才重算得出——**只更新原 52 文件的折中经实测不成立**。

> 复跑提示：本机 `rustup` 默认为 `x86_64-apple-darwin`（Rosetta）。所有 Rust 验证须显式使用 `cargo +1.98.0-aarch64-apple-darwin`，否则得到的是 Rosetta 结果。另：多会话并发跑 cargo 时出现过一次瞬时链接失败，重跑即过，非真实失败。

## 3. 遗留缺口：本缺陷为何能存活至今

**没有任何测试守住这个常量。** 生成器与生成物均无 SHA-256 已知向量测试，CI 也从不执行生成的 Rust hasher——`packages/rust` 只跑 `cargo check` 与 `cargo tree`。同类缺陷可以再次静默发布到全部下游。

建议（未实施，需裁决是否纳入）：

1. 生成的 Rust runtime 携带 KAT：空串、`"abc"`、以及一个跨块（>55 字节）输入；
2. 一条 Rust ⇄ C# ⇄ Python `hashlib` 的三方一致性断言——**单看任何一侧自洽都发现不了这个 bug，正因为三者从未互相比对**；
3. 将 KAT 纳入 CI 必跑集合。

## 4. 待裁决清单

均为公共语义问题，需按 ADR → Schema/ID → 正反 Fixture → README/Baseline → 七仓镜像回路处理。按「能解开多少下游」排序。

### 4.1 持久化 / WAL / Snapshot 的线编码形态

**问：** 是否需要一个独立于 `CanonicalJsonV1` 的**二进制** canonical 形态，还是这些面一律走 JSON canonical？

ADR-041 冻结的 `canonicalForm.formId = "CanonicalJsonV1"`、`encoding = "AsciiEscaped"`，明确是 **JSON 文本**的 canonical form。而 `LumioGameRuntime` R-00141 要的是 UInt64 / 字节数组 / endianness / MessagePack primitive 的二进制线编码。实测上游对这一层零规范：

```
grep -rliE "messagepack|msgpack" schemas .spec/decisions packages   → 零命中
endianness / wire encoding 的命中全在 native ABI 语境
  （ADR-006 / ADR-020 / ADR-040 / native-managed-abi.schema.json / root-abi-bundle.schema.json）
```

不答此问，即使给出 digest domain tag 也定不下字节。**连带影响**：若裁决为「一律 JSON」，`LumioGameRuntime` 的 `Directory.Packages.props` 中 `MessagePack 3.1.8` 不应留在依赖表。

### 4.2 是否新增 Snapshot / MappingSet / ChangeSet 三个 digest domain

ADR-041 冻结的 digestDomain 恰好五个，全在 Loader / artifact 面：

```
manifestDigest      -> CoreEngineManifestBody      artifactSetDigest -> ArtifactSetV1
artifactIndexDigest -> ArtifactIndexV1             targetProfileDigest -> TargetProfileV1
capabilitySetDigest -> CapabilitySetV1
```

`mappingSetHash` / `snapshotId` / `SnapshotV1` / `MappingSetV1` / `ChangeSet` 在 ADR-041 中出现次数**均为 0**。且域分离是**结构性**的：每个摘要输入必须是带强制 `digestDomain` 成员的 JSON 对象（唯一例外 `manifestDigest`，由 ADR-018 先于本 ADR 冻结）。因此「复用 CanonicalJsonV1 算 snapshotId」当前**没有合法 domain tag**，自造 tag 即各仓自行发明公共合同。

> 已明确可依赖的部分：`LumioClient` 的 artifact / manifest 验证面（五个域之内）字节口径唯一权威，为 `SHA-256(CanonicalJsonV1(<带 digestDomain 成员的对象>))`，无 prefix / salt / length framing，profile 自带 8 条 Golden 可直接作测试向量。该仓据此已可开工，**不需要新裁决**。

### 4.3 generated 面的能力边界

`LGE-V1.4-2026-08-27` 的 generated artifact **零类型本体、零 validator、零 builder**。四仓独立复核，`origin/main` 上 `packages/csharp/` 为 8 文件 437 行；`validator` / `validate` / `builder` 实质命中 0；`Bindings` 表点名的类型定义数全为 0（`ConfigTable` / `ProcessorDescriptor` / `TxnJournalRecord` / `CommandLogRecord` / `WalRecordEnvelope` / `EntityIdentity` / `ReplicationEnvelope` / `SessionRevisionVector`）。

而下游多张卡的验收项写着「调用 generated validator」「用 generated 类型开 overload」「按 generated ID ordinal 排序」。当前对下游是一对**无解约束**：既「不得自行发明公共合同」，又「必须调用 generated validator」。`LumioServer` 的降级做法（自实现权限比对）字面违反 ADR-022 的 `Hand-written per-repo validators were rejected for drift`；三仓各写一份 Envelope View 字面违反 ADR-028。

**问：** 边界是「schema 目录 + 名称绑定」，还是应产出类型本体与 validator/builder？若前者为有意设计，请写明各下游如何自行实现并保证跨仓 canonical bytes 一致；若后者，请给覆盖范围、排期与各下游此期间的处理口径。

### 4.4 发布形态是否覆盖 `netstandard2.1`

`packages/csharp/` 六个包 `<TargetFramework>` **全为 `net8.0`**。`LumioClient` 核心冻结在纯 `netstandard2.1`（Unity 与纯 .NET Host 的共同面），**引用不了**。即便 4.3 裁决为补齐类型本体，TFM 不动则该仓一行都用不上。

`LumioGameRuntime` 自身的 netstandard2.1 面亦残缺：`GeneratedContracts` 须排除 `contract-runtime-csharp` 才能过（`SHA256.HashData` 为 net5+）；`Observability` 声明 `netstandard2.1` 但编译不过（`System.Threading.Channels` 不在该 BCL，无补包引用）。

**问：** `netstandard2.1` 是否为一等公民？该问题横跨架构源与至少三个下游仓，当前无统一答案。

### 4.5 `FullSnapshot` / `Delta` 如何承载世界状态

`tools/lumio_contract.py` 的 `_REPLICATION_BODY_REQUIRED`：

```
FullSnapshot : snapshotId, tickId, sessionRevisionVector, schemaEpoch, mappingSetHash
Delta        : baseSnapshotId, fromRevision, toRevision, mappingSetHash,
               confirmationSequence, tombstones
```

**没有任何承载世界状态的字段**，8 条正向 fixture 的 body 字段集 exact-set 等于上表，无隐藏字段。而 ADR-028（Accepted）的 Alternatives 原文堵死了 free-form payload：`Keeping a free-form payload was rejected because two implementations can pass the gate and disagree on Snapshot identity.`

后果：MVP 立项目标「多浏览器联机挖/放方块实时互见」**在公共面上无法表达**。`LumioServer` 据此将 A1 拆为 A1-α（握手 → admission → FullSnapshot → BaselineAck → revision 前进 → DeltaAck → 断连 Full Resync，可交付）与 A1-β（「Bot 看到方块被挖」，BLOCKED）。

更深一层：`replication-mapping` 冻结的是「哪些字段被复制」的描述符，Envelope 只带 `mappingSetHash`，**被映射值的线编码整层未定义**——不是补一个 body 字段能了结。

同源第二条：8 个 messageType 无一表示 client→server gameplay 命令；把命令塞进 `BaselineAck.body` / `DeltaAck.body` 虽可通过 schema，但 D-009 明令禁止发明 dispatch wire format。

### 4.6 卡状态阻塞（非契约问题，但挡着下游）

`R-00003` / `R-00004` 的交付均已合入 `origin/main`（`44f617b` / `a4a7956`，CI 绿），但 Workflow 上仍停在「验收中」。下游卡的「精确前置」按卡状态判定而非 git，因此 `R-00015` 仍显示 3 缺 3，`LumioCoreEngine` 的 32 张 backlog 一张都开不了工。需核验后流转「已完成」。

## 5. 已排除的伪缺陷（防止再次进入裁决）

### 5.1 「descriptor 把消费仓同时写进 consumers 与 forbiddenDependents，自相矛盾」——**不成立**

三个仓独立报告过此条，均已撤回。证据：

```
forbiddenDependents: {"minItems":2, "items":{"enum":["LumioClient","LumioGame"]}}
consumers:           {"items":{"enum":["LumioClient","LumioGame","LumioGameRuntime","LumioServer"]}}
```

`forbiddenDependents` 的 enum 只有两个值且 `minItems: 2`，**只能**取该固定二元集，永远填不进 Runtime / Server——只能取单一固定值的字段不可能是消费方准入白名单。validator 的唯一拒绝条件是 `implementationDependencies` 非空；失败 fixture `generated-contract-artifact-client-dep.json` 同时带 `consumers:["LumioClient"]` 与 `forbiddenDependents:["LumioClient","LumioGame"]`，唯一违规点是 `implementationDependencies`，直接证明两字段共存合法。

真实语义为「**本 artifact 不得依赖这两个仓**」（ADR-023 的零实现依赖方向）。字段名 `dependents` 字面误导，一天内骗过三个仓——属**命名改进项**，非契约缺陷。

### 5.2 「CI 的 outputHash stability 步骤失败是 K[28] 引起的」——**不成立**

该步骤的摘要由 Python `hashlib` 计算，与生成的 Rust 实现无关。它一度在旧 main（`7f6c0c6`）上失败，真实原因是陈账：`packages/` 用更旧的 `tools/` 发布，`compilerHash 99a786e7 → 3b4230a3`。该陈账已由 R-00003 的重新 generate 清除。

### 5.3 「新产物缺 allow 头，是镜像分支引入的 clippy 缺陷」——**归因错误**

架构源自身 `packages/rust` 的 `clippy --workspace --all-targets --all-features -- -D warnings` 为 **exit 0**：那些 ABI 常量是独立 library crate 的 `pub` 项，`dead_code` 不触发。在 `LumioVoxelEngine` 中生成树被 vendored 进 `lumio-voxel-contracts` **内部私有 mod** 且未 re-export，才成为死代码。属**消费上下文差异**，非上游缺陷；下游 re-export 该 ABI 面即解（已验证 exit 0），无需改生成器、不触发额外 compilerHash churn。

## 6. 机制层建议（需裁决是否立项）

1. **`compilerHash` 把生成器源码与产物身份绑死。** `compiler_hash()` 对 `tools/lumio_contract.py` + `tools/lumio_generate.py` 取哈希，故工具改一行即移动全部 12 个 artifact 的身份，全下游 manifest 立即失配——即便一个生成字节都没变。2026-08-28 单日已变四次：`99a786e7 → 3b4230a3 → 2545ab1c → 01049476 → 3a46fc31`。建议评估拆分为「生成器版本号（人工递增）+ 内容哈希」。
2. **下游闸门不得以任何人的工作区为真值。** `LumioGameRuntime` 的 `eng/verify-generated-contracts.sh` 从 `$LUMIO_ARCHITECTURE_ROOT` **工作区**重新生成再比对，等于采样「此刻恰好 checkout 的那个分支」，把自身确定性外包给了别人的工作区。应 pin 到 `origin` 上的提交号 + 该提交的 `packages/` 内容取源。
3. **跨仓锚点不得使用未合并的 SHA。** 多仓在同一天踩到：rebase 合并会重写 SHA，`84dcad5` / `d134046` / `f426278` 均已失效。判据用 `git ls-remote` 或 fetch 后的 `git branch -r --contains`；`git cat-file -t` 对「本地已提交未推送」漏报，`git rev-parse origin/main` 在未 fetch 的仓里会读到陈旧 remote-tracking ref。给出跨仓 hash 时应同时标注其所基于的上游 commit。
4. **`LumioCoreEngine` 的 V1.2 lock 滞后是有意的，非欠账**（卡面钉死「不得因相邻仓出现 V1.4 而静默升级」）。升级代价实测：requiredPaths `131 → ~251`、62~63 处内容漂移、1 处删除（`fixtures/invalid/processor-read-write-conflict.json` 在 V1.4 中已不存在，属「required path 消失」）、镜像目录改名。另需改 `sync-architecture.sh` 的投影规则——它只认五个前缀，`packages/**` 会被判为不可投影路径直接 fail。应排在三张 Gate 卡关闭之后，单独走卡。

## 7. 环境与协作纪律

- **并发会话共用同一工作区**是当日的实际事故源：多个会话共开 `~/LumioGames/LumioGameEngineArchitecture`，曾出现一个会话 `git checkout -b` 换掉另一会话脚下分支。约定：任何会话要改文件，先 `git worktree add` 到隔离目录，共享 checkout 不切分支、不提交。
- **Rust 验证须显式指定 `+1.98.0-aarch64-apple-darwin`**：本机 `uname -m` 为 arm64，但 rustup 默认 host 是 `x86_64-apple-darwin`（Rosetta）。
- **生成器需要 Python 3.10+**（`Path.write_text(newline=)`）。本机默认 `python3` 为 Xcode 自带 3.9.6，会直接 `TypeError`；`lumio_contract.py validate` 在 3.9 下可用。
- `lumio_contract.py generate` 会清空输出目录，**已入库的 `packages/rust/Cargo.lock` 会被删除**，每次 generate 后须 `git checkout -- packages/rust/Cargo.lock`。

---

*本文档为交接记录。§2 与 §5 是已核实的事实，§4 与 §6 需要架构所有者裁决。*
