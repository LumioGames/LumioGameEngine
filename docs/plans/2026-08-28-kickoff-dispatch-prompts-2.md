# 2026-08-28 · 各仓开工派活提示词（第二轮）

> 配套报告：[`../reviews/2026-08-28-seven-repo-progress-audit-2.md`](../reviews/2026-08-28-seven-repo-progress-audit-2.md)（12:21–12:28Z 测量）。
> **派活方式：`spawn_task` 新开会话，带 `cwd` 指向目标仓目录，只粘提示词。** 不用 SendMessage 派活（会打断对方在途工作）；对方用 SendMessage 回报，我再用 SendMessage 回复。
> 提示词只做三件事：**指路 / 立规 / 设禁区**。工作内容本体在 Workflow 卡上，提示词不复述卡面。

## 公共纪律（每份提示词都已内嵌，此处不重复粘贴）

① 领卡先流转「实现中」并写 reason；② 证据评论只引用**已推送 origin** 的提交号，先 push 再回写；③ 测试证据必须是链接执行的真实输出，`cargo check` / `dotnet build` 不算测试证据；④ 交付 = 改动清单 + 验证证据 + known gaps + 沉淀落点，本仓收口门槛必过；⑤ **公共契约缺口 → 停，卡上标 BLOCKED 上报，不本地绕过**；⑥ 只动本仓文件；⑦ 做完流转「验收中」，「已完成」由总调度核验后流转；⑧ 动手前开隔离 `git worktree`，不在共享工作区切分支。

---

## W0 · 解阻波（必须先完成，不完成不进 W1）

### W0-1 · 架构源 —— D-1 状态载荷与上行承载

> **归属排他**：该仓当前多会话并发，本卡需明确单一归属，避免重复裁决。

```
【仓】~/LumioGames/LumioGameEngineArchitecture（架构源）
【已定方向】甲·拆两步。依据 docs/reviews/2026-08-28-replication-state-payload-adjudication.md（已在仓内）。

第一步（本次范围，不依赖状态载荷如何设计）：
1. 收紧 replication body 门禁——replication-envelope.schema.json 的 body 加 additionalProperties:false，
   或 tools/lumio_contract.py 的 replication_body_errors 改 exact-set 判定。
2. mappingSetHash 定型：它当前在任何 Schema 里都没有类型定义，需补类型 + 定义「无映射集」时的合法取值。
3. length 定型，或显式声明不作主张。

【验收判据（可机器判定）】上述报告 §2.1 的五个探针必须从「全部 PASS」转为「对应项 FAIL」：
  探针1 FullSnapshot.body 注入私有载荷 / 探针5 Ack 夹带命令 → 由第 1 项拦下
  探针2 mappingSetHash=42 / 探针3 =null                     → 由第 2 项拦下
  探针4 length=999999999                                     → 由第 3 项拦下或明确记载为有意不约束
【顺序】严格走 ADR → Schema/ID → 正反 Fixture → README/Baseline → 七仓镜像。
【禁区】不解冻 D-009/D-011；不为「有正反例」造假 Fixture；第二步（状态载荷线编码）不在本次范围，
       它与 D-9 二进制 canonical 合并裁决，勿分两次冻结。
【收口门槛】node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs
           && python3 -m py_compile tools/lumio_contract.py && python3 tools/lumio_contract.py validate
```

### W0-2 · 架构源 + CoreEngine —— `architecture.lock` 升级

> **两张卡严格串行**，都改 `sync-architecture.sh`、都触发 `compilerSha256` 重生成，不可并行。

```
【仓】~/LumioGames/LumioCoreEngine（卡①②）；架构侧待澄清项回架构源
【现状（本轮第三方独立复核）】architecture.lock.json = LGE-V1.2-2026-08-27 / 2d7980d95b16 / requiredPaths 131；
  packages/ 不在 requiredPaths 内 → 物理上无法消费 V1.4 产物。

卡①「升级 lock 到 V1.4」：改 tools/sync-architecture.sh:50-51 两个常量 + --update-lock + 镜像目录改名。
  三个陷阱：唯一删除项 fixtures/invalid/processor-read-write-conflict.json 必须与 140 个新增在同一次
  --update-lock 处理；改脚本会变其 SHA 而 compilerSha256 与之强耦合，必须重生成 generation-record.json；
  镜像内 ids/index.json 的 baselineId、lock 的 architectureBaselineId、镜像目录名三处必须同步。
  ★ 卡面必须写明「本卡不解除任何下游阻塞」——R-00015 要的东西全在 packages/ 下，卡①的五前缀投影不含它。
卡②「扩展投影规则纳入 packages/」：改 project()(63-69)、--update-lock 枚举表(197-198)、守卫串(185)。
  属 R-00012 文件集，不得由 R-00015/R-00018 顺手改。

【守门第一步】确认架构源 origin/main 上 packages/abi 与 packages/canonical 均存在（pin 必须 ≥ a4a7956）；
  不满足即停并回报，不得绕过重做。
【待架构侧澄清（先问再动）】packages/rust/Cargo.lock 是否进镜像；packages/.gitignore 与 README 是契约还是仓务文件。
【禁区】ADR-040/041 若仍是 Draft，把 Draft 公共构造冻进只读镜像会在转 Accepted 时触发 300+ 路径重登记——
       是否等转 Accepted 再做卡②，先回报等裁决。
```

### W0-3 · 架构源 —— D-3 generated 面能力边界

```
【仓】~/LumioGames/LumioGameEngineArchitecture
【需裁决】packages/csharp/ 六个包只给目录表（8 文件 437 行），没有 ReplicationEnvelope 等类型本体，
  ProtocolPermissionValidator 只有 15 个字段名字符串、无可执行校验方法——而 ADR-022:42 明文
  「Hand-written per-repo validators were rejected for drift」。
【阻塞对象】GameRuntime 26 张（R-00138 S03/S06/S07、R-00139 S04/S06、R-00141 S02、R-00149、R-00150 S04）、
  Client、Server 三仓被迫各写手写 DTO 与手写 gate。
【裁决要回答】generated 面只给目录表，还是给类型本体 + validator/builder？
  若裁决 catalog-only，必须一并说明「不得自行发明公共合同」与「必须调用 generated validator」
  这对约束下游怎么解——尤其 ID ordinal 的权威来源在哪里。
【合并处理】LumioGameRuntime 已就同一前提提请求，与本项合并成一次裁决。
```

### W0-4 · 总调度 —— NativeCore 67 张验收流转

```
【性质】不是开发任务，是流转欠账清理。
【依据】本轮抽样 R-00056/00075/00107/00144/00179 的证据 SHA 六个全部实测为 origin/main 祖先，证据质量合格。
【做法】按模块批量走验收流转，不逐卡人工考古；证据已足，不需重跑测试。
【前置】需用户先裁定「验收责任人 / 批量放行规则」——当前无人定义谁来做验收，这是积压根因。
【禁区】验收未实跑的卡最多流转到「实现中」，不越级到「已完成」。
```

---

## W1 · 解阻后并行（异仓并行、同仓串行）

### LumioClient —— 补卡后按模块流转

```
【仓】~/LumioGames/LumioClient
【问题】origin/main 上有 11 个模块 / 242 个 .cs / 12,224 行 / 130 个测试实体，
  但 Workflow 里 10 张卡中没有任何一张实现卡——12k 行产出全部兜在 R-00031 一张计划卡下。
【第一步（需用户先裁定粒度）】按 11 个模块建 11 张，还是按 Wave 0-6 建 7 张，或只补一张总卡挂证据？
  ★ 建卡是写操作，必须先取得用户明确授权（确切对象清单 + 数量），不得自行决定。
【第二步】补卡后为已交付模块补证据评论（引用 origin/main 提交号）并流转到「验收中」。
【第三步】4 张 SPIKE（R-00253/254/255/256）可与补卡并行，文件集互不重叠。
```

### LumioGameRuntime —— 等 W0-3

```
【仓】~/LumioGames/LumioGameRuntime
【守门第一步】确认架构源已就 D-3 给出裁决；未裁决即停并回报，不得用手写 DTO 或自研 validator 顶替。
【现状】2,064 行 / 22 测试 / 5 张 in_progress；6 张 wave 卡已正确判「前置未满足、未开工」——这个处置是对的，保持。
【解阻后】R-00138 → R-00139 → R-00141 → R-00149 → R-00150 按卡面依赖顺序。
```

### LumioServer —— 等 W0-1，可先做 A1-α

```
【仓】~/LumioGames/LumioServer
【现状】modules/ 下真实现仅 44 行（6,844 行是 xtask 治理工具，950 行 testkit）——业务实现确实未开始，48 张 backlog 准确。
【可立即开工】A1-α（WSS 握手 → admission → FullSnapshot → BaselineAck → revision 前进 → DeltaAck → 断连重连
  Full Resync）只依赖已冻结面，不受 D-1 影响。设计已在 origin/main:docs/specs/2026-08-28-mvp-csharp-host-design.md。
【阻塞】A1-β（第二个客户端看见方块被挖）等 W0-1，保持 BLOCKED，不得本地绕过。
【保持】出站 body exact-set 自律断言继续保留——公共门禁当前拦不住多余字段（报告 §2.1），这条自律是有效的。
```

### LumioVoxelEngine —— 流转欠账

```
【仓】~/LumioGames/LumioVoxelEngine
【现状】25,041 行 / 186 测试，全项目实现量最大；25 张 in_review + 14 张 acceptance。
【本轮未核验】其 in_review 卡的评论本轮未读，需先逐卡核证据再决定流转，不得据本报告直接批量放行。
```

### LumioGame —— 内容规格

```
【仓】~/LumioGames/LumioGame
【现状】origin/main 零源码，1 张卡（R-00259 acceptance，设计文档卡）。
【依赖】MVP 内容规格依赖 D-1 裁决后的世界状态表达方式，先做不依赖该裁决的部分。
```
