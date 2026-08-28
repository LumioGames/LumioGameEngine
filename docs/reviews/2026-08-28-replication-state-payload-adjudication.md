# 2026-08-28 · Replication 状态载荷缺口 · 独立复核与裁决请求

> 复核会话：`lumiogameenginearchitecture-stoic-golick`。**本文只做复核与选项分析，未改任何公共面，未建任何卡。**
> 测量时刻：**2026-08-28T12:06Z**，git 状态复测 **12:12Z**（跨仓状态几十分钟即过期——§0.2 正是一例）。
> 来源：`LumioServer` R-00260（`origin/main:docs/specs/2026-08-28-mvp-csharp-host-design.md`，`490fdb1`，已实测为其 `origin/main` 祖先）。

## 0. 先说三件会改变处置方式的事

### 0.1 本诉求已被平行会话上报，且已进 `origin/main`

`lumiogameenginearchitecture-6a` 已交付 `docs/reviews/2026-08-28-gate-p0-delivery-and-escalations.md`，**在本次复核进行期间推送**，当前 `origin/main = 4f36d92`。对照关系：

| 本次诉求 | 已有落点 | 状态 |
| --- | --- | --- |
| 主诉求（状态载荷）+ 次诉求 A（上行承载） | 该文 **D-1** | 已上报，措辞一致 |
| 次诉求 B（C# catalog-only / ADR-022 落差） | 该文 **D-3** | 已上报，且已合并 Runtime 侧同一前提 |
| 次诉求 E（R-00258 §0 措辞） | 该文 三·2 | 已上报，拟改措辞与本诉求一致 |
| 次诉求 F（`forbiddenDependents` 命名） | 该文 三·3 | 已上报，判为命名改进项 |
| **次诉求 C（`mappingSetHash` 无映射集取值）** | — | **未覆盖** |
| **次诉求 D（`length` 语义）** | — | **未覆盖** |

**处置建议：不新开一份平行上报。** C / D 两项以及下面 §2 的新测量并入既有 D-1，避免同一裁决被两份文档分别推动。

### 0.2 `main` 分叉已自行解决（复核期间状态变化）

复核开始时 `main` 与 `origin/main` 分叉，K[28] 被两个会话各修一次（`c862682` / `bcc8eb9`，`git diff` 为空）。

> **锚点提醒（2026-08-28 补）**：上面的 `c862682` 只用于描述当时的分叉历史，**不可作为跨仓证据锚点**——其分支 `fix/sha256-k28-round-constant` 已随 PR #5 删除，`git branch -r --contains c862682` 为空，该 SHA 只存在于开发机本地。跨仓引用一律用 `bcc8eb9`。判定锚点有效性用 `git branch -r --contains <sha>`，`git cat-file -t` 对「本地已提交但未推送」会漏报。
**复核结束时（2026-08-28T12:12Z）已恢复一致：`main == origin/main == 7bdad78`。** 另一会话已合并并推送，`origin/main` 另含 `7bdad78`（D-8 normalization 数据化）。

早先的本地 `fcaea48` 不再是 `origin/main` 祖先，但其内容经 `bcc8eb9` 带入且两树逐字相同，**无内容丢失**。经用户裁决后复核，**未执行任何重置**——分叉已不存在，重置为无意义操作。

### 0.3 查重结果（RM-00001 = room `01a04225-4fc…`）

全量 261 张需求逐页拉取后确认：RM-00001 恰好 8 张（R-00003/00004/00005/00006/00008/00009/00257/00258），**无任何一张覆盖状态载荷或上行命令承载**。R-00258 已读全（正文 + 2 条评论 + 0 附件），其第 2 条评论已自行更正 `f426278 → a738524` 的 rebase 锚点漂移。

## 1. 缺口成立 —— 逐条复现

| # | 诉求断言 | 复核结果 |
| --- | --- | --- |
| 1 | `_REPLICATION_BODY_REQUIRED` 全是标识与版本 | ✅ [`tools/lumio_contract.py:362-371`](../../tools/lumio_contract.py) 逐字一致 |
| 2 | 8 条正向 fixture body 字段集 exact-set 等于上表 | ✅ 8 条逐条实测相等（`replication-mapping` / `state-machine-*` 非 Envelope，不计入） |
| 3 | ADR-028 Accepted，明文否决 free-form payload | ✅ 原文逐字一致 |
| 4 | 缺的是「被映射值的线编码」整层 | ✅ `mappingSetHash` **在任何 Schema 里都不存在**（见 §2.2），Envelope 只带这一个哈希 |
| 5 | R-00258 §0 与 §3.3 自相矛盾 | ✅ §0 称「已经完整覆盖 MVP A1 所需的公共语义」，§3.3 的 body 必填表正好证明状态载荷缺位 |
| 6 | ADR-022 否决手写 validator | ✅ `ADR-022:42`「Hand-written per-repo validators were rejected for drift」 |
| 7 | `ReplicationEnvelope` 类型未生成 | ✅ `Bindings.cs:9` 声明该映射，全仓 `grep` 无任何类型定义 |

## 2. 新增测量 —— 缺口比上报的更深

### 2.1 门禁拦不住不合规实现（**本节是本次复核的主要增量**）

`replication_body_errors`（[`tools/lumio_contract.py:536-539`](../../tools/lumio_contract.py)）**只查缺失、不查多余**；`replication-envelope.schema.json` 的 `body` 是裸 `{"type":"object"}`，**无 `additionalProperties:false`**。在隔离副本上实测五组变异，**全部通过 174 条 fixture 门禁、零失败、零告警**：

| 探针 | 变异 | 门禁结果 |
| --- | --- | --- |
| 1 | `FullSnapshot.body` 注入私有世界状态载荷 | **PASS** |
| 2 | `mappingSetHash = 42`（整数） | **PASS** |
| 3 | `mappingSetHash = null` | **PASS** |
| 4 | `length = 999999999` | **PASS** |
| 5 | `BaselineAck.body` 夹带 gameplay 命令 | **PASS** |

**结论：ADR-028 当前在其自身立论上未被机器强制。** 它拒绝 free-form payload 的理由原文是「two implementations can pass the gate and disagree on Snapshot identity」——而探针 1 与 5 正是这句话描述的状态：两个实现可以同时通过门禁，各自携带任意私有载荷。

这条改变修复的形状：**只补字段不够，必须同时收紧门禁**，否则新冻结的字段同样不可强制。`LumioServer` 的「出站 exact-set」自律断言正确，但它是仓内自律，公共面无对应物。

### 2.2 次诉求 C 比上报的更严重

`mappingSetHash` 全仓只出现在 `lumio_contract.py` 的必填键名元组、ADR-028 正文、R-00258 文档三处。**它在任何 Schema 中都没有类型定义**——不是「取值域 hash256」，而是**完全无类型约束**（探针 2、3 佐证）。因此「无映射集时的合法取值」未定义之外，还叠加「有映射集时的取值域也未定义」。

### 2.3 次诉求 D 佐证

8 条正向 fixture `length` 一律 `256`，而其 body 紧凑 JSON 为 17–314 字节、整信封为 482–789 字节。**`length` 与任何一种读法都对不上**（`replication-full-snapshot.json` 整信封 789 > 256）。`tools/lumio_contract.py:463` 的 `length` 属 `recovery_record_checksum`，是另一个域，与 Envelope 无关。该字段目前是**纯占位符**。

### 2.4 次诉求 F 需修正定性

`forbiddenDependents` 不只是命名误导——它被 [`tools/lumio_contract.py:1288-1292`](../../tools/lumio_contract.py) 断言**必须恒等于 `{"LumioClient","LumioGame"}`**，因此**不携带任何 per-artifact 信息**，是一个必填常量。真正做依赖判定的是 `implementationDependencies`（`fixtures/invalid/generated-contract-artifact-client-dep.json` 正是因它为 `["LumioClient"]` 而失败，其 `forbiddenDependents` 反而是合规值）。定性应为「**冗余常量字段 + 命名误导**」。

### 2.5 对来源诉求的一处数字更正

C# 生成物为 **8 个文件 / 437 行**（含后加的 `RootAbi.cs` 101 行），非诉求所称约 266 行。`ProtocolPermissionValidator` 包只有 `ActivePermissionFields.cs`（23 行，15 个字段名字符串），无任何可执行校验方法；`schemas/protocol-permission-gate.schema.json` 存在，但无对应生成实现。结论方向不变。

## 3. 裁决选项（供架构所有者决定，本会话不代决）

主诉求给了三条路，复核后各自代价：

| 选项 | 内容 | 代价 / 风险 |
| --- | --- | --- |
| **甲** | 为 `FullSnapshot`/`Delta` 新增公共状态载荷字段 + 冻结其线编码 | 触碰 ADR-028 冻结的 typed body（破坏性），需新 ADR 取代/细化；**必须同时定义 primitive 字节布局**，否则重蹈 D-9（`ADR-010:20` 指向不存在的二进制 canonical） |
| **乙** | 裁定 `replication-mapping` 承担值编码职责 | 该 Schema 现只冻结**哪些字段被复制**（描述符），要扩到值编码等于重定义其职责；`mappingSetHash` 仍需先定义类型 |
| **丙** | 裁定某既有字段承担 | 复核未找到候选——8 个 body 无一语义可容纳状态；不推荐 |

**本会话建议：甲，且拆成两步。** 第一步先把**门禁收紧**（`body` 加 `additionalProperties:false` 或验证器改 exact-set）与 **`mappingSetHash` / `length` 定型**——这三项不依赖状态载荷怎么设计，是纯粹的「把已有意图变成机器可判」，且能立即让 §2.1 的五个探针转为失败。第二步再冻结状态载荷编码，届时它才是可强制的。

第一步与 D-9（二进制 canonical profile 是否补）强耦合：**状态载荷的线编码就是 D-9 缺的那套 primitive 布局**，建议合并裁决，勿分两次。

## 4. 上行命令承载（次诉求 A）

8 个 MessageType 无一表示客户端命令；客户端可合法发出的只有 `Handshake`/`BaselineAck`/`DeltaAck`/`ResyncRequest`，全是复制链路控制消息。D-009 仍冻结（`packages/index.json` 的 blocked 列表实测仍含 `D-009 protocol-dispatch not frozen`）。探针 5 证明**塞进 Ack body 机器可通过**——`LumioServer` 拒绝这么做是对的，但公共面没有任何东西阻止别的仓这么做。

需裁定：MVP 期合法承载方式，或明确「MVP 不做上行命令，只做服务端权威演示」（后者即 `LumioServer` 已按此拆出的 A1-α / A1-β）。

## 5. 收口门槛（本会话未改动，仅记录基线）

```
node .spec/tools/spec-lint.mjs                → spec-lint: OK
node --test .spec/tools/spec-lint.test.mjs    → tests 13 / pass 13 / fail 0
python3 -m py_compile tools/lumio_contract.py → OK
python3 tools/lumio_contract.py validate      → Validated 174 fixture(s), 0 failure(s).
```

变异探针在**隔离副本**（scratchpad）上执行，工作区 fixture 未被修改。

## 5.5 裁决结果（2026-08-28 · 架构所有者）

| 项 | 裁决 |
| --- | --- |
| 主诉求方向 | **甲·拆两步**——先收紧门禁 + 给 `mappingSetHash`/`length` 定型；再冻结状态载荷编码 |
| 落单 | **暂不建卡**。C / D 两项与 §2.1 的门禁测量并入既有 escalations 文档的 D-1，不另起平行上报 |
| `main` 分叉 | 复核期间已自行解决，未执行重置（见 §0.2） |

**第一步的具体内容**（不依赖状态载荷如何设计，可独立开工）：

1. `replication-envelope.schema.json` 的 `body` 加 `additionalProperties:false`，或 `replication_body_errors` 改 exact-set 判定——目标是让 §2.1 的探针 1、5 转为失败；
2. `mappingSetHash` 定型（含「无映射集」时的合法取值，次诉求 C）——目标是让探针 2、3 转为失败；
3. `length` 定型或显式声明不作主张（次诉求 D）——目标是让探针 4 转为失败，或明确记载该字段无约束是有意为之。

**第二步与 D-9 合并裁决**：状态载荷的线编码即 D-9 缺失的 primitive 字节布局，勿分两次冻结。

## 6. Known gaps

- 本文**未建任何卡**（已裁决：暂不建卡）；C / D 两项并入既有 D-1。
- 三条裁决选项的代价评估基于本仓静态复核，未与 `LumioVoxelEngine`（ADR-035 chunk payload 所有者）交叉确认。
- 第一步三项改动本文只给出目标判据（哪个探针应转为失败），**未实现**——实现需另行授权开工。
