# 2026-08-29 晚 · TD 总调度交接(第二班)

> 接前一份 [`2026-08-29-td-handoff.md`](2026-08-29-td-handoff.md)。那份仍然有效——**真值分层、Workflow API 踩坑、今日已定裁决(§5)、沉淀纪律(§6)一律继承,不重复**。
> 本文只记**这一班发生的变化**。事实截止:2026-08-29 深夜。所有引用的提交号均在 origin。

## 0 · 一句话现状

七仓收敛推进一大轮:**Workflow `done` 从 96 张增至 132 张(净增 36),工作项 3 → 8**;NativeCore **全室 68 张收口**;Client 与 Runtime 的 **CI 准入缺口已闭合**(今日之前所有门禁只在写它的那台机器上跑过一次)。

**但发现了一个 CRITICAL 级跨语言契约缺陷,尚未修复——见 §3,那是下一班的第一件事。**

## 1 · 各仓 origin/main(本班结束时)

| 仓 | HEAD | 本班变化 |
|---|---|---|
| LumioGameEngineArchitecture | `69568f8` | PR #23/#27/#28/#29/#30/#31/#32 |
| LumioNativeCore | `e2a801e` | R-00083 解锁收敛 + 镜像重 pin |
| LumioVoxelEngine | `0daf550` | R-00290 SHA-256 KAT;R-00203 第三轮复审 |
| LumioCoreEngine | `3246810` | R-00020 Root ABI 运行时绑定 |
| LumioGameRuntime | `072787b` | R-00284/285/286 缺陷批(**CI 准入闭合**) |
| LumioServer | `996e48a` | R-00271 契约镜像 |
| LumioClient | `22202ce` | R-00287/288/291(**CI dotnet test 双平台**) |
| LumioGame | `39f88c9` | 未动(R-00259 更正在途) |

## 2 · Workflow 卡态(本班结束时)

| Room | 仓 | 总 | done | 待核销 | backlog |
|---|---|--:|--:|--:|--:|
| RM-00001 | Architecture | 11 | 10 | 0 | 1 |
| RM-00002 | NativeCore | 68 | **68** | 0 | 0 |
| RM-00003 | Voxel | 55 | 23 | 18 | 14 |
| RM-00004 | CoreEngine | 40 | 12 | 0 | 28 |
| RM-00005 | Runtime | 34 | 7 | 1 | 26 |
| RM-00006 | Server | 67 | 7 | 2 | 58 |
| RM-00007 | Client | 14 | 5 | 4 | 5 |
| RM-00008 | Game | 2 | 0 | 2 | 0 |

工作项:done 8 / review 1(T-00003)/ todo 3(T-00004/5/9)。

## 3 · 接手第一件事:canonical 编码 CRITICAL(未修复)

**架构仓已发布的 `contract-runtime-rust` 里有一个可构造指纹碰撞的公共函数。**

`packages/rust/lumio-gen-contract-runtime/src/lib.rs:40`(生成源 `tools/lumio_generate.py:1352` 附近):

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

**三个缺陷**:① key 不转义;② value **不加引号直接拼接**(隐含要求 value 已是预编码 JSON,但无任何文档说明);③ 不拒重复 key。

**碰撞实证**:`[("a", "1,\"b\":2")]` 与 `[("a","1"),("b","2")]` 产出同一字符串 `{"a":1,"b":2}`,故 sha256 相同。

**为什么严重**:LumioVoxelEngine 用同一模式算 fingerprint(4 处:`query/plan.rs:152`、`mutation/fingerprint.rs:46`、`mutation/commit.rs:268`、`snapshot/mod.rs:34-40`),而 **fingerprint 是幂等重放判据**(同指纹返回原 receipt 且**不执行**)——碰撞 = 语义不同的请求被静默当作重放。字段值来自 C# Runtime,**属不可信输入面**。`snapshot/mod.rs` 解码侧同样按裸引号切分,**编解码一致地错,round-trip 反而通过**,这是现有测试没抓到它的原因。

**已查清的定位事实**(总调度第一手):该函数在 `ContractRuntime` artifact 里(**不在** `canonical-serializer`),故表面上不是 CanonicalJsonV1 的实现;consumers = 四仓;**零文档、零测试、零 fixture**;C# 侧**无对等物**(单侧发布面)。

**进行中**:总调度已编排一个多代理 workflow(四路调查 → 三路独立方案 → 三视角对抗评审 → 合成)产出裁决建议,**结论未出**。下一班请先读该 workflow 的合成结论,再裁决。三个候选方向分别是:补转义(签名不变)、换成值自持的构造式类型(让非法状态不可表达)、把它从公共面移除。

**连带影响**:全七仓的**重 pin 已由总调度叫停**,等此裁决落地后统一发起(否则修完 `compilerHash` 再变一次,重 pin 白做)。今日 `compilerHash` 已变两次:`e401077a…` → `870e8635…` → `0aaf61d6…`。

## 4 · 本班新增的裁决(继承,不要重开)

1. **OperationId namespace 出批**(不进 V1.5 批)。依据:ADR-040 §7 的否决带条件从句 "while the dispatch surface stays blocked",而 `packages/index.json` 的 `blocked` 实测仍含 D-009。**NativeCore 空 seam 是符合规范的终态,不是待补缺口**;重开条件唯一是 D-009 解冻 + 新 ADR 取代该条。落库:R-00269 卡评论 + PR #32。
2. **trust P2-2 走「ADR 显式处置」,P2-3 出批单修**。`trust.trustDomain` 实测为 `Test`,P2-2 原文即写明「Test 域可接受」;preimage v2 是密码学构造变更,会把批的验证面从枚举字段扩到签名字节。要求把 Production 冻结前置钉进 ADR-042 附节。
3. **LumioGame 的 generated 包 TFM 前提作废**:六个 C# 包**已是双 TFM `netstandard2.1;net8.0`**(落地 `99f94fb`),而 R-00259 设计文档写单 `net8.0` 并据此推出「消费收敛到 net10.0 单 TFM 适配工程的硬理由」——**该理由失效**,§170 的 BLOCKED 前置也已解除。S3 验收项③「不引用 `Lumio.Gen.*`」的技术前提消失,若保留须换理由。更正在途。
4. **R-00273 / R-00276 卡面已修订**:不得再手写 `MvpEnvelopeDocument` DTO 与 permission gate 执行体(ADR-048 已发布 `ReplicationEnvelope` 与 `ProtocolGate.Evaluate`);**`Body` 必须保持 `OpaqueJson`**(换具体类型即抢跑 D-009,硬红线);body required 的真值来源**从「手抄 `lumio_contract.py`」改为「运行期读镜像 schema」**(schema 被 sha256 锁住、机器可读;手抄是纸面纪律)。
5. **CoreEngine R-00021/R-00022 登记阻塞**:实测本机无 Linux 交叉链接器 + 工具链摘要必然失配(设计如此)。**否决「另行登记 darwin 宿主」**——那会削弱判据本身(P0 staging build 的意义就是在钉定环境产出可复现制品)。等 Ubuntu 22.04 环境。
6. **不交半成品**:通过条件是合取时,交付可验证的子集会在 main 上留下「主路径永不可验证」的假象,比不交更坏。
7. **R-00289 落错仓**(对象全在 LumioGame),改派;**执行会话拒绝越界是正确的**。

## 5 · 本班学到的(建议下一班沉淀进 lessons)

1. **派活提示词里写死会腐烂的值,本身就是一个坑。** 我在提示词里写了 `compilerHash` 具体值,当天它变了两次,五个会话拿到的都是过期值。正确做法:**只给提交号 + 要求执行方 fetch 后实测读取**。多个执行会话独立采用了更好的做法(`git show <rev>:<path>` 字节级 + 本机重算 + 记提交号),并主动顶回了我的错误口径。
2. **探针本身也会失效,必须验证它打中了预期的那一面。** 本班三例:Client 的 `Assert.True(false)` 被 `xUnit2020` 编译期拦下(证明的是「构建失败会红」而非「测试失败会红」);Voxel QA 的 perl 变异因大括号未转义**未真正施加**而误报 STILL GREEN;CoreEngine 的 `cargo tree` 同传 `-p` 与 `--workspace` 使 DAG 护栏**根本没在校验真实 DAG**。三例的共同点:**探针跑了、结果看着合理、但它测的不是你以为的那件事。**
3. **自过期守卫到期时,按现实翻转断言并把结论写成给下游的指令**,不是照抄卡面让测试恒红。(Server R-00271 的处置。)
4. **「按消费方式裁剪镜像范围」是会留洞的直觉。** 决定要不要镜像的是「它是不是契约真值」,不是「我这个语言用不用得上」。(Client R-00291 原方案按「C# 引用不了 crate」排除 `packages/rust/`,恰好绕开了 `packages/index.json`。)
5. **卡面文本是验收项的唯一载体**(Workflow 无独立的原生验收项资源,`GET /requirements/{id}/acceptance-items` 返回空)。**声称「TD 已批注」而卡面零命中**本班出现三次(CoreEngine R-00017、Runtime R-00112/R-00127)。**卡面修订权归 TD,执行会话拒绝改是正确的——所以这是 TD 该做没做。**
6. **计数式论证必然腐烂**,一律改「存在性 + 身份」断言。本班实例:ErrorCode 43→53、`ActivePermissionFields.Names` 15 项、BannedSymbols 17 vs 声称 18、Voxel 房间 55 vs 卡面 53。

## 6 · Workflow API 补充(前一份未记)

- **评论是扁平集合**,不是嵌套:`GET /comments?targetType=<t>&targetId=<uuid>&limit=200`、`POST /comments {targetType,targetId,body}`。`targetType` 词表:`requirement` / `work_item` / `bug`。**正文字段是 `body` 不是 `content`。** `GET /requirements/{id}/comments` 与 `/work-items/{id}/comments` **均 404**(而 `/requirements/{id}/acceptance-items` 是嵌套的,存在但返回空)。
- **`in_review` 是「需求评审中」不是「交付待验收」**,不能直跳 `done`。完整链:`backlog → in_review → approved → in_progress → acceptance → done`,多数流转 `requireReason`。可用路径查 `GET /requirements/{id}/transitions`。
- **`PATCH /requirements/{id}` 会整体覆盖 `description`** —— 改卡面前**必须先备份原文**。(本班用探针试端点时覆盖过一次,靠开工时拉的全量 dump 恢复。**探测写端点前先备份,或用无害负载。**)
- 服务端会**剥掉正文末尾换行**,读回校验按 `rstrip('\n')` 比对,不是截断。

## 7 · 待办(按优先级)

### P0
1. **canonical 编码裁决与修复**(§3)。裁决落地后统一发起七仓重 pin。
2. **Voxel 返工批**:偶数道 QA 判 **13/14 不通过**,奇数道 2 张不通过(R-00041/R-00045 已派活修证据锚定)。R-00203 第三轮复审 verdict **RETURN**,报 1 CRITICAL + 7 HIGH + 16 MEDIUM。**R-00093 / R-00119 已暂扣不核销**(被 R-00203 独立点名有实质缺陷)。
   - 结构性根因(可一次性解决):`lumio-voxel-domain` 与 `lumio-voxel-ops` **都没有 `[dev-dependencies]`**,导致 test-support 里的 `FaultInjector` 与 `DeterministicExecutor` 对这两个 crate 的集成测试**编译期永久不可达**——R-00068/76/78/96/104 缺的「故障注入 / 确定性交错 / 差分」全部卡在这一点。
   - 另有功能 bug:`bounded_port.rs` 的 `pop()` **不归还有界额度**,端口容量单调耗尽(slots=1 的端口一生只能 submit 一次);现有测试恰好 pop 后不再 submit,结构性绕过。
3. **Client 三张不通过**:R-00031(observability **真 flaky 测试**,`BoundedEventDispatcherTests.DroppableQueueFull_DropsOnlySchemaAllowedClass` 用 250ms 硬墙断宿主调度速度;**R-00287 落地后会直接卡 CI**)、R-00065/R-00067(五文件包哈希失真未重算,TD 此前已裁决须在评审时重算、未执行)。
4. **Server R-00260**(四路复核 3 路 RETURN,42 条以「已登记遗留缺陷」移交而非修复,其中 39 条未裁决)、**R-00270 返工中**。

### P1
5. **多平台执行环境**(合并议题,不要逐仓单独凑):Voxel `eng/*.ps1` 从未实跑、Runtime `.ps1` 同、Server `.ps1` 归 R-00282、CoreEngine R-00021/22 需 Ubuntu 22.04。
6. Server R-00273 起 wave 2(卡面已修订完,会话待命);Client T-00003 → T-00004(**注意:R-00291 证实 13 条别名上游从未发布,状态是「缺席」不是「解冻」,T-00004 排期需据此调整**)。

### 待用户授权立卡(总调度无落卡权)
`spec-lint` 不校验 lessons 条目格式 · PR #23 遗留 P2-1/P2-4 · CoreEngine CI 接入 · `contracts/*.lock.toml` 的 Windows 绝对路径(致 `contracts verify` 在非 Windows 宿主整份失效)· Voxel 结构性 dev-dependencies 缺口 · SPIKE-HYBRIDCLR P0-4 · R-00289 Room 迁移 · 禁用包清单历史清点。

## 8 · 禁区

**R-00283(美术三方向比稿)仍留用户终裁,本班未动。** 材料在 LumioGame `39f88c9`。
