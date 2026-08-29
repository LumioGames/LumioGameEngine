# 2026-08-29 深夜 · TD 第二班收尾交接

> 接 [`2026-08-29-td-handoff.md`](2026-08-29-td-handoff.md)(第一班)与 [`2026-08-29-td-handoff-evening.md`](2026-08-29-td-handoff-evening.md)(本班中段)。
> **那两份的真值分层、Workflow API 踩坑、已定裁决一律继承,不重复。** 本文是本班的最终状态与剩余工作。

## 0 · 本班结果

| 指标 | 班初 | 班末 |
|---|--:|--:|
| Workflow `done` | 96 | **138**(净增 42) |
| 工作项 done | 3 | **8** |
| 全室收口的仓 | 0 | **1**(NativeCore 68/68) |

**闭合的三个系统性缺口**:① Client 与 Runtime 的 **CI 准入路径**(此前所有门禁只在写它的那台机器上跑过一次);② Voxel 的 **dev-dependencies 结构性缺口**(它使五张卡的故障注入验收项**编译期永久不可达**);③ **真实执行证据** —— Voxel 25 张卡的交付评论此前无一例外记 `cargo test` 因 Windows 缺 `link.exe` 而 exit 101 未执行,本班在可链接宿主上首次补齐(158 → 172 passed)。

## 1 · 各仓 origin/main(本班末)

| 仓 | HEAD |
|---|---|
| LumioGameEngineArchitecture | `cc7b340` |
| LumioNativeCore | `e2a801e` |
| LumioVoxelEngine | `b5731b7` |
| LumioCoreEngine | `980c83f` |
| LumioGameRuntime | `218ac50` |
| LumioServer | `d6bf309` |
| LumioClient | `22202ce` |
| LumioGame | `03e8ea0` |

## 2 · 卡态(本班末)

| Room | 仓 | 总 | done | 待核销 | backlog |
|---|---|--:|--:|--:|--:|
| RM-00001 | Architecture | 12 | 10 | 0 | 2 |
| RM-00002 | NativeCore | 68 | **68** | 0 | 0 |
| RM-00003 | Voxel | 55 | 25 | **16** | 14 |
| RM-00004 | CoreEngine | 40 | 13 | 0 | 27 |
| RM-00005 | Runtime | 34 | 8 | 0 | 26 |
| RM-00006 | Server | 67 | 8 | 3 | 56 |
| RM-00007 | Client | 16 | 5 | 4 | 7 |
| RM-00008 | Game | 2 | 1 | 1(**用户终裁**) | 0 |

## 3 · 接手第一件事:canonical 缺陷的执行

**裁决已完成并合入**([`2026-08-29-canonical-object-pairs-adjudication.md`](2026-08-29-canonical-object-pairs-adjudication.md),PR #36 → `ae1054e`),**执行未完成**。

**要点**(全文见该文档,此处只给不读全文会误事的三条):

1. **Voxel 侧立即独立开工、不等架构仓** —— 可利用面完全在其内部,而架构仓侧**零调用方**。**这与直觉相反:上游缺陷不必然先修。** 在途会话已完成 W0 断代评估并在做 W1。
2. **架构仓侧删除该函数、发布自有 formId 的类型化编码器,跳基线并入 `LGE-V1.5`**(已作为「项 7」加入 V1.5 批规划,扩容授权出处已写在该项开头)。**不得声称 `CanonicalJsonV1`** —— ADR-041:22 的成员名文法把该 helper 的全部真实 key 排除在外。
3. **断代方式已裁决:硬切 + 形态标**,且**加标必须与编码改动同批交付**。

**W0 评估的关键结论(逐条查证非推断)**:断代影响面**比预估窄得多** —— 七个真实调用形态里只有两行 CHANGED;**snapshot 字节 SAME,故旧 snapshot 仍可 restore、无需迁移**;本仓无持久化 ledger,**幂等重放窗口 = 进程生命周期**。

## 4 · 剩余待核销 24 张的性质(不是一批同质工作)

| 组 | 张数 | 性质 | 在途 |
|---|--:|---|---|
| **Voxel 偶数道判不通过** | 13 | **实质返工**——QA 变异探针证实 2 张测试空洞,另有功能 bug 与「测试断言了错误行为」三处 | 结构性根因已解除(dev-deps),各卡断言待补 |
| **Voxel R-00093 / R-00119** | 2 | **总调度暂扣**——被 R-00203 独立审查点名有实质缺陷(R-00093 即那个 CRITICAL) | 等 canonical 修复 |
| **Voxel R-00203** | 1 | 第三轮复审 verdict **RETURN**,报 1 CRITICAL + 7 HIGH + 16 MEDIUM | CRITICAL 已裁决 |
| **Client R-00031/65/67** | 3 | 已派活:flaky 测试(**会卡 CI**)、五文件包哈希失真未重算 | 在途 |
| **Server R-00260** | 1 | 裁决已落卡:**39 条未裁决 findings 按 A/B/C 三级分级**,A 级处置完即可流转 | 待派活 |
| **Server R-00270 / R-00274** | 2 | absences 全批返工 / wave 3 实现 | 在途 |
| **Client R-00288 已核销**,余 T-00003 等 | — | — | 在途 |
| **Game R-00283** | 1 | **美术三方向终裁,留用户,禁区** | — |

## 5 · 待用户处理

### 5.1 R-00283 美术三方向终裁(唯一原样保留的禁区)

材料在 LumioGame `39f88c9`,推荐 **B > C > A**,含 C 的升位条件与 A 的回看条件。**本班全程未触碰**——QA 与两个交付会话均未读评论、未流转、未评论、未改相关文件。

### 5.2 待授权立卡清单(总调度有裁决权、无落卡权)

按优先级:

1. **LumioServer 两处闸门哈希可碰撞**(P1,**新发现**)—— `tools/xtask/src/contracts.rs` 的 `directory_hash_with_options`(`:1033`,`relative` 不受约束,可含 `=` 与换行)与 `generator_identity_hash`(`:471`,两文件间**无分隔符**)。**都有生产调用方,结果用于契约漂移闸门——碰撞即闸门静默放行。** 已第一手核实。
2. **架构仓 `failure-bundle` 未纳入 `CLOSED_CONTRACT_TYPES`**(阻塞 Runtime R-00138 的 S03/S07,**R-00268 交付范围未覆盖**)。
3. **CoreEngine 整体 CI 接入**(该仓与 Runtime/Client 不同,至今无对应卡;R-00266 的守护当前只在本地生效)。
4. **`spec-lint` 不校验 lessons 条目格式**(实测:五段全缺的畸形条目仍 `EXIT=0`)。
5. **PR #23 遗留 P2-1**(门禁只校验域的存在、不校验内容,ADR-041 自己 Verification 段写明的缺陷)与 **P2-4**。
6. **Voxel 五张卡的验收项 4 断言待补**(通路已解除,断言未写)。
7. **`Observability` 的 `netstandard2.1` 目标编译失败**(Runtime,**对照 worktree 已证明是既有缺陷、源自 `0754da8`**)。
8. ~~SPIKE-HYBRIDCLR P0-4~~ —— **已完成**(Client 三卡复修时按 TD 的执行约束**先做 P0-4 再算哈希**,design `:196` / `:1795` 的「8.12.0 候选」已改 `8.14.1`,合入 `45d804b`)。
9. **Host 侧 snapshot 读入必须在物化前限长**(跨仓要求,**新登记**)—— 上限值归 Host / 部署侧,**不得下沉到 codec**。依据:VOX-D-008 已裁「Host owns DAG orchestration, fsync, and Active-pointer swap」,Host 拥有落盘自然也该拥有读入时的长度约束;而在 `&[u8]` 边界加检查是**假防御**(字节已物化,内存已花掉)。**此前无任何地方登记过,只存在于一次会话推理里** —— 详见 [`2026-08-29-canonical-object-pairs-adjudication.md`](2026-08-29-canonical-object-pairs-adjudication.md) §4.2。
10. **Voxel 复杂度判据是计时测试**(`fe2b800`)—— 若日后在 CI 上抖动,**直接删判据、不要放宽阈值**(放宽会让它同时失去检出力);删掉会留下可见缺口,放宽留下不可见失效。届时需重新设计一条非计时的替代判据。
11. R-00289 Room 迁移(应属 RM-00008)· 禁用包清单历史清点 · LumioServer 全量 `pub use` 收敛为白名单 · `contracts/*.lock.toml` 的 Windows 绝对路径(致 `contracts verify` 在非 Windows 宿主整份失效)· Client 两个 adapter 测试工程零测试方法且 `dotnet test` 返回 0(与 B-00001「零测试即失败」同族)· Voxel R-00119 的 `write_occupied` 死分支(补可达用例或删分支并改写证据为「由 `&mut` 独占 + `Drop` 保证」)。

### 5.3 多平台执行环境(合并议题,不要逐仓单独凑)

- **CoreEngine R-00021/R-00022** 需 Ubuntu 22.04 / glibc 2.35 / gcc-11(实测本机无 Linux 交叉链接器 + 工具链摘要必然失配,**且已否决「另行登记 darwin 宿主」——那会削弱判据本身**)。
- **Voxel `eng/*.ps1` 至今无任何机器实跑过**;Runtime `.ps1` 同;Server `.ps1` 归 R-00282。

## 6 · 本班沉淀(已合入 `.spec/knowledge/lessons.md`)

八条,分三批合入(PR #35 / #37 及并发会话的 #30/#31)。**其中两条记的是总调度自己的失误**——判据是同类复发或影响面大,不是谁犯的:

1. 探针本身也会失效——先验证变异真落地、且打中预期那一面(三仓三例)
2. 守护的判据要做成**输入的纯函数**,反例才不必靠发货路径上的后门驱动
3. **派活提示词里写死会腐烂的值**,等于给每个执行者发一份过期真值(**TD 失误**,影响 5 个会话)
4. 怀疑失败是否自己引入,**建对照 worktree 实测**而非推理
5. 未锚定的忽略模式在大小写不敏感文件系统上**吞掉生成物**,而本机显示一切正常
6. **某处陈述被发现失真,要复跑同节其余「现状实测」**——一句过期意味着那一批都过期(**TD 失误**)
7. **拼接式编码的单射性:先说清楚靠哪条不变量**——三轮都说错的实录
8. **授权一处行为变更前,要核「触发条件的取值域」**,而不只判「该条件下这样改对不对」(**TD 失误**)

## 7 · 给下一班的三条工作方式建议

1. **派活提示词只给不可变锚点**(提交号、tag、符号名),**不给派生值**(哈希、计数、行号)。本班在这上面栽过:提示词写死 `compilerHash`,当天变两次,五个会话拿到过期值。多个执行会话独立采用了更稳做法并**主动顶回**。
2. **执行会话顶回你的口径时,先假定它是对的再去核。** 本班被顶回四次(ADR 不可改写、R-00274 依赖链、缺口分类落点、单射判据三轮),**四次都是执行方对**。它们在代码里,你在文档里。
3. **卡面文本是验收项的唯一载体**(Workflow 无原生验收项资源,`acceptance-items` 返回空)。「声称 TD 已批注而卡面零命中」本班出现三次——**卡面修订权归 TD,执行会话拒绝改是正确的,所以那是 TD 该做没做**。

## 8 · Workflow API(补第一、二班未记的)

- **评论是扁平集合**:`GET /comments?targetType=<t>&targetId=<uuid>`、`POST /comments {targetType,targetId,body}`。`targetType` 词表 `requirement` / `work_item` / `bug`;**正文字段是 `body` 不是 `content`**;嵌套路径 `/requirements/{id}/comments` 与 `/work-items/{id}/comments` **均 404**。
- **`in_review` 是「需求评审中」不是「交付待验收」**,不能直跳 `done`。链:`backlog → in_review → approved → in_progress → acceptance → done`,多数流转 `requireReason`。查 `GET /requirements/{id}/transitions`。
- **`PATCH /requirements/{id}` 整体覆盖 `description`** —— 改卡面前**必须先备份原文**。(本班用探针试端点时覆盖过一次,靠开工时拉的全量 dump 恢复。**探测写端点前先备份,或用无害负载。**)
- 服务端**剥掉正文末尾换行**,读回校验按 `rstrip('\n')` 比对,不是截断。
