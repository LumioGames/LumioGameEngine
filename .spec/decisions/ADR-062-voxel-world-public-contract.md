# ADR-062：体素公共语义改从 `lumio.voxel-world.v1` 取——16³ 数据单元改名 Section，旧制度体素契约作废

状态：Draft（2026-09-04，契约随 `4d05e8c` 落地并在本轮扩至 44 错误码 / 49 规则 / 98 用例；随 M1/M2 实现验证后转 Accepted）
取代：作废 [ADR-024](ADR-024-voxel-p0-contract-set.md)、[ADR-035](ADR-035-voxel-snapshot-payload.md)、[ADR-036](ADR-036-voxel-streaming-durability-ack.md) 中「Chunk 是 16³ 数据单元」的分层语义与其 `schemas/` 依赖；三者的事务、Pin 栅栏、耐久回执与 canonical 排序等**技术结论**仍可被引用，但其 Schema/Fixture 链已随旧制度删除，不再是可校验真值
Owner：`LumioGameEngine`（契约与裁决真值）、`LumioVoxelEngine`（唯一实现）、`LumioGameRuntime` / `LumioGame` / `LumioClient`（消费方）

## 治理原则

- 沿用 ADR-056：**第一性原理——如无必要，勿增实体。** 世界里只有 Section 一个数据单元；每格 dense 数据只有方块一路。
- 沿用 ADR-060：**彻底清理，不留兼容。** 旧的 `page` 概念与 `BlockType` 草案魔数整体删除，不留别名、不留过渡段。

## 背景

体素是最后一块没有公共契约的核心系统。它此前的真值分散在三处，且三处都已失效：

1. **ADR-024 / 035 / 036 依赖的 `schemas/` / `fixtures/` / `ids/` 校验链，已随旧「架构源 + Baseline 门禁 + 八仓镜像」制度一并删除**（见本目录 README 开头）。生成源仓不存在，那批 schema 永远不会再生成。
2. **`LumioVoxelEngine` 把 16×16×16 = 4096 格的数据单元叫 `chunk`**，命名来自上述已死镜像。
3. **`mvp-placevoxel-content-spec.md` §6.2 声称「`MaterialId` 是消费方拥有的目录，Voxel 侧按不透明 uint16 存取」**——与「引擎要解释方块以做调色板、材质类、网格与透光」直接冲突。

后果是消费方按三份互不相容的真值排设计。`LumioGame` 炸弹人 Stage 0a 的 `ITerrainStore`（其仓 ADR 0016）即按第 2、3 条写下：方法名 `ChunkRevision(chunkId)`、`MaterialId` 为不透明 uint16、竖直轴取 `z` 且允许 `z = -1`、快照编码对齐 ADR-035。四条今天全部不成立。

## 决策

- **体素公共语义的唯一真值是 [`engine/wire/voxel-world-v1.json`](../../engine/wire/voxel-world-v1.json)（`lumio.voxel-world.v1`）**，校验入口 `node eng/verify-wire.mjs`。设计说明在 [`knowledge/features/voxel.md`](../knowledge/features/voxel.md)，冲突时以契约为准。消费方不得在实现仓另写一份真值。
- **三层分层与命名冻结。** `Block` → `Section`（16×16×16 = 4096 格，**数据载体、最小同步单位、驻留单位、版本锚点**）→ `Chunk`（竖摞 16 个 Section = 16×256×16，**不携带数据、不持有独立 revision**，是存档打包与按列计算的容器）→ `World`。**任何消费方不得用 Chunk 指代 16³ 数据单元。**
- **规范键以元数防呆。** Section `s:<x>:<y>:<z>`（y 限 0~15），Chunk `c:<x>:<z>`。三坐标的 `c:` 键在语法上即非法，必须显式拒绝，且不得被解读为 `c:x:z` 或 `s:x:y:z`。
- **`BlockId` 是 32 位且引擎解释它**：`BlockType << 8 | BlockState`，高 24 位种类、低 8 位摆法（动态位段）。**一律按无符号处理**——房间局部段的作用域位落在 bit 23，即 `BlockId` 的最高位，用有符号 `int32` 承载会变成负数。**这条取代 `mvp-placevoxel-content-spec.md` §6.2 的「不透明 uint16」。**
- **段表只靠一个位加一个 256 分界**：作用域位（bit 23）= 0 是全局官方段（`0` 空气 / `1` 错误块 / `2` 被 ECS 实体占用 / `3` 结构占位 / `4–255` 系统预留 / `256+` 官方素材库连号稠密）；= 1 是房间局部段（玩家素材库，局部号 `BlockType & 0x7FFFFF`，映射表随存档走）。旧草案的 `9999 / 10000 / 2000000` 三个魔数作废。
- **材质类是 BlockType 的配表属性，归引擎。** v1 只有 `Solid` / `Liquid` 两类，由**同一张表**声明网格 / 渲染通道 / 碰撞 / 透光四轴；不得在网格器、渲染器、物理、光照里各写一份分支，不得编码进 ID 分段，不得成为第三路逐格数据。新增材质类的唯一判据：这个差异能否只靠贴图表达——能，就不是新类。
- **载荷四编码，一个信封一个分发点。** `Uniform` / `Palette`（8 位索引，≤256 项）/ `Raw` 是全量编码，`Delta`（每条 6 字节 = 格内偏移 + 新 BlockId）是增量编码。**Delta 必须携带 `baseSectionRevision`**，对不上即 `delta_base_revision_mismatch`，拒收并请求一次全量重发，不许静默打补丁；首次送达与重同步禁用 Delta。
- **物理查询移进本契约**（原在 `scope.excludes`）：射线 / 重叠 / 扫掠三种，命中最小单位是 Block，结果三态 `Hit / Miss / Unresolved`。**`Unresolved` 既不等于 Miss 也不等于 Hit，且是正常结局不是错误码。** 阻挡与否只能查材质类表。语义在本契约，**C 函数签名属 [`engine/abi/native-abi.json`](../../engine/abi/native-abi.json)**。
- **光照是派生数据，永不入载荷**、不落盘、不上网（不可变的原始地图预烘焙除外）。
- **方块与实体的绑定只留一条稀疏引用**（`格内偏移 → NetEntityId`），业务数据挂 ECS 实体；体素侧不得自带第二套稀疏业务存储，业务字段不得随体素派发。
- **y 是竖直轴，世界 y 无符号 0~255**（`sectionY = worldY >> 4`、`cellY = worldY & 15`）；x 与 z 是水平轴，各自 signed 32 位，负坐标一等公民。**不接受 y-up / z-up 可配置项**——可配置意味着两套理解并存，而按「z 竖直」写下的代码能照过全部正则，错误要漂到渲染和物理才暴露。
- **官方方块目录是全局段的唯一分配来源**：每行六字段（`blockType` / `name` / `materialClass` / `behaviorTemplate` / `assetRef` / `stateLayout`），从 256 起**连号稠密**分配不留空洞（配表按编号直接下标，空洞会逼出哈希表）；编号与 `name` **永不回收、永不改写、永不重排**；实现仓不得自行铸号。
- **玩法侧批量读是独立于派发的一条路**：三种请求（单格 / 矩形 / 列），**结果必带 `sectionRevision`**，缺块四态与派发面共用一套且**不得把 Pending/Unavailable 填成空气**，预算是声明出来的数字（单次 262144 格 = 64 个 Section）且**超限整条拒绝、不静默截断**，结果写进调用方缓冲。它不是订阅——持续观察一片区域走改动层派发。
- **写入条目是结构化字段，不是字符串 map**：`sectionKey` + 格内偏移（0~4095）+ 新 `BlockId` + **`expectedSectionRevision`**；一批要么全生效要么全不生效（上限 65536 条），按事务 ID 幂等。字符串键值 map 让字段名、类型与边界全都不可机器校验，即 `unstructured_mutation_entry`。
- **尺寸与坐标语义一经冻结即不可变更**，改动等于全量转档，没有例外。

## 本 ADR 明确不冻结的（消费方按现状排期，不要当作已定）

| 面 | 现状 | 归属 |
|---|---|---|
| ABI 面的体素 slot | `engine/abi/native-abi.json` 体素 slot 数为 0；聚合根 `lumio_engine_get_api_v1` 现有 ping / CLR host / timer | 本仓，待开卡 |
| Native 聚合与托管入口 | `engine/native/modules/` 未组入 VoxelEngine；`engine/managed/Lumio.Engine.SDK` 托管入口未开 | 本仓，待开卡 |
| 实现面 | `LumioVoxelEngine` 的段表 / 稠密配表 / 三态存储 / Delta 编解码 / 批量读 / 逐格写全部为零 | 实现仓，待开卡 |

> 原表中的**写入请求形状**、**官方方块目录与铸号规程**、**竖直轴语义**、**玩法侧矩形批量读**四项已于同日补进契约并冻结，见上方「决策」。

## 替代方案

- **继续用 ADR-024/035/036 作真值**：被否。它们的校验链已删除，`chunkOrder` / `voxel-chunk-page` 等 id 指向不可再生成的产物；继续引用会让消费方以为有机器校验兜底。
- **保留 `chunk` 指代 16³、另给列一个新名字**：被否。存量代码与文档里 `chunk` 的两种含义无法用命名区分，而键的元数（三坐标 vs 两坐标）可以——防呆放在语法层比放在文档层可靠。
- **`MaterialId` 维持不透明 uint16、目录归消费方**：被否。调色板、材质类解析、网格合面、透光衰减都要求引擎理解方块；不透明就意味着这四件事全部下沉到消费方，等于每个产品各造一套体素。
- **为玩家方块铸全局唯一编号**：被否。集中铸号且永不回收，百万玩家各造二十块即两千万个号，24 位装不下。

## 失败语义

契约现有 44 个稳定错误码（`unknown_section_key` / `unknown_chunk_key` / `delta_base_revision_mismatch` / `delta_used_for_first_delivery` / `lighting_in_payload` / `dirty_section_not_durable` / `block_type_scope_violation` / `room_local_type_without_mapping` / `player_type_declares_behavior` / `palette_reclaim_before_escalation` / `dead_palette_entry_in_payload` / `business_data_in_payload` / `binding_commit_split` 等）。三条红线级失败：**把 `Pending`/`Unavailable`/`Unresolved` 物化成空气**、**携带光照或业务字段入载荷**、**未经回执覆盖卸载脏 Section**。

## 兼容影响与迁移

- 开发态契约，无部署中的体素消费方，不需要迁移窗口。
- `LumioVoxelEngine` 的 `refactor/section-chunk-rename` 分支**已停止推进**（Owner 2026-09-04 裁决）；体素公共语义与其落地由本仓全权负责，不再产出跨仓交接说明。两个 golden 断代（snapshot manifest 摘要、差分 trace）是「尺寸与坐标语义变更等于全量转档」的直接落地，不是测试维护。
- `LumioGame` 炸弹人 `ITerrainStore` 的四条判断需修订，逐条落点见 [`reviews/2026-09-04-bomber-voxel-asks-reply.md`](../reviews/2026-09-04-bomber-voxel-asks-reply.md)。
- 后续改本契约的唯一顺序：本仓改 `engine/wire/voxel-world-v1.json` → 实现仓复制到其 `wire/` → 更新常量与摘要 → 一致性测试变绿。

## 验证

`node eng/verify-wire.mjs` 覆盖 7 份契约；`voxel-world-v1.json` 现有 98 条声明用例（47 testCases + 51 invalidCases）全绿。`node .spec/tools/spec-lint.mjs` 校验本 ADR 的登记与链接可达。实现侧的一致性由 `LumioVoxelEngine` 解析同一份 JSON 逐条断言常量，并校验其 SHA-256 与仓内 `CONTRACT_SHA256` 相符。
