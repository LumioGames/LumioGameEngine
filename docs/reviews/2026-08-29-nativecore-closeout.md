# 2026-08-29 · NativeCore S0 收口报告(NC-W2)

> 路线图:[`../plans/2026-08-28-stepwise-convergence-roadmap.md`](../plans/2026-08-28-stepwise-convergence-roadmap.md) S0;派单:[`../plans/2026-08-28-nativecore-convergence-dispatch.md`](../plans/2026-08-28-nativecore-convergence-dispatch.md)。
> 收口执行:架构仓 TD 会话(总调度);三道工作会话 NC-A/B/C 全部交付并已通知关闭。

## 1 · 总账

| 项 | 数 |
|---|---|
| RM-00002 需求卡 | 68 |
| **已完成(done)** | **66** |
| BLOCKED(验收中停留) | 1 — R-00083(Capability bits;解铃条件 = 架构源 D-015 裁决,ADR-040 §7 明令裁决前不得从注册表派生 capability key) |
| 蓝图卡(backlog 停留) | 1 — R-00007(按其变更控制条款,待 R-00083 解铃与全量归档裁决后收尾;蓝图 r2 评论已补:Root ABI 已发布 + 受影响 R-* 处置清单) |
| 代码真值 | NativeCore `origin/main` **`e192459`**(PR #3 merge commit,9 个每卡提交保留在可达历史) |

**S0 完成条件三证**:① 提交号——`e192459`(合入)、各卡交付提交号见 §2 表;② 门禁真实输出——合后主干实跑 `cargo test --workspace` **106 passed / 0 failed**(82 suites)、`cargo clippy --workspace --all-targets -- -D warnings` exit 0、`cargo build --workspace --benches` exit 0(宿主 x86_64-apple-darwin,rustc 1.94.0);③ Workflow 读回——三批流转后逐卡读回 28/28、31/31、7/7 均 `done`,R-00083=`acceptance`、R-00007=`backlog` 与本表一致。

## 2 · 三批收口记录

| 批 | 道 | 卡数 | 验收项 | 关键证据 | 流转锚 |
|---|---|--:|---|---|---|
| 第一批 | NC-B(QA) | 28 | 113/113 通过 | 门禁 13 命令全绿 @c180bdd;**27 项 RED 变异复证 27/27 杀死,无空洞测试**;总调度抽样 R-00010/87/100/102 | 28/28 done 读回 |
| 第二批 | NC-C(QA) | 31 | 137/137 通过 | 同门禁全绿 @c180bdd;**30 项 RED 变异复证 30/30 杀死**;抽样 R-00105/114/177/261 | 31/31 done 读回 |
| 第三批 | NC-A(代码) | 7 | 30/30 通过 | reviewer 对抗审查**放行**(无 P0/P1):镜像与架构仓 `1f2ead3` 逐字节 cmp 一致、`gen-contracts` 零漂移、`dump-symbols` 0 导出、范围纪律通过、两处敏感点复核属实;合入后门禁全绿 | 7/7 done 读回 |

第三批各卡交付提交:R-00056 `14ca3ed`、R-00072 `9f52ef3`、R-00074 `3c599eb`、R-00069 `8fd9bba`、R-00079 `ee03cb6`(+测试基建 `8d507c6`)、R-00179 `93c88af`、R-00180 `c0174c7`;知识沉淀 `8ec472f`。

**RED 验收项裁决口径(2026-08-29,全室统一)**:历史「实现前失败」无实录且锚点不可回退复现——QA 批次以**变异式失效证明**替代(单点最小变异→真实失败输出→恢复→通过,两段实录);契约绑定批以 **reviewer 测试-实现互锁对抗核验**替代(同级等价)。所有替代证据评论均如实注明「不代表按 TDD 时序留有原始 RED 实录」。57 个变异全部被测试杀死,零空洞测试。

## 3 · 契约绑定交付实质(NC-A)

以架构仓 `origin/main 1f2ead3` 为绑定锚:`docs/architecture/abi/` 字节级镜像 + `.baseline.sha256` 钉 Hash + `cargo xtask gen-contracts` 生成数值 + 测试对镜像互证。ErrorCode 53 值全绑定(1001–1053,含 ADR-046 native kernel status band 1044–1053);13 个 ErrorCategory 全量单射映射;provider `lumio_core_api` 表按发布 Header 落地,C smoke 以镜像 Header 独立编译;symbol/dependency 负向 Gate 扩展到发布面。较收敛前净增 16 个测试(90→106)。

## 4 · 缺口清单与上行裁决(→ 路线图 S2/S4 输入)

1. **D-015(新,架构源)**:ADR-040 §7 的 capability 派生裁决——R-00083 唯一解铃条件。
2. **疑似上游契约缺陷(架构源)**:`lumio_core_init` 的 `out_context` 按发布签名是**按值传递的 `lumio_handle_t`**,无法把创建的 context 回传调用方;连同 `core_config_v1` body 未发布,init 槽位维持 null。需架构源裁决(改 `ptr:mut` 或明确语义)。
3. **ADR-046 §2 口径阻抗(架构源)**:use-path 期望已释放句柄浮出 InvalidHandle(1029),而 kernel arena `get()` 对空槽产出 AlreadyReleased(→1030);当前公共面不可达(init 未发布),已在 `mapping.rs` 与测试注释显式记录。需 §2 补口径或下游补 handle 卡。
4. **OperationId namespace 未发布(架构源,预期内)**:`ArchitectureOperationId`/`operation_ids()` 维持空 seam(另因 lumio-job 负向门保留);待上游发布后随小卡收敛。
5. **reviewer P2 两条(NativeCore 仓务小修,不阻塞)**:P2-1 `docs/architecture/abi/README.md:35` 生成物文件名笔误(`registry_data.rs`→`generated_data.rs`);P2-2 `lumio_core.h` 的 pin 与 `outputFiles[CHeader].digest` 等值断言未自动化。
6. **三平台 smoke Windows 腿**:维持缺口,路线图 SW 旁路(前置:Windows 机装 MSVC link.exe);Linux 腿由 CI ubuntu-latest 覆盖。
7. cargo deny / cargo audit / Miri 未执行(不在任何验收项口径内)——如需引入属新规,请用户裁决。

## 5 · 过程事故与工具反馈(已按纪律单独上报用户)

- Workflow 服务端:TTFB 1.3–2.8s/请求;chunked 响应偶发截断(IncompleteRead)→ 客户端盲重试造成重复评论(两 QA 道共 26 条,均已带 reason 软删,终态每卡评论数核验一致);建议:鉴权/RLS 缓存、chunked 断流排查、`POST /comments` 幂等键、验收项批量判定接口。
- workflow-ops 技能文档笔误:流转执行端点实为 `POST /{id}/transition`(单数;复数路径仅 GET),技能正文的「POST transitions」写法会 405。
- 一次宿主瞬时故障(clang 链接器段错误,R-00160 复证首轮)重跑即过,已排除。

## 6 · S0 之后(按路线图)

- **S1(已派)**:VoxelEngine 决策门测量(D-12 解阻),单会话一单。
- **S2(等用户)**:D-7 补审/豁免裁决 → D-2 两张卡立卡授权(执行 cwd=LumioCoreEngine)+ D-10 卡面修订。
- **裁决队列新增**:D-015 与 §4.2/4.3 两条契约缺陷进入架构源裁决面;D-1 仍是最长链,建议尽早开议。

## 附录(2026-08-29 补) · D-7 关闭与 Gate 深审 P2 台账

用户裁决「补独立审查」已执行:reviewer 对 `origin/main 1f2ead3` 终态深审,**放行——可安全 pin 进 CoreEngine 只读镜像**(独立重算 8 个 sha256;净室 canonicalizer 复现 10/10 Golden;剥空 normalization 精确退回且失败集与 ADR-041 §4 一致;Ed25519 8 向量独立库交叉验证;13 个负例真实失败含主动篡改实验;validate 191/0、generate 零漂移、三语言构建 0 warning)。D-7 残余就此关闭,三张 Gate 卡各附审查结论评论。

**P2 台账(6 条,不阻塞冻结)**:
1. typeMapping 的 C 拼写与生成代码不一致(缺 `struct` 关键字;`lumio_generate.py:1131–1137` 实际发出 `const struct …*`)——冻结记录内部自相矛盾。
2. `signedAt` 不在签名 preimage,时间窗检查不受密码学保护——Test 域可接受,**Production 域冻结前须 ADR 显式处置或 preimage v2**。
3. 时间窗比较为字典序(`lumio_contract.py:990`),分数秒时间戳会误判——潜伏缺陷,建议解析后比较或收紧 timestamp def。
4. trust 向量 payloadDigest 与现行 ManifestBody 摘要脱钩(ADR-044 改动所致)——向量自洽,建议注明 self-contained。
5. `docs/adr/` 缺 ADR-045 符号链接(spec-lint 未覆盖此检查)。
6. ADR-041 正文出现两个 §5。

同族延伸:`out_context` 按值签名问题同样存在于 `lumio_voxel_world_create` 的 `out_world`(根因:ABI 文档对 out 参用 `handle:` typeRef,方向不冻结)——修复时两处一并,并入 §4.2 台账项。
