# 2026-08-28 · LumioNativeCore 跨仓请求裁决（B-ABI-001..008）

> 会话：`lumiogameenginearchitecture-distracted-chandrasekhar-edbf54`（隔离 worktree）。基线 `LGE-V1.4-2026-08-27` 未变。
> **测量时刻**：`2026-08-28T12:23:28Z` / `12:40:20Z` / `12:47:38Z` / `12:55:48Z`（四次，终测为最后一次）。下游上一次测量约在 `12:00 UTC+8 = 04:00Z`，其间 `origin/main` 前进了 13 个提交，本文每条结论都基于终测值。

## 0. 锚点与测量

| 项 | 值 | 命令 |
| --- | --- | --- |
| `origin/main`（12:23:28Z） | `f5ce0e3` | `git ls-remote --heads origin` |
| `origin/main`（12:40:20Z） | `f9c446b` | 同上 |
| `origin/main`（12:47:38Z） | `fde7e34` | 同上 |
| `origin/main`（12:55:48Z，终测） | `d812617` | 同上 |
| 远端分支（12:55:48Z） | `refs/heads/main` 与 `refs/heads/claude/stoic-golick-3eff95`；当日其余特性分支均已合并并删除 | 同上 |
| 本轮交付分支 | `claude/distracted-chandrasekhar-edbf54`（基于 `d812617`）**未推送** | `git log --oneline -1` |

**下游引用纪律**：本文标为「本轮新增」的结论**尚未推送、尚未合入 `origin/main`**。在它进入 `origin/main` 之前，下游不得据此改代码；标为「已在 `origin/main`」的结论现在就可用。

## 1. 逐条回答下游的六个请求

### 请求 1（唯一硬阻塞）· NativeCore 是不是生成产物的消费方 —— **已裁决为 (B)，已在 `origin/main`**

裁决在下游测量之后落地：`b8f8c50`（PR #7，2026-08-28 20:17:30 +08）已把 `packages/index.json` 的 `rootAbi.consumers` 设为 `["LumioCoreEngine","LumioNativeCore"]`，12 个生成产物的 `consumers` 保持不含 NativeCore。ADR-040 §5 写明：「A repository not in this list has no standing to depend on `packages/abi/`」。

```
$ git show origin/main:packages/index.json | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['rootAbi']['consumers'])"
['LumioCoreEngine', 'LumioNativeCore']
```

即 **(B)：NativeCore 消费 `packages/abi/` 的 C header 与 `ids/` / `schemas/` / `fixtures/`，不消费 Rust/C# 生成包。**

(B) 的后半段（按什么索引发现、有没有 hash 校验口径）此前**没有落字**，本轮补上：ADR-040 新增 **§7「What a `rootAbi` consumer reads, and what it may assert」**（本轮新增）：

- **发现**：四个索引，无第五个入口 —— `packages/index.json` 的 `rootAbi` 块、`ids/index.json`、`schemas/index.json`、`fixtures/index.json`。
- **数值权威**：生成的 Rust/C# 包只发布 **id 字符串**，`ids/index.json` 是 numeric 的唯一权威。从生成包里读 ordinal 等于读一个从未发布过的东西。
- **校验**：bundle 自校验 —— 重算每个 `outputFiles[].path` 的 SHA-256 与其 `digest` 比对，重算 bundle 自身摘要与 `rootAbi.bundleDigest` 比对，并记录 `rootAbi.compiler.digest`。
- **`ids/` / `schemas/` / `fixtures/` 在 V1 没有 per-file digest**，`docs/architecture/.baseline.sha256` 只覆盖架构正文一个文件（实测：该文件只有 1 行）。它们的完整性保证只有「钉死的镜像 revision 的对象身份」——**钉 revision，不钉分支名**。
- **补充一条下游可能不知道的硬约束**：按 ADR-006，跨仓 root symbol **由 CoreEngine 的 `root-abi`/`composition` 独占导出**，NativeCore 提供被组合的 API Table 契约，其发布物**不导出 root symbol**。两个仓都 bind `lumio_core.h`，但只有一个导出 `LUMIO_ENTRY_SYMBOL`。

### 请求 2 · ErrorCode 缺 Native kernel 侧 12 类 —— **已裁决：补进公共注册表（本轮新增）**

裁决依据不是下游便利，而是已冻结契约的推论：ADR-040 §3 已把 `lumio_status_t` 冻结为「承载 ID Registry `ErrorCode` numeric，`0` 为成功」，因此 **`ErrorCode` 命名空间就是 Root ABI 的全部公共状态空间**；而 ADR-006 的组合模型没有任何中间层能把仓内私有码翻译成注册值。所以一个未注册的返回值就是 Root ABI 上一个未注册的**公共**数值。

同时，ADR-006 有三处散文行为在注册表里根本没有对应值：「Buffer-too-small returns required size」无 `BufferTooSmall`；「both map to stable Error Codes」（捕获 panic）无稳定码；「Cancellation, timeout … are terminal」只有 loader 作用域的 1025/1026。

**新增 [ADR-046](../../.spec/decisions/ADR-046-native-kernel-status-band.md)（Draft）**，分配 10 个新值（1044–1053），另有 3 类由既有值承担：

| numeric | id | 条件 |
| --- | --- | --- |
| 1044 | `InvalidArgument` | 参数违反文档化前置条件（非 handle / capability / buffer 尺寸问题） |
| 1045 | `WrongContext` | handle 结构合法但 `context` 字不匹配 |
| 1046 | `BufferTooSmall` | 输出缓冲不足；被调方把所需尺寸写入 buffer 的 `capacity`，不写 payload |
| 1047 | `CapacityExceeded` | 定容结构（handle 表 / arena / slot 表）满 —— 区别于 `QueueFull`(1036) 与 `BudgetExceeded`(1035) |
| 1048 | `Cancelled` | 完成前被取消；终态，未写任何可观测状态 |
| 1049 | `TimedOut` | 超过截止时间；终态，未写任何可观测状态 |
| 1050 | `ContextClosing` | context 正在排空，拒绝新工作；在途工作仍可完成 |
| 1051 | `ContextDestroyed` | context 已不存在；调用在销毁后完成，不能写状态 |
| 1052 | `PanicBoundary` | 在 ABI 边界捕获 panic/异常；进程存活，槽位结果**未证**，默认 `FaultClass = SlotStateUnproven` |
| 1053 | `InternalInvariant` | 被调方自身不变式被破坏；永远是被调方缺陷，不是调用方错误 |

**不新分配、由既有值承担的三类**（冗余 numeric 一旦发布永不回收，故不重复分配）：

| 下游类别 | 注册值 | 理由 |
| --- | --- | --- |
| 无效 handle | `InvalidHandle` (1029) | 已冻结，覆盖结构非法与 generation 不匹配 |
| 已释放 | 使用路径 `InvalidHandle` (1029)；释放路径 `HandleDoubleRelease` (1030) | Index+Generation 编码使「释放后使用」= generation 不匹配；1030 正是「释放两次」 |
| 能力不可用 | `CapabilityMissing` (1020) | 同一谓词；该值的定义并不限定 loader 作用域 |

**同时新增一道门**：`ErrorCode` numeric 必须放得进 `lumio_status_t`。注册表 schema 原本允许到 `4294967295`，而 `int32_t` 装不下——现在 `tools/lumio_contract.py` 拒绝任何超过 `2147483647` 的 `ErrorCode` numeric，并配反向 fixture `ids/status-range`。

**对下游 `T-error-03` 的口径**：从「建立架构错误码映射（映射对象可能不存在）」改为「把冻结的 13 个 `ErrorCategory` 按上面两张表映射到 10 个 kernel band numeric 加 1020/1029/1030」。分配权仍在架构源，NativeCore 永不自行分配 numeric，也不得从槽位函数返回未注册的非零 status。

### 请求 3 · Capability 注册表与 Header 常量的关系 —— **裁决：V1 明确不冻结，`CapabilityKey` 保持 crate-private（本轮新增）**

实测事实（不是推测）：

```
$ grep -n capabilityBits schemas/native-managed-abi.schema.json
28:    "capabilityBits": { "type": "integer", "minimum": 0 },
$ grep -rn "capabilityBits" .spec/decisions/ docs/architecture/    # 语义定义：零命中
```

`7` 这个值在全仓**没有任何语义定义**：schema 只说「非负整数」，生成器原样透传到 `#define LUMIO_CAPABILITY_BITS 7u`。所以「是掩码还是计数」目前既不是掩码也不是计数——**它未被冻结**。本轮不编造答案，而是把「未冻结」写成规则：

- ADR-040 §7 明确：V1 **既不冻结** `lumio_root_api.capability_bits` 是掩码还是计数，**也不指派任何 bit 位**。
- ID Registry 的 `Capability` 命名空间装的是 **CoreEngine 的包能力**，其 numeric 是**枚举序号，不是 bit 位**——这正是 WebSocket 传输档被拒绝分配 `Capability` id 的同一条理由（见 `DECISIONS_PENDING` 的 2026-08-28 TransportProfile 记录）。
- **裁决**：下游 `CapabilityKey` **绑 crate-private**，不得从 `LUMIO_CAPABILITY_BITS` 或 9 值注册表推导。新开 **D-015** 待裁决行；确认后才谈绑定。

### 请求 4 · `OperationId` 命名空间不存在 —— **裁决为不适用（本轮新增）**

```
$ grep -rn "OperationId\|operationId" schemas/ ids/ .spec/decisions/ docs/architecture/
（零命中）
```

这不是「注册表缺失」，是**概念不存在**。公共的可调用操作身份已经有了，就是 (`apiTable[].name`, `slots[].slotIndex`) 这对——它已发布在 bundle 里，并被布局 Golden 断言。ADR-040 §7 写明：没有 `OperationId` 命名空间，不保留、也不需要保留，只要 dispatch 面还被 `DECISIONS_PENDING` D-009 挡着。

**对下游的口径**：B-ABI-004 关闭为「不适用」。`T-job-01` / `T-job-04` 的 crate-private 测试 ID **是正确做法**，不需要公共 test range，也不需要架构源为其预留。若将来 job 提交 API 进入 ABI 文档，操作身份仍是槽位，不是另一套注册表。

### 请求 5 · `layoutProfile` 只有一档 —— **裁决：V1 仅保证 `linux-x86_64-glibc`（本轮新增）**

ADR-040 §7 写明：V1 只为 `linux-x86_64-glibc` 发布 Golden，**其余平台布局下游不得断言**。共享 POD 用定宽字段、不用 `size_t`，所以它们*本意*不随宿主工具链走——但**本意不是 Golden**，没有发布物就不构成保证。

新开 **D-016** 待裁决行，记录 P1 平台矩阵（windows-x86_64 + darwin-arm64/x86_64 + linux-x86_64）与「补档是 additive、不动 BaselineId」这一影响面。已确认 `LumioCoreEngine` P0 只用 `p0-linux`，缺口只在 NativeCore 与 P1。

### 请求 6 · codec / diagnostics —— **仍阻塞：不批准转正**

先纠正一处溯源：`ARCH-P1-004 / 005 / 009` 与 `OPEN-002` 在本仓**零命中**（`grep -rn 'ARCH-P1-00\|OPEN-00' docs .spec` 无输出），那是下游侧编号。本仓的权威落点是两处：

1. 架构正文 [`LumioGameEngine_Architecture_v1.4.md:558`](../architecture/LumioGameEngine_Architecture_v1.4.md) 的模块表，NativeCore 行的「可扩展」列写着：SIMD、`codec`（纯字节压缩/校验/diff，**待批准**）、`diagnostics`（**待批准**）。
2. `DECISIONS_PENDING` **D-004**（Transport/Codec/压缩选型）：「Adapter-only choice does not change baseline; envelope/codec changes do.」

**裁决**：维持「待批准」，本基线**不发布** codec / diagnostics 的公共格式、算法、operation、error、record schema、ID 或 batch 产物。下游 ADR 0005（NativeCore 仓内编号，与本仓 ADR-005 Replication 纯属撞号）的自我冻结——维持 pending、只做 feature-gated 私有原型、不进公共 Header 与 export list、转正只由架构源批准驱动——**与架构源当前状态一致，予以背书**。

额外一条下游应知的前置：本仓 P0 Gate 评审的 **D-9** 指出 `ADR-010:20` 引用的「the same canonical codec rules」当前**无指向物**（已冻结的 canonical 只有 JSON 文本形态的 `CanonicalJsonV1`，而 ADR-035 已把二进制 voxel payload 的字节冻得很死却没定义 primitive 层布局）。**codec 的公共面在 D-9 裁决之前不可能转正**——先有二进制 canonical 规范，才谈得上公共 codec。

## 2. B-ABI-001..008 逐条裁决

| 编号 | 题面 | 裁决 | 依据 |
| --- | --- | --- | --- |
| B-ABI-001 | **本仓未收到题面** | **无法裁决** | 下游本轮请求未描述 001；请补题面，或确认它已随 PR #1 一并解除 |
| B-ABI-002 | **本仓未收到题面** | **无法裁决** | 同上 |
| B-ABI-003 | Root ABI 类型/常量 | **已解除** | `origin/main:packages/abi/lumio_core.h` 含 `lumio_handle_t`(16B)、`lumio_buffer_t`(24B)、`lumio_status_t`=int32、`LUMIO_ENTRY_SYMBOL`；双方已确认 |
| B-ABI-004 | Operation ID Registry 与 test range | **已裁决为不适用** | 概念不存在；公共操作身份是 (`apiTable.name`, `slotIndex`)，已发布；crate-private 测试 ID 正确且无需公共保留（ADR-040 §7，本轮新增） |
| B-ABI-005 | — | **已解除** | 随 PR #1 / `44f617b`；双方已确认 |
| B-ABI-006 | — | **已解除** | 随 PR #1 / `44f617b`；双方已确认 |
| B-ABI-007 | codec 公共格式/算法/operation/error 的裁决与产物 | **仍阻塞** | 架构正文 v1.4:558 维持「待批准」；D-004 未确认；且 D-9（二进制 canonical 无指向物）是其硬前置。本基线不发布任何 codec 公共产物 |
| B-ABI-008 | diagnostics record schema / ID / batch 的裁决与产物 | **仍阻塞** | 架构正文 v1.4:558 维持「待批准」。本基线不发布任何 diagnostics 公共产物 |

**额外一条（下游未编号但本轮已裁决）**：ErrorCode kernel 语义缺口 —— **已裁决并落地**（ADR-046 Draft，1044–1053），见 §1 请求 2。这条解开后 `contract-types` 的 `{ _private: () }` 占位 newtype 中错误码那一支即有源；但注意**它依赖本轮交付进入 `origin/main`**。

## 3. 改动清单

| 文件 | 改动 |
| --- | --- |
| `.spec/decisions/ADR-046-native-kernel-status-band.md` | **新增**（Draft）：kernel band 1044–1053、三条既有值映射、int32 范围门 |
| `.spec/decisions/ADR-040-root-abi-generated-bundle.md` | **修订**（Draft 允许修订）：新增 §7 消费方发现/校验/不冻结项/导出方唯一性 |
| `.spec/decisions/README.md` | 登记 ADR-046 |
| `docs/adr/ADR-046-native-kernel-status-band.md` | 兼容软链接 |
| `ids/index.json` | ErrorCode 43 → 53 值 |
| `fixtures/valid/id-registry.json` | 与 `ids/index.json` 同步（门禁要求二者逐字节相同） |
| `fixtures/invalid/id-registry-status-range.json` | **新增**反向 fixture：numeric `2147483648` 越出 `lumio_status_t` |
| `fixtures/index.json` | 登记 `ids/status-range` |
| `tools/lumio_contract.py` | 新增 `_STATUS_NUMERIC_MAX` 与 ErrorCode 范围语义规则 |
| `docs/architecture/DECISIONS_PENDING.md` | 新增 D-015（capability bits 语义）、D-016（Root ABI 平台布局矩阵） |
| `packages/**` | 按生成源重生成（生成物不手改） |
| `docs/reviews/2026-08-28-nativecore-abi-adjudication.md` | 本文 |

**未改动、且刻意未动**：`packages/abi/lumio_core.h`（逐字节相同）、架构正文 v1.4 与 `.baseline.sha256`、`schemas/**`、`packages/rust/Cargo.lock`（它是 P0 Gate 评审 §三.5 的待裁决对象，本轮不恢复、不补 ignore）。

## 4. 验证证据

全部在交付分支（基于 `origin/main = d812617`）上执行，本机 `python3` 是 3.9 而 `generate` 需要 3.10+，故用 `python3.11`（CI 不受影响）。

```
$ node .spec/tools/spec-lint.mjs                                  → spec-lint: OK          退出码 0
$ node --test .spec/tools/spec-lint.test.mjs                      → 13/13 pass             退出码 0
$ python3.11 -m py_compile tools/lumio_contract.py                →                        退出码 0
$ python3.11 tools/lumio_contract.py validate                     → Validated 191 fixture(s), 0 failure(s).   退出码 0
      其中 PASS ids/registry (valid) / PASS ids/duplicate (invalid) / PASS ids/status-range (invalid)
```

复现 `.github/workflows/repository-policy.yml` 的 Hash / 文件检查：

```
$ shasum -a 256 -c docs/architecture/.baseline.sha256
docs/architecture/LumioGameEngine_Architecture_v1.4.md: OK                                  退出码 0
$ 必需文件 test -s（README/LICENSE/.gitattributes/v1.4/ADR_INDEX/评审稿/schemas|fixtures|ids index/工具/requirements） → 全部存在
$ grep '^\*\.md text eol=lf$' .gitattributes / '^# LumioGameEngine V3 (v1.4)' / 'LGE-V1.4-2026-08-27'(正文与 README) → 全部命中
$ python3.11 tools/lumio_contract.py generate --out <scratch> 后与已入库 packages/index.json 比对
  full packages/index.json identical to fresh generate: True
$ cargo check --manifest-path packages/rust/Cargo.toml             → Finished dev profile   退出码 0
$ cargo tree --manifest-path packages/rust/Cargo.toml -p lumio-gen-contract-runtime          退出码 0
$ cargo test --manifest-path packages/rust/Cargo.toml -p lumio-gen-contract-runtime → 3 passed 0 failed  退出码 0
$ python3.11 tools/lumio_kat.py → csharp + hashlib + rust agree on 3 FIPS 180-4 vectors      退出码 0
$ cargo clippy --manifest-path packages/rust/Cargo.toml --all-targets -- -D warnings → Finished  退出码 0
$ grep -RniE 'AllowUnsafeBlocks>true|DllImport|PInvoke|NativeLibrary|PackageReference' packages/csharp | grep -viE '…' → 无匹配（C# 无 Native）
```

产物身份变化（相对 `origin/main = d812617`）：

| 项 | `origin/main`（`d812617`） | 本轮交付 |
| --- | --- | --- |
| `compilerHash` | `5940e51f017822b2…` | `217437fd4755e1a3…` |
| `contract-types-rust.outputHash` | `ec297fa8e1e997e6…` | `5c686850855bdf11…` |
| `contract-types-csharp.outputHash` | `94316e589813bcae…` | `69a2f24659520258…` |
| `rootAbi.bundleDigest` | `88321f1c3374c40c…` | `03ca75361fed3ca9…`（**只有 `compiler.digest` 一个字段变了**） |
| `rootAbi.inputHash` | `696a58d0525b897b…` | `696a58d0525b897b…`（不变） |
| `packages/abi/lumio_core.h` | — | **逐字节相同**（`git diff origin/main..HEAD -- packages/abi/lumio_core.h` 空） |

12 个 artifact 中 10 个的 `outputHash` 完全不变——只有 `ContractTypes`（Rust 与 C#）承载错误码 id 列表。`compilerHash` 变动是编辑 `tools/lumio_contract.py` 的必然结果，不是内容漂移。

## 5. Known gaps

1. **本轮改动未推送、未合入**。交付分支只在本地 worktree 上；对外发布动作需用户确认。下游在它进 `origin/main` 前只能消费第 2 节标为「已在 `origin/main`」的结论。
2. **未做独立审查**。本会话未派 reviewer 子代理，属 `AGENTS.md` 宿主差异表的 Inline Fallback 降级——「写 ≠ 审」的独立性缺失是已知降级，与 ADR-040/041/042 三次交付同一性质。ADR-046 分配的是**永不回收**的公共数值，建议 CODEOWNERS 独立审一次再合。
3. **ADR-046 是 Draft**。与 ADR-040/041/042 一样，需架构所有者确认后随下一基线转 `Accepted`。把 Draft 状态的公共数值冻进 CoreEngine 只读镜像会在转 Accepted 时再触发一次登记（同 P0 Gate 评审 D-2 的第③条排序约束）。
4. **本 ADR 的编号是 046，不是 043**。本轮起草时 `origin/main` 的最高 ADR 号是 042；落地过程中上游连续占用了 ADR-043（Loader 重入）、ADR-044（Evidence Profiles）与 ADR-045（Replication body 闭合），本卡两次重编，最终号是 **ADR-046**。若下游已记下 043 或 045，请以 046 为准。
5. **B-ABI-001 / 002 无题面**，无法裁决。
6. **ErrorCode numeric 仍不出现在任何生成产物中**。生成的 `ContractTypes` 只发 id 字符串；数值权威只有 `ids/index.json`。这是既有属性，ADR-046 未改变它，但它与 P0 Gate 评审 **D-3**（「ID ordinal 的权威来源在哪里」）是同一个问题，D-3 未裁决。
7. **D-015 / D-016 只是登记，不是裁决**。capability bits 语义与 P1 平台布局矩阵仍未冻结。
8. **本仓 `origin/main` 分钟级在动**。本文四次测量之间（12:23:28Z → 12:55:48Z，约 32 分钟）主干前进了 9 个提交、合并并删除了四个特性分支、占用了三个 ADR 号，本卡因此重编两次并重做四次 rebase + 重生成。**本轮交付若不尽快合入，会再次落后并需要重做一次重生成。**
9. **一次已捕获并已修复的冲突消解缺陷**：第三次 rebase 时对 `packages/` 取上游版本的同时，误把上游的 `tools/lumio_kat.py`、`tools/lumio_generate.py`、`.github/workflows/repository-policy.yml` 与一张任务卡回退了。已用 `git diff --name-status origin/main..HEAD --diff-filter=D` 审计发现并逐个恢复；最终净差异只剩本文 §3 列出的 11 个文件，`D`（删除）为空。下游无需关心，但记录在此以免同类问题静默复发。

## 6. 沉淀落点

- 决策：[ADR-046](../../.spec/decisions/ADR-046-native-kernel-status-band.md)（新增，Draft）与 [ADR-040 §7](../../.spec/decisions/ADR-040-root-abi-generated-bundle.md)（修订，Draft）——本仓公共 ADR 的唯一落点。
- 待裁决：`DECISIONS_PENDING` D-015 / D-016。
- `knowledge/` 无新增：本轮没有产生新的开发规范或可复用模式，按 `AGENTS.md`「改完沉淀」的豁免条款声明豁免。
