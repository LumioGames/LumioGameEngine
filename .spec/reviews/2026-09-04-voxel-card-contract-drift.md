---
name: 2026-09-04-voxel-card-contract-drift
description: 体素九张需求卡与 voxel-world-v1 契约的逐条漂移复核——每卡判定、rule/errorCode 全量覆盖矩阵与排期缺口;派活或改这批卡前查
metadata:
  type: doc
  status: 已交付
---

# 体素需求卡 × 契约漂移复核（2026-09-04）

对线上蓝图 `voxel-impl-2026-09-04` 九张卡（R-00432…R-00440）与 [`voxel-world-v1.json`](../../engine/wire/voxel-world-v1.json) 的一次**只读**复核。产出判定与缺口；当时**未改动任何线上单据、契约、ADR 或设计文档**，需要改的全部列在第六节等 Owner 裁决。2026-09-05 Owner 已对 R-00434 的 D1/D2/D3 授权，落地结果见本文末尾增量记录与 [ADR-066](../decisions/ADR-066-voxel-owner-rulings.md)。

## 一、复核口径

### 真值优先级

1. `engine/wire/voxel-world-v1.json`（`lumio.voxel-world.v1`）
2. [`ADR-062`](../decisions/ADR-062-voxel-world-public-contract.md)
3. [`knowledge/features/voxel.md`](../knowledge/features/voxel.md)

契约与文档冲突时以契约为准。

### 三方一致性（写操作防呆，已核验通过）

| 检查点 | 值 |
|---|---|
| `.workflow` 解析出的 profile | `lumiogamesengine` |
| config 的 `base_url` 子域 | `lumiogamesengine` |
| `/projects/current` 的 `project.subdomainPrefix` | `lumiogamesengine` |
| 项目 | `LumioGamesEngine`（`proj_b697…`），`status=active`，无 `publicDemo` |

Token 仅经环境变量携带，全程未回显。本次为纯读，未发生任何 POST/PATCH。

### 取数完整性

- 九张卡按四路读全：正文 + 验收项 + 评论 + 附件。**九卡附件均为 0**。
- 验收项共 **44 条**，全部 `systemSemantic=not_started`；**九卡状态全部为 `backlog`**。
- 七张卡（R-00433/434/435/436/437/438/439）各带 1 条 PR #81「契约已递进」评论；**R-00432 与 R-00440 无评论**。
- `/requirements` 以 cursor 翻页至 `nextCursor` 为空（9 页 / 440 条），确认本批次即 R-00432…R-00440，无编号更大的遗漏卡。相邻的 R-00422（炸弹人 Stage 0 来源卡）是报回四处缺口的消费方，不在本次范围。

## 二、P0：PR #81 尚未合入 main，多张卡的前置状态失真

| | `main`（`f2d4564`） | 当前分支 HEAD（`f7f8a0d`） |
|---|---|---|
| errorCodes / rules / 用例 | **44 / 49 / 98**（47+51） | **51 / 56 / 110**（53+57） |
| `identity.cellOffset` | ✗ 无 | ✓ 有 |
| `residency.pinnedRegions` | ✗ 无 | ✓ 有 |
| 顶层 `behaviorTemplates` | ✗ 无 | ✓ 有 |
| 顶层 key 数 | 26 | 27 |

```
$ git merge-base --is-ancestor f7f8a0d main && echo YES || echo NO
NO 未合入 main
$ git branch --contains f7f8a0d
* docs/2026-09-04-voxel-consumer-gaps
```

**后果**：七条评论以既成事实口吻宣布「契约已递进（2026-09-04，PR #81）」；**R-00440 更把 `residency.pinnedRegions` 写进「前置产物(接口)」并标注「已冻结，PR #81」**——该块在 `main` 上不存在。任何 worker 按卡里写的「架构仓 `main`（`f2d4564`）」去拉契约，拿到的都是**没有 cellOffset、没有 pin、没有 behaviorTemplates** 的旧版本，而评论却要求「不要按本卡创建时的快照实现」。两者互相矛盾，worker 无从解。

已在 `main` 成立、不受此影响的变动：`page` 概念整体删除（`main` 与 HEAD 全文 `page` 均 **0 处**，块名为 `sectionPayload`）、BlockType 段表 bit 23 + 256 分界（旧魔数 `9999`/`10000`/`2000000` grep **0 命中**）、载荷第四编码 `Delta`，以及 `blockCatalog` / `blockRead` / `blockWrite` / `physicsQuery` / `blockEntityBinding` / `assetLibraries` 六块。

**本报告以 HEAD（`f7f8a0d`）为对照基准**——因为需要复核的近期变动只在 HEAD 成立；「未合入 main」列为第一号待裁决项。

## 三、逐卡判定

### R-00432 — 契约来源切换 + chunk→section 改名 · **已过时应关闭**

`backlog` / P1 / **0 验收项 / 0 评论 / 0 附件**。

内容本身与契约**无冲突**：两条键正则与 `identity.sectionKey.pattern`、`identity.chunkKey.pattern` 逐字一致；「三坐标 `c:` 键必须显式拒绝」对应 `identity.arityIsTheGuard`；`layering.chunk-carries-no-data` 为红线的说法对应 rule 6；点名的七个错误码（`unknown_section_key` / `unknown_chunk_key` / `section_y_out_of_range` / `coordinate_out_of_bounds` / `section_unavailable` / `stale_section_revision` / `dirty_section_not_durable`）**全部存在于今天的 `errorCodes`**。

判定依据不是内容错，是**已被裁定取代**：R-00433 决策账本第 7 行「`refactor/section-chunk-rename` 分支停止推进，R-00432 由本批次取代（Owner 2026-09-04）」；ADR-062 迁移段同述。它也是唯一没收到 PR #81 通告的实现卡。

> ⚠ **关闭前必须先安排承接**：R-00432 是 rules 1–5（`key.section.arity` / `key.chunk.arity` / `key.canonical` / `key.section.y-range` / `key.coordinate-bounds`）与 rule 12（`residency.dirty-needs-ack`）的**唯一覆盖卡**。直接关闭会让 6 条规则、5 个错误码变成孤儿——详见第四节。

### R-00433 — 原始需求（来源真值卡）· **需改正文**

- **契约计数过期**：正文两处写「44 错误码 / 49 规则 / 98 用例」，为 PR #80 后、PR #81 前的快照；今天是 51 / 56 / 110。评论已更正，正文未同步。
- **决策账本缺四行**：现有 7 行覆盖 page 删除、bit23+256、玩家素材库只挂官方模板（rule 29）、Delta 必带 base、物理查询进 wire、y 竖直无符号、R-00432 被取代——**但没有** `identity.cellOffset` 算式（rule 50）、`residency.pinnedRegions`（rules 51–54）、`behaviorTemplates` v1 穷尽（rule 55）、单格读 `presence` 必答（rule 56）。这四项已冻结进契约，却只活在评论里，没进这张「来源真值与变更控制记录」卡。
- **交付轨道过期**：「轨道 A：I-1 → I-2 → {I-3, I-4, I-5}」**未含 I-6**（R-00440，蓝图 `r2/I-6`）。「轨道 B：A-1 → {A-2, A-3}」中 **A-2 / A-3 至今没有对应卡**（本批次只有 A-1 = R-00439）。

### R-00434 — I-1 段表 / 配表 / 材质类 / BlockState 位段 · **需补验收项 + 需改正文**

契约常量逐条核对，**8 个全部命中**（均在 `limits`）：

| 卡里的断言 | 契约路径 | 值 | 结论 |
|---|---|---|---|
| `blockTypeScopeBit=23` | `limits.blockTypeScopeBit` / `blockId.scope.bit` | 23 | ✓ |
| `blockTypeScopeMask=8388608` | `limits.blockTypeScopeMask` | 8388608 | ✓ |
| `systemReservedTypeMax=255` | `limits.systemReservedTypeMax` | 255 | ✓ |
| `firstOfficialBlockType=256` | `limits.firstOfficialBlockType` | 256 | ✓ |
| `worldYMin=0` / `worldYMax=255` | `limits.worldYMin` / `worldYMax` | 0 / 255 | ✓ |
| `blockTypeMax=16777215` | `limits.blockTypeMax` | 16777215 | ✓ |
| `blockStateMax=255` | `limits.blockStateMax` | 255 | ✓ |

段表七段（`0` 空气 / `1` 错误块 / `2` ECS 占用 / `3` 结构占位 / `4-255` 预留 / `256-8388607` 官方 / `8388608-16777215` 房间局部）与 `blockId.typeSegments` 逐条吻合；`room_local_index() = BlockType & 0x7FFFFF` 与 `blockId.scope.roomLocal.localIndex` 一致；门位段 2+1+1+1、液面 4 位与 `blockId.stateLayout` 一致；`sectionY = worldY >> 4` / `cellY = worldY & 15` 与 `identity.axes.worldY.derivation` 一致；「旧魔数作废」经 grep 证实（契约内 0 命中）。

**缺口**——评论明令「原验收 7 条仍成立，另需覆盖这两处」，而 7 条验收项一条都没对应：

1. `identity.cell-offset-formula`（rule 50）+ `cell_offset_out_of_range`（契约新增 2 个 testCase、1 个 invalidCase）——**无验收项**。
2. `catalog.behavior-template-must-be-registered`（rule 55）+ `unknown_behavior_template`——**无验收项**。

正文亦未同步：详细要求 #5 只把 `behaviorTemplate` 列为目录六字段之一，未要求解析到 `behaviorTemplates` 登记表；全文无 `cellOffset` 算式。

### R-00435 — I-2 Section 三态存储 · **一致**（建议补 1 条 P2）

块名引用正确（`sectionPayload`，非已删除的 `sectionPage`）。逐条命中：三态条件与 `cellBytes` 4096 / 16384（`sectionPayload.encodings`）；8 位唯一索引宽（`singleIndexWidth`）；256 位位图 32 字节、有死槽直接覆盖、全活才升级（`paletteSlotReclamation`）；信封五字段与摘要先于解释（`envelope.required` / rule 9）；`palette_overflow`、`section_encoding_mismatch`、`palette_reclaim_before_escalation`、`dead_palette_entry_in_payload`、`section_digest_mismatch`、`chunk_carries_data` 六码全部存在。8 条验收项与正文验收标准一一对应。

**建议（P2）**：评论新增「按格索引必须使用写死的 `cellOffset` 算式」这一强制要求，但声明「原验收 8 条不变」——该约束因此没有任何机器可判的验收钩子。R-00435 是信封的所有者，`envelope.conditional.baseSectionRevision`（「其余三种全量编码不得携带」）也落在它的边界内，与 R-00436 验收 #4 是同一处接缝。

### R-00436 — I-3 Delta 编解码 · **一致**（暴露 1 处契约缺口）

`Delta.bytesPerEntry: 6`、偏移 0~4095 + 32 位 `BlockId`、`requiresBase`、`neverFirstDelivery`，以及 `payload.delta-needs-matching-base`（rule 32）/ `payload.delta-not-for-first-delivery`（rule 33）与 `delta_base_revision_mismatch` / `delta_used_for_first_delivery` 全部命中。5 条验收项与正文一致。

> ⚠ **契约缺口（非卡缺陷）**：验收 #4「全量编码携带 `baseSectionRevision` 时被拒」在契约里**只有 `sectionPayload.envelope.conditional` 的散文表述，没有对应 rule、没有对应 errorCode**。这条验收项无码可引，实现方只能自拟拒绝码——公共语义出现分叉的典型入口。

### R-00437 — I-4 玩法侧批量读 · **需补验收项 + 需改正文**

`blockRead.budget.maxCellsPerRequest = 262144`（= `limits.maxCellsPerReadRequest`）、四态 `Ready`/`Unchanged`/`Pending`/`Unavailable`、y/z/x 铺开序（与 `identity.cellOffset.order` 声明的「两处不得出现第二种序」一致）、`read_budget_exceeded` / `read_result_missing_revision` / `section_unavailable` 均命中。

**缺口**：评论**自身明文要求**「新增 `read.cell-carries-presence` 规则与 `cell_read_missing_presence` 错误码，**本卡需补一条对应验收**」——6 条验收项里没有。这是九张卡里唯一一处评论直接点名要补而未补的。正文详细要求 #1 描述单格读时也未写明「`presence` 必答、缺块不给 `BlockId` 字段」。

### R-00438 — I-5 结构化逐格写 · **需补验收项**（P2）

`blockWrite.batch.maxEntriesPerBatch = 65536`、条目四字段（`sectionKey` + 0~4095 偏移 + `BlockId` + `expectedSectionRevision`）、prepare/commit 两段式、`unstructured_mutation_entry` / `stale_section_revision` / `write_batch_too_large` 均命中。6 条验收项与正文一致。

- **建议（P2）**：评论新增 `cell_offset_out_of_range` 的越界校验要求，声明「原验收 6 条不变」，该错误码因此在全批次内无任何验收覆盖。
- ⚠ **契约缺陷**：rule 49 `write.batch-is-all-or-nothing` 的**文本**是「任一条目校验失败即整批拒绝，不得部分应用」，其 `onViolation` 却挂 `write_batch_too_large`（尺寸超限错误）。契约里**不存在**表达「批被部分应用」的错误码；而 `write_batch_too_large` 的真正语义出处 `blockWrite.batch.onExceeded` 反而没有对应 rule。看上去是错位。本卡验收 #2 与 #4 分别落在两种语义上，实现方会撞见这个矛盾。

### R-00439 — A-1 native-abi.json 体素 slot · **需补验收项 + 需改正文**

262144 / 65536 / 四态显式字段 / `BlockId` 无符号（`blockId.unsignedDiscipline`）均与契约一致。验收 #6 的「`verify-wire.mjs` 7 份契约全绿」经核实**当前确为 7 份**（`account-port` / `entity-binding-and-query` / `gameplay-command-envelope` / `hello-wire` / `native-timer-abi` / `platform-port` / `voxel-world`）——属实，但写死份数的断言会在第 8 份契约落地时假失败。

**缺口**——评论要求三项，验收项与正文均未承载：

1. `cellOffset` 算式与取值范围要写进 ABI doc（跨语言两端不得各推一次）。
2. 单格读返回结构里 `presence` 必答、`BlockId` 可缺失，不得零值顶替。
3. **为 `residency.pinnedRegions` 预留 pin 声明与就绪查询的 slot**。

第 3 项是**正文未承载的范围扩张**：详细要求 #2 的函数表只列「批量读 / 写入提交 / `SectionRevision` 查询」，卡标题同样只写这三项。ABI 是跨仓冻结面，slot 集合漏一项要再走一轮生成与合入。

### R-00440 — I-6 区域常驻 pin · **内容一致 / 前置失真**

与契约逐条吻合：四条规则 `residency.pin-guarantees-ready`(51) / `pin-budget-is-hard`(52) / `pinned-never-evicted`(53) / `pin-ready-before-gameplay`(54)，四个错误码 `pinned_read_returned_pending` / `residency_pin_exceeds_budget` / `pinned_section_evicted` / `pin_region_not_ready`，以及「不参与流式调度」（`notStreaming`）、「干净也不许卸载，强于脏页栅栏」（`neverEvicted`）。验收项里的 `61×61` 格 2 格高 → `4×4×1 = 16` 个 Section ≈ 64 KB 与 `residency.pinnedRegions.sizing` 逐字一致。

**问题两处**：

1. **前置失真**：前置产物写「`residency.pinnedRegions` 块与 `residency.pin-*` 规则（已冻结，PR #81）」，但该块未在 `main`（第二节）。这张卡的前置在 main 口径下不成立。
2. ⚠ **契约缺口**：`residency.pin-budget-is-hard` 与 `residency_pin_exceeds_budget` 依赖「声明的驻留预算」，而**契约的 `limits` 里没有任何驻留预算字段或常量**，`pinnedRegions` 块内也没有。验收项 2「声明超出驻留预算的 pin 当场失败」因此无机器可校验的界。（`voxel.md` 第 127 行有个 `maxResidentChunks = 4096`，但它不在契约里，且量纲是 Chunk，与 `sizing` 明说的「pin 的成本按 Section 算」不同。）

## 四、覆盖矩阵：56 条 rule

`✓` = 有卡且有对应验收项；`△` = 卡正文提到但无验收项（弱覆盖）；`▲` = 评论要求补但尚无验收项；`✗ GAP` = 无任何卡覆盖；`⊘` = 仅 R-00432 覆盖，该卡拟关闭后成为孤儿。

| # | rule id | onViolation | 覆盖 | 卡 / 说明 |
|---|---|---|---|---|
| 1 | `key.section.arity` | `unknown_section_key` | ⊘ | R-00432（拟关闭） |
| 2 | `key.chunk.arity` | `unknown_chunk_key` | ⊘ | R-00432（拟关闭） |
| 3 | `key.canonical` | `unknown_section_key` | ⊘ | R-00432（拟关闭） |
| 4 | `key.section.y-range` | `section_y_out_of_range` | ⊘ | R-00432（拟关闭） |
| 5 | `key.coordinate-bounds` | `coordinate_out_of_bounds` | ⊘ | R-00432（拟关闭） |
| 6 | `layering.chunk-carries-no-data` | `chunk_carries_data` | ✓ | R-00435 验收 7 |
| 7 | `payload.palette-cap` | `palette_overflow` | ✓ | R-00435 验收 1 |
| 8 | `payload.encoding-matches-content` | `section_encoding_mismatch` | ✓ | R-00435 详求 6 + 用例全覆盖 |
| 9 | `payload.digest-before-interpretation` | `section_digest_mismatch` | ✓ | R-00435 验收 6 |
| 10 | `presence.missing-is-not-air` | `section_unavailable` | ✗ GAP | 改动层派发面无卡（R-00437 只覆盖读侧 rule 46） |
| 11 | `presence.short-ticket-is-zero-bytes` | `section_encoding_mismatch` | ✗ GAP | 零字节短票无卡 |
| 12 | `residency.dirty-needs-ack` | `dirty_section_not_durable` | ⊘ | R-00432（拟关闭）；R-00440 只管 pin |
| 13 | `residency.ack-covers-declared-bound` | `stale_section_revision` | ✗ GAP | 落盘回执无卡 |
| 14 | `lighting.never-on-the-wire` | `lighting_in_payload` | ✗ GAP | R-00433 已列为非目标（已知不排期） |
| 15 | `material.class-is-a-type-attribute` | `material_class_not_a_cell_lane` | △ | R-00434 详求 7，无验收项 |
| 16 | `material.class-must-be-declared` | `unknown_material_class` | △ | R-00434 详求 5，无验收项 |
| 17 | `material.no-texture-only-class` | `unknown_material_class` | ✗ GAP | 无卡提及 |
| 18 | `material.no-cross-class-merge` | `cross_material_face_merge` | ✗ GAP | 网格面非本批次 |
| 19 | `liquid.static-in-v1` | `liquid_auto_propagation_unsupported` | ✗ GAP | 无卡 |
| 20 | `binding.block-implies-live-entity` | `entity_binding_missing` | ✗ GAP | `blockEntityBinding` 整块无卡 |
| 21 | `binding.no-orphan-entity` | `entity_binding_orphan` | ✗ GAP | 同上 |
| 22 | `binding.entity-type-matches-block-type` | `entity_binding_type_mismatch` | ✗ GAP | 同上 |
| 23 | `binding.reference-table-is-sparse` | `entity_binding_not_sparse` | ✗ GAP | 同上 |
| 24 | `binding.two-halves-share-one-commit` | `binding_commit_split` | ✗ GAP | 同上 |
| 25 | `binding.business-data-never-on-the-wire` | `business_data_in_payload` | ✗ GAP | 同上 |
| 26 | `blockType.scope-bit-is-authoritative` | `block_type_scope_violation` | ✓ | R-00434 验收项 10 |
| 27 | `blockType.system-reserved-range` | `system_reserved_type_misuse` | ✓ | R-00434 验收项 3 |
| 28 | `blockType.room-local-needs-mapping` | `room_local_type_without_mapping` | ✗ GAP | 随档映射表属存档面，非本批次 |
| 29 | `blockType.player-picks-template-only` | `player_type_declares_behavior` | ✗ GAP | `assetLibraries.player` 无卡 |
| 30 | `palette.reclaim-before-escalation` | `palette_reclaim_before_escalation` | ✓ | R-00435 验收项 4 |
| 31 | `palette.no-dead-entry-in-payload` | `dead_palette_entry_in_payload` | ✓ | R-00435 验收项 5 |
| 32 | `payload.delta-needs-matching-base` | `delta_base_revision_mismatch` | ✓ | R-00436 验收项 1 |
| 33 | `payload.delta-not-for-first-delivery` | `delta_used_for_first_delivery` | ✓ | R-00436 验收项 2 |
| 34 | `query.unresolved-is-not-air` | `unresolved_hit_treated_as_air` | ✗ GAP | `physicsQuery` 整块无实现卡 |
| 35 | `query.unresolved-is-not-solid` | `unresolved_hit_treated_as_solid` | ✗ GAP | 同上 |
| 36 | `query.collision-comes-from-material-table` | `collision_behavior_not_from_material_table` | ✗ GAP | 同上 |
| 37 | `query.overflow-must-be-reported` | `query_buffer_overflow` | ✗ GAP | 仅 R-00439 详求 3 在 ABI 面提到 truncated 标志 |
| 38 | `query.deterministic-across-ends` | `query_result_divergence` | ✗ GAP | 同上 |
| 39 | `query.read-only` | `query_mutates_world` | ✗ GAP | R-00437 验收 5 只覆盖 blockRead 只读，不覆盖物理查询 |
| 40 | `axis.y-is-vertical-unsigned` | `world_y_out_of_range` | ✓ | R-00434 验收项 4 |
| 41 | `catalog.dense-allocation` | `block_catalog_not_dense` | ✓ | R-00434 验收项 1 |
| 42 | `catalog.name-is-stable-and-unique` | `block_catalog_name_reused` | ✓ | R-00434 验收项 2 |
| 43 | `catalog.row-must-be-complete` | `block_catalog_row_incomplete` | △ | R-00434 详求 5，无验收项；契约亦无 invalidCase |
| 44 | `read.budget-is-declared` | `read_budget_exceeded` | ✓ | R-00437 验收项 2 |
| 45 | `read.result-carries-revision` | `read_result_missing_revision` | ✓ | R-00437 验收项 3 |
| 46 | `read.missing-is-not-air` | `section_unavailable` | ✓ | R-00437 验收项 1 |
| 47 | `write.entry-is-structured` | `unstructured_mutation_entry` | ✓ | R-00438 验收项 2 |
| 48 | `write.expected-revision-required` | `stale_section_revision` | ✓ | R-00438 验收项 1 / 10 |
| 49 | `write.batch-is-all-or-nothing` | `write_batch_too_large` | ✓ | R-00438 验收项 3（契约本身错位，见三·R-00438） |
| 50 | `identity.cell-offset-formula` | `cell_offset_out_of_range` | ▲ | R-00434/435/436/438/439 评论要求，**五卡皆无验收项** |
| 51 | `residency.pin-guarantees-ready` | `pinned_read_returned_pending` | ✓ | R-00440 验收项 10 |
| 52 | `residency.pin-budget-is-hard` | `residency_pin_exceeds_budget` | ✓ | R-00440 验收项 2（无预算常量，见三·R-00440） |
| 53 | `residency.pinned-never-evicted` | `pinned_section_evicted` | ✓ | R-00440 验收项 3 |
| 54 | `residency.pin-ready-before-gameplay` | `pin_region_not_ready` | ✓ | R-00440 验收项 1 |
| 55 | `catalog.behavior-template-must-be-registered` | `unknown_behavior_template` | ▲ | R-00434 评论要求，无验收项 |
| 56 | `read.cell-carries-presence` | `cell_read_missing_presence` | ▲ | R-00437 评论**明文要求补**，无验收项 |

**统计**：✓ 23 条 / △ 3 条 / ▲ 3 条 / ⊘ 6 条 / ✗ GAP 21 条 = 56。

## 五、覆盖矩阵：51 个 errorCode

错误码归属由 `rules[].onViolation` 推出，覆盖状态随其规则。**51 个错误码全部被至少一条 rule 引用，且没有任何 rule 的 `onViolation` 落在 `errorCodes` 之外**（契约内部自洽）。

| 覆盖 | 错误码 |
|---|---|
| **✓ 有卡且有验收项**（23） | `chunk_carries_data`, `palette_overflow`, `section_encoding_mismatch`, `section_digest_mismatch`, `block_type_scope_violation`, `system_reserved_type_misuse`, `palette_reclaim_before_escalation`, `dead_palette_entry_in_payload`, `delta_base_revision_mismatch`, `delta_used_for_first_delivery`, `world_y_out_of_range`, `block_catalog_not_dense`, `block_catalog_name_reused`, `read_budget_exceeded`, `read_result_missing_revision`, `section_unavailable`, `unstructured_mutation_entry`, `stale_section_revision`, `write_batch_too_large`, `pinned_read_returned_pending`, `residency_pin_exceeds_budget`, `pinned_section_evicted`, `pin_region_not_ready` |
| **△ 弱覆盖**（3） | `material_class_not_a_cell_lane`, `unknown_material_class`, `block_catalog_row_incomplete` |
| **▲ 评论要求待补**（3） | `cell_offset_out_of_range`, `unknown_behavior_template`, `cell_read_missing_presence` |
| **⊘ 仅 R-00432 覆盖，关闭即孤儿**（5） | `unknown_section_key`, `unknown_chunk_key`, `section_y_out_of_range`, `coordinate_out_of_bounds`, `dirty_section_not_durable` |
| **✗ GAP 无卡覆盖**（17） | `lighting_in_payload`, `liquid_auto_propagation_unsupported`, `cross_material_face_merge`, `entity_binding_missing`, `entity_binding_orphan`, `entity_binding_type_mismatch`, `entity_binding_not_sparse`, `business_data_in_payload`, `binding_commit_split`, `room_local_type_without_mapping`, `player_type_declares_behavior`, `unresolved_hit_treated_as_air`, `unresolved_hit_treated_as_solid`, `query_buffer_overflow`, `query_result_divergence`, `collision_behavior_not_from_material_table`, `query_mutates_world` |

### 排期缺口按面归拢

| 面 | 未覆盖 | 备注 |
|---|---|---|
| **物理查询** `physicsQuery` | rules 34–39 / 6 码 | ADR-062 决策 #8 已把物理查询移进契约、并定「C 签名属 `native-abi.json`」，但**本批次没有任何实现卡**，R-00439 的函数表也未含物理查询 |
| **方块实体绑定** `blockEntityBinding` | rules 20–25 / 6 码 | 整块无卡；R-00433 的非目标清单里也没列它——既没排期也没显式排除 |
| **键与坐标合法性** | rules 1–5 / 4 码 | 仅 R-00432，随其关闭而孤儿 |
| **驻留与落盘回执** | rules 12–13 / 1 码 | R-00440 只覆盖 pin，不覆盖脏页栅栏与回执覆盖界 |
| **改动层派发** `diffDispatch` | rules 10–11 | 零字节短票与派发侧缺块语义无卡 |
| **材质 / 液体 / 房间局部映射 / 玩家模板** | rules 17–19, 28–29 | 部分属存档与网格面，非本批次 |
| **光照** | rule 14 | R-00433 已显式列为非目标，属**已知不排期**，非遗漏 |

### 按 `voxel.md` 功能模块归拢（设计文档 → 线上卡）

设计概要 §4 的抬头是「每块可以直接开需求单」，共 13 个模块；§5 TODO 阶段 0 列了 8 张卡。对照线上：

| 模块 | 线上卡 | 状态 |
|---|---|---|
| M1 世界分层与方块编码 | R-00434 | ✅ |
| M1a 官方方块目录 | R-00434（并入 I-1） | ✅ |
| M2 Section 存储三态 | R-00435 | ✅ |
| M3 光照 | — | ⊗ 已声明非目标 |
| M4 网格生成与零拷贝交付 | — | ⊗ 已声明非目标 |
| M5 房间改动层与同步 | R-00436 仅覆盖 `Delta` 编解码 | ⚠ **部分**：派发面本身无卡 |
| M6 权威写入与事务 | R-00438 | ✅ |
| **M6a 方块与实体的绑定** | **无** | ❌ **缺口** |
| **M7 物理检测** | **无** | ❌ **缺口** |
| M7a 玩法侧批量读 | R-00437 | ✅ |
| M8 流式加载与驻留 | R-00440 仅覆盖 ③a 区域 pin | ⚠ **部分**：驻留与落盘回执无卡 |
| M9 存档与恢复 | — | ⊗ 已声明非目标 |
| M10 存档可读性与离线检查 | — | ⊗ 已声明非目标 |

R-00433 的非目标清单是「光照传播、网格生成、流式加载、存档格式、检查器 CLI」——**M6a 与 M7 既不在任何卡里，也不在这份非目标清单里**，属真缺口而非有意不排期。二者恰好对应第四节的 rules 20–25 与 34–39（12 条规则 / 12 个错误码）。

§5 TODO **阶段 0「先立规矩」8 张卡**的落单情况：

| 设计卡 | 线上 | 说明 |
|---|---|---|
| 0-1 数值 profile 落成生成配置 | R-00434 近似覆盖 | 以「契约副本 + SHA-256 漂移测试 + 8 个常量断言」实现；断言清单未含 `lightBitsPerCell` |
| 0-2 BlockType 段表与两张稠密配表 | R-00434 | ✅ |
| 0-2b 官方方块目录 | R-00434 | ✅ |
| 0-3 BlockState 动态位段框架 | R-00434 | ✅ |
| 0-3b 材质类表 | R-00434 | ✅ |
| 0-4 Section 三态存储 | R-00435 | ✅ |
| 0-5 存档字节自描述 | **无** | 归 M10，落在非目标（存档格式 / 检查器 CLI）内 |
| **0-6 Section 稀疏引用表与绑定不变量** | **无** | 归 M6a，**不在非目标内** → 缺口 |

阶段 1 垂直切片 6 步中，第 5 步的「DDA 三种检测」（M7）与第 6 步「带业务数据的方块跑通」（M6a）同样无卡；第 4 步「房间改动层：按 Chunk 打包存档、按 Section 派发、零字节短票、缺块挂起」只落了 `Delta` 一角。阶段 2（检查器 CLI、烘焙管线、资产导入、监控）与阶段 3（驻留预算实测、调度优先级、视距）整体无卡，与非目标清单及「预上线」定位一致。

## 六、契约自身待裁决的缺陷（本次顺带发现，未改动）

1. **rule 49 错位**：`write.batch-is-all-or-nothing` 文本讲「部分应用」，`onViolation` 挂 `write_batch_too_large`（尺寸错误）；契约无「批被部分应用」的错误码，而 `write_batch_too_large` 的语义出处 `blockWrite.batch.onExceeded` 没有对应 rule。
2. **全量编码携带 `baseSectionRevision` 无码**：只有 `sectionPayload.envelope.conditional` 的散文，无 rule、无 errorCode。R-00436 验收 #4 因此无码可引。
3. **pin 预算无常量**：`residency_pin_exceeds_budget` 依赖的「驻留预算」在 `limits` 与 `pinnedRegions` 里都没有字段，无机器可校验的界。
4. **3 个错误码无 invalidCase 覆盖**：`block_catalog_row_incomplete`、`pin_region_not_ready`、`write_batch_too_large`。
5. **`verify-wire.mjs` 对本契约不做语义执行**：57 个 invalidCase 全部 `validatorCheck: false`，且该脚本无 voxel 专属校验分支——「全绿」只等于声明完整性通过，不等于语义验收。ADR-062 第 70 行的「全绿」措辞易被读成后者。

> 另：ADR-062 与 `voxel.md` 亦滞后于契约（ADR 状态行仍写 44/49/98、决策段完全没有 cellOffset 与 pin 两条；`voxel.md` 第 101 / 446 行把楼梯 / 半砖 / 门列为行为模板，与 `behaviorTemplates.v1IsExhaustive` 的「v1 只有 FullCube 与 Liquid，没有『等等』」冲突，且黑话表内两条同名词条口径相反）。本次未改，按「契约与文档冲突时以契约为准」不影响实现方判断，但建议随下一次契约合入一并修订。

## 七、待 Owner 裁决事项

| # | 事项 | 优先级 | 建议 |
|---|---|---|---|
| 1 | **PR #81（`f7f8a0d`）合入 `main`** | **P0** | 先合入。否则七条评论与 R-00440 的前置声明全部悬空，worker 拉到的是旧契约 |
| 2 | R-00437 补 `cell_read_missing_presence` 验收项 | **P1** | 评论已明文要求，直接补 |
| 3 | R-00434 补 `cell_offset_out_of_range` 与 `unknown_behavior_template` 两条验收项 | **P1** | 评论已明文要求覆盖 |
| 4 | R-00439 补三条验收项（cellOffset 进 ABI doc / 单格 presence / **pin slot**），并改正文函数表与标题 | **P1** | pin slot 是跨仓冻结面，漏了要再走一轮生成合入 |
| 5 | R-00432 关闭前先安排 rules 1–5 与 12 的承接卡 | **P1** | 直接关闭会产生 6 条孤儿规则 / 5 个孤儿错误码 |
| 6 | R-00433 更正契约计数、补四行决策账本、补 I-6 进轨道 A、明确 A-2 / A-3 是否落卡 | **P1** | 它是来源真值卡，失真会向下游传播 |
| 7 | R-00440 更正前置产物表述（或以事项 1 解决） | **P1** | 与事项 1 联动 |
| 8 | **物理查询 M7（rules 34–39）与方块实体绑定 M6a（rules 20–25）是否排期** | **P1** | 12 条规则 / 12 个错误码完全无卡；`voxel.md` 把两者列为可直接开单的模块，而 R-00433 非目标清单未排除它们——需明确「排期」还是「显式排除」 |
| 8b | **M5 改动层派发面**（按 Section 派发 / 零字节短票 / 缺块挂起 / 按 Chunk 打包）是否排期 | **P1** | 只落了 `Delta` 一角；rules 10–11 无卡 |
| 8c | **M8 驻留与落盘回执**（脏页栅栏 / 回执覆盖界）是否排期 | **P1** | R-00440 只覆盖 pin；非目标只写「流式加载」，未排除「驻留」 |
| 9 | 契约四处缺陷（第六节 1–4）是否修订 | P2 | 修订须走「先改架构仓契约并递增 ADR」的既定顺序 |
| 10 | R-00435 / R-00436 / R-00438 是否补 cellOffset 相关验收项 | P2 | 三卡评论均声明「原验收不变」，属可接受的判断，补则更严 |
| 11 | R-00439 验收 #6 的「7 份契约」改为不写死份数 | P2 | 当前属实，第 8 份契约落地时会假失败 |
| 12 | ADR-062 与 `voxel.md` 随契约同步修订 | P2 | 见第六节末 |

## 八、验证

```
$ node .spec/tools/spec-lint.mjs
$ node eng/verify-wire.mjs
```

输出见交回物。本次为纯读复核，除本文件外未改动仓内任何文件，未对 Workflow 执行任何写操作。

## 九、2026-09-05 Owner 裁决增量（R-00434）

本节只记录已授权的 R-00434 变更，不回写前文当日复核事实：

- **D1 已定**：`entity_occupancy_placeholder` 改为 `BlockType=2`（ECS occupancy）；`BlockType=3` 保留为 structure placeholder。
- **D2 已定**：`0..3` 使用 typed built-in sentinel；`4..255` reserved / non-resolvable；普通材质与行为模板解析仅适用于已登记官方目录行或已映射房间局部行；其他 admitted type 返回新增 `unregistered_block_type`。既有 `room_local_type_without_mapping` 保留在存档映射完整性上下文。
- **D3 已定**：目录行结构校验优先。任一必需字段缺失 / null / 空值返回 `block_catalog_row_incomplete`；结构完整且非空未知 `materialClass` 才返回 `unknown_material_class`。

契约已增加 resolver / catalog-validation machine vectors，`verify-wire` 对其执行专属断言；ADR-062、`voxel.md` 与 ABI 生成源摘要同步到 52 error codes / 57 rules / 110 顶层场景。原第六节第 5 项（「verify-wire 不做体素语义执行」）已由该专属分支关闭；其余未授权缺陷与排期事项不因本增量自动改变。
