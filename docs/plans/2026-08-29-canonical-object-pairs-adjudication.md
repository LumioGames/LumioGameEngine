# 2026-08-29 · `canonical_object_pairs` 缺陷裁决

> **状态:已裁决,待执行。** 裁决人:TD 总调度。
> 依据:一次四路调查 → 三路独立方案 → 三视角对抗评审 → 合成的多代理审议(11 个 agent),外加总调度对承重事实的独立复核(下文标「TD 已核」)。
> 复核锚点:架构仓 `origin/main` `81f7fff`。

## 0 · 一句话裁决

**缺陷真实、已发布、已扩散;但唯一可利用的那一半不在架构仓。**

- **LumioVoxelEngine 侧立即独立开工,不等架构仓** —— 可利用面完全在其内部。
- **架构仓侧删除 `canonical_object_pairs`、发布自有 formId 的类型化编码器,并入已在排的 `LGE-V1.5` 基线批** —— 需要一次新裁决把它加进该批(V1.5 规划扉页禁止自行扩容),**本文即该裁决**。

## 1 · 缺陷

`packages/rust/lumio-gen-contract-runtime/src/lib.rs:40`(生成源 `tools/lumio_generate.py:1352`):

```rust
pub fn canonical_object_pairs(pairs: &mut [(String, String)]) -> String {
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::from("{");
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 { out.push(','); }
        out.push('"'); out.push_str(k); out.push_str("\":"); out.push_str(v);
    }
    out.push('}'); out
}
```

key 与 value 均不转义;value **不加引号直接拼接**;不拒重复 key。**碰撞有三条独立路径**:① value 侧 `[("a","1,\"b\":2")]` ≡ `[("a","1"),("b","2")]`;② key 侧 `[('a":1,"b','2')]`;③ Voxel 的 `fields` 是**裸拼接**,连引号都不需要——**一个逗号就够**。

**它同时违反本仓自己已发布的两条冻结条款**:`CANONICAL_ENCODING=AsciiEscaped` 与 `CANONICAL_DUPLICATE_MEMBERS=Reject`。而它发明了一个**全仓任何文字中都不存在**的前置条件(「value 必须已是预编码 JSON」——`pre-encoded` / `预编码` 全仓零命中,crate 内 `///` 计数为 0),对应 ADR-048:16「a repository must not invent a public contract」。

## 2 · 四条承重事实(TD 已独立复核)

**F1 · ADR-041:22 的成员名文法把这个 helper 排除在外。** 原文要求每个成员名匹配 `^[A-Za-z][A-Za-z0-9]*$`,而该 helper 的**全部真实 key**(`txn_id`、`c:0:0:0`、`chunkRevision.c:0:0:0`)**一个都不匹配**。
→ **任何把它宣布为 `CanonicalJsonV1` 实现、或塞进 ADR-041 绑定面清单的方案,都是盖一个假合规章**——这是 K[28] 的同型错误(自称是 X 而不是 X)。**这一条否决了三个候选方案中的两个的核心做法。**

**F2 · 架构仓内零调用方。** 全仓仅 3 处命中:生成源、生成物、一份交接文档。**它是纯发布面**,架构仓自己一次都没用过——所以架构仓侧的修复**不会**破坏本仓任何东西,阻力只在下游。

**F3 · C# 侧完全无对等物**(`ContractRuntime.cs` 对 `anonical` 零命中),而 **ADR-039:16 把 canonical 编码职责指派给 ContractRuntime 并要求两语言 `identical observable behavior`**。即当前状态本身已违反 ADR-039。

**F4 · 当前不是已上线的远程可利用漏洞——这更正 R-00203 的定级。** Voxel 仓内**无已实现的 FFI 边界**:`root_abi.rs` 是 `Option<extern "C" fn>` 空槽表且挂 `#[allow(dead_code)]`,生产端非测试构造点只造 `fields: BTreeMap::new()`。**「字段值来自 C# Runtime」目前不成立。**
→ 当前性质是「**crate 公开 API 边界的契约缺陷**」,**Root ABI 一接线即转为真漏洞**。R-00203 的方向对、定级过重。

## 3 · 裁决

### 3.1 定级

| 面 | 级别 | 说明 |
|---|---|---|
| 架构仓 | **P1** | 已发布的公共契约缺陷,违反自己发布的两条冻结条款 + 发明未文档化的前置条件 |
| LumioVoxelEngine | **P1** | 潜伏的安全语义缺陷,**尚未上线可利用**(见 F4);Root ABI 接线即转真漏洞 |

**不降级的理由**:已扩散(Voxel 侧 `#[path]` vendor + `pub use` 再导出、**8 处真实调用**、外加 4 份各自复制的 `quote()`);修复会造成 fingerprint 语义断代,越晚修代价越大;而合并窗口**现在恰好为零冲突**(21 个 worktree 中无任何未合并分支触碰 `tools/` 或 `packages/`)。

### 3.2 架构仓侧:删除 + 发布自有 formId 的类型化编码器,**跳基线并入 V1.5**

- **删除 `canonical_object_pairs`**,代之以**类型化的构造式编码器**(值自持:字符串恒加引号并转义、整数不含分隔符),使**非法状态不可表达**,而不是给现有函数补转义。
- **不声称 `CanonicalJsonV1`、不扩 ADR-041 绑定面清单**;发布**自有 formId**(建议 `CanonicalObjectV1`)。依据 F1。
- **跳基线,并入 `LGE-V1.5`**。ADR-041:100 当初保基线的理由是「Nothing was removed」,**该前提已被本次删除推翻**;且 `baselineId` 是 `const`。
- **本文构成把该项加入 V1.5 批的新裁决**(V1.5 规划扉页禁止自行扩容)。

**为什么不补转义**:补转义仍然是拼接,单射性仍然载荷在「**所有调用方都传对了东西**」上。两个下游仓的独立自查印证了这条判断——
- LumioCoreEngine 的 `source_tree_digest` 走「拼接 + 定宽字母表约束」,单射成立**但载荷在另一个函数的一条校验上**,该仓花了三条测试才把这条依赖钉住;
- LumioGameRuntime 的 durable key 安全性**借自「47 个 schemaId 恰好都不含冒号、无一是另一个的前缀」**,而**没有任何机制守护这三条前提**。

**三方独立得出同一结论:拼接式编码的分隔符安全性必须由编码自身保证(长度前缀、转义、或禁止分隔符进入字段),不能靠「实测没撞上」。** 而「所有调用方」是个比「一条集中校验」弱得多的锚。

### 3.3 两条三个候选方案全都漏掉的构造(必须并入)

**构造 X · C# 孤代理 → UTF-8 替换字符碰撞(修完之后的新碰撞)。** Rust `String` 保 UTF-8 装不下孤代理,**C# `string` 是 UTF-16 可以**;.NET 默认 `Encoding.UTF8` 用 replacement fallback,`"\uD800"` 与 `"�"` 的 UTF-8 字节**都是 `ef bf bd`** → 两个不相等的字符串产出同一份 canonical bytes 与同一个 sha256。**Rust 侧构造不出该输入,任何 Rust 单侧测试与 Rust↔C# 差分向量表都抓不到它。**
→ **强制修法**:C# 一律用 `new UTF8Encoding(false, throwOnInvalidBytes: true)`,孤代理判为具名错误 `LoneSurrogate`;Rust 侧对称拒绝解码出的孤代理转义。**没有这一条,「修完不可能碰撞」是假的。**
→ **该条来自评审的 dotnet 10.0.400 实测,合成者未复跑;落地时必须在 C# 侧实跑确认。**

**构造 Y · 跨语言排序方向相反。** `U+1F600` vs `U+FFFD`:码点序/UTF-8 字节序 → `U+FFFD` 在前;UTF-16 码元序 → `U+1F600` 在前。Rust `String::cmp` = UTF-8 字节序 = 码点序;C# `StringComparer.Ordinal` = UTF-16 码元序。ADR-041:22 已把**码点序定为规范性**、把三序重合定为**非承重的巧合**,故 `CompareOrdinal` 实现**不合规**,即使今天没有 golden 能区分它。
→ 排序向量应放在 **ContractRuntime 自己的向量表**,不是 ADR-041 的 published goldens(10 条 golden 的成员名全集无一违反文法、无一需要转义,**打不到分歧点**)。

### 3.4 LumioVoxelEngine 侧:立即独立开工,不等架构仓

可利用面完全在 Voxel 内部:`mutation/fingerprint.rs:33-36` 的 `request.fields` **key 与 value 双双裸推、value 连引号都不加**;`snapshot/decode.rs:74-86` 按裸引号切分**与编码侧错得一致**,`decode.rs:19-23` 的 recanon 守卫**按构造恒过**(实测:编码方 2 pair / 解码方 3 pair,字节与摘要全等,守卫无察觉);`tests/mutation_receipt.rs:79-90` 的 `expected_fingerprint` **复刻了同一个 bug,断言在有无转义时恒成立**。

**碰撞的实际后果**(已由调查方追出完整链路):先提交 B 并 finalize,再提交 A,`commit.rs:45-54` → `check_entry`(`receipt_ledger.rs:216-221`)指纹相等 → `Duplicate` → **直接返回 B 的 receipt,A 从未执行而调用方拿到成功回执**。正确转义下应是 `RevisionConflict`。

**fingerprint 语义断代不可避免**:现有字节可伪造 ⇒ 必须变 ⇒ 全部历史幂等重放判据失效。**这是产品决策,须先于代码裁决。**

### 3.5 否决项

- **否决**「先合加法半、暂不删旧函数」的退路 —— 中间态会**同时存在两条公共编码路径**。
- **否决**把该函数宣布为 `CanonicalJsonV1` 实现的任何变体(F1)。
- **否决**把 value 文法窄化到「对象在任何深度一律拒绝」的方案 —— **10 条 golden 里 9 条含嵌套容器值**,该方案结构上不可能复现它们。

## 4 · 一个独立发现:六个下游 pin 全部过期,LumioServer 钉在 K[28] 修复**之前**

**TD 已核**:`LumioServer/contracts/architecture-contracts.lock.toml:38` 的 `artifact_contract_runtime_hash = "ee8fa744…"` —— 该值是 **K[28] 圆常量修正(`bcc8eb9`)之前**的算错哈希。即该仓至今**锁着一个用错误 SHA-256 实现算出来的摘要**。

**这与本裁决同批处理**:七仓重 pin 已由总调度叫停,等本裁决落地后统一发起,届时给稳定锚点(D-5 冻结点 tag 已在 V1.5 批规划内)。**Server 这一条要单独点名核验**,不能混在批量里静默带过。

## 5 · known gaps(诚实登记)

1. **单射性是被断言的,不是被证明的。** 结构性论据(每个字符串被引号包住并转义、每个整数不含分隔符,故 `",:{}[]` 只在结构位出现)**只能写在 ADR 里,机器读不到**。本仓无 fuzz 基建。
2. **架构仓的修复是必要不充分。** value 侧真正的入口在 Voxel;删除虽强制编译中断,但**只对重新 vendor 的仓生效**。
3. **迁移期有一处「编译通过但摘要不同」的静默面**:裸 value `"7"` 迁移时选 `Str("7")` 得 `"7"`、选 `Int(7)` 得 `7`,**两种都编译通过、摘要不同**。缓解(强制):交回物必须含对**真实历史请求样本**跑出的前后 digest 对拍表,不接受「重新 vendor 完成」。
4. **其余 3 个 consumer 仓(Client / Runtime / Server)是否有同型 `quote()` 复制未核** —— 必须作为审计任务下发,**不得默认为「无」**。
5. **Python canonicalizer 是事实上的规范且零测试**(`lumio_contract.py:34` `import lumio_generate as _abi`,goldens 的「重算」用的就是生成器自己的 Python 实现)。本裁决未修这个根本问题。
6. **构造 X 未由合成者复跑 dotnet**,落地时必须在 C# 侧实跑确认。
7. **「必须并入 V1.5」这条约束由人的动作守护,不由机器。** 若被当成普通 fix 批合入 main,下游会得到一个 **baselineId 没变、公共面却不兼容**的 artifact。

## 6 · 执行顺序

| 道 | 内容 | 前置 |
|---|---|---|
| **W-1** | 审计其余 3 个 consumer 仓是否有同型 `quote()` 复制(消灭 gap 4 的推断) | 无,可立即并行 |
| **W0** | **Voxel fingerprint 断代的产品裁决**(历史幂等重放判据全部失效,如何处理) | 无,须先于代码 |
| **W1** | Voxel 侧修复(转义 + 拒重复 key + 修 decode 侧 + 修那条恒成立的测试断言) | W0 |
| **W2** | 架构仓侧:新编码器 + 删除旧函数 + 新 ADR + 向量表(含构造 X / Y) | 并入 V1.5 批 |
| **W3** | 七仓统一重 pin(**LumioServer 的 `ee8fa744` 单独点名核验**) | W2 + V1.5 跃迁完成 |

**注意 W0/W1 排在 W2 之前**:可利用的那一半在 Voxel,而架构仓侧零调用方、无自身风险。**这是本裁决与直觉相反的一点——上游缺陷不必然先修。**

---

*本裁决的多代理审议产出(四路调查、三路方案、三视角评审、合成稿)存于会话工作区,未入库;本文是其可执行结论的成文版。三个候选方案没有一个可原样落地的详细论证亦在其中。*
