---
name: 2026-09-04-bomber-voxel-asks-reply
description: 对 LumioGame 炸弹人 A8/A9 体素需求清单的逐条答复——能/不能/什么时候、本仓待补面、对方需修订的判断；排 Stage 2 或改 ITerrainStore 前查
metadata:
  type: doc
  status: 已交付
---

# 炸弹人体素需求清单（A8 / A9）答复

> **问方**：`LumioGame`（炸弹人 Stage 0a 已把地形移出 ECS，收敛到 `ITerrainStore` 之后，见其仓 ADR 0016）。
> **答方**：`LumioGameEngine`（体素公共契约与裁决真值）。
> **依据**：[`engine/wire/voxel-world-v1.json`](../../engine/wire/voxel-world-v1.json)（`lumio.voxel-world.v1`）与 [`ADR-062`](../decisions/ADR-062-voxel-world-public-contract.md)；设计说明见 [`knowledge/features/voxel.md`](../knowledge/features/voxel.md)。
> **核验方式**：逐条对着 `LumioVoxelEngine` / `LumioGameRuntime` 的**源码**核，不是对着文档核。命中与未命中都在下文标出处。

---

## 0. 三条改变问题形状的更正

### 更正 1 · 那六个模块不是方块存储

对方判断：「Rust 侧已完成的领域实现在 C# 这边一行也调不到，解除 C ABI 即可用。」

**调不到属实，原因不是 ABI。** `LumioVoxelEngine` 今天没有方块：

| 核验项 | 结果 |
|---|---|
| `crates/*/src` 下 `BlockId` / `block_id` | **0 命中**（只在契约一致性测试里出现） |
| `lumio-voxel-domain/src/section/` | 只有 `directory / slot / payload / delta / dirty / replacement`，无格数据 |
| `section/payload.rs` | 一个 Section = **不透明字节 + SHA-256**（"Pages are sealed; no Storage pointers"） |
| `lumio-voxel-ops/src/query/execute.rs` | directory presence 查询，注释 `No payload leak`，返回 `{section_id, presence, schema_id}`——**不返回方块值** |
| `palette` / 三态编码 | src 下 0 命中 |

已完成的是**跨域事务那根脊柱**（prepare/commit 幂等回执、Section revision、脏页与耐久回执栅栏、canonical manifest、snapshot capture/restore、参考实现差分）。**「方块存在哪、怎么取一格」尚未开工。** 所以 C ABI 不是 Stage 2 的前置，块存储才是。

### 更正 2 · 16³ 叫 Section，竖直轴是 y 且无符号

| | 对方以为的 | 契约 |
|---|---|---|
| 16³ 数据单元 | Chunk | **Section**，键 `s:<x>:<y>:<z>` |
| Chunk | 就是那个立方体 | **16 个 Section 竖摞（16×256×16），不存数据、无独立 revision**，键 `c:<x>:<z>` |
| 竖直轴 | `z`（`z=-1` 地面 / `z=0` 砖层） | **`y`，限 0~15 层 / 0~255 格，无负数** |
| 水平轴 | x, y | **x, z**，负坐标一等公民 |

轴映射：`游戏(X, Y, Z) → 引擎(x=X, z=Y, y=Z+1)`，地面层落 `y=0`、砖层落 `y=1`。

### 更正 3 · `MaterialId` 不再是不透明 uint16

对方边界依据引的是 `mvp-placevoxel-content-spec.md` §6.2「Voxel 侧按不透明 uint16 存取」。**该条已被 ADR-062 取代**：现在是 32 位 `BlockId = BlockType << 8 | BlockState`，无符号，且**引擎要解释它**（调色板、材质类、网格合面、透光衰减都依赖它）。

`ITerrainStore.GetBlock -> MaterialId` 现在就改 `uint32`；晚改的代价是全调用点改类型，正是「只换实现不改调用方」要避免的事。

---

## 1. 逐条答复

### ① 方块目录的边界：对，但对方把引擎那半说小了

目录归引擎、玩法行为绑定归 `LumioGame`——成立，对方不必自建。

引擎拥有的不止「外观 + 怎么存」，还有**材质类**：BlockType 的一列配表属性，一张表同时定**网格 / 渲染通道 / 碰撞 / 透光**四轴，玩法层与玩家都不能定义。

| 炸弹人方块 | 段 | 材质类 |
|---|---|---|
| Air | **0**（系统哨兵，已占） | — |
| 铁皮 / 积木 / 木箱 / 木头 / 鞭炮 / 地面 / 冰 | 256 起官方段（8 个新号） | **Solid** |
| 水 | 同上 | **Liquid**（v1 静态液体，放置即固定液面，不做流动模拟） |

⚠️ 引擎的 Liquid 是**不挡路 + 透光衰减 + 半透明通道**。炸弹人的「阻断爆炸 + 禁止放弹 + 溺水」是玩法、不冲突，但**别指望引擎碰撞面会挡人**。

### ② 三个方法：两个能给，第三个不给逐格版本

| 要的 | 答复 |
|---|---|
| `ChunkRevision(chunkId) -> u64` | **能，改名 `SectionRevision(sectionKey)`。** u64 单调、per-Section，已在跑。**Chunk 没有 revision**，是契约红线 |
| `ApplyBatch(mutations, expectedRevision)` | **能。** prepare/commit 两段 + expectedWorldRevision + 按 TxnId 幂等回执，代码已存在；但**请求形状尚未进契约**（见 §2 C-3） |
| `GetBlock(x, y, z)` | **不给逐格版本。** 语言边界纪律：跨 FFI 只传「小的结构化命令」与「大的只读缓冲指针」；48 次逐格过界正是这条规矩要拦的 |
| 批量读 | **`physicsQuery.overlap` 已覆盖大半**——盒内相交格坐标 + BlockId 写进调用方缓冲，体素侧不分配、不返回需释放的句柄，装不下报 `truncated` 与实际总数。缺的是「取矩形内**全部**格（含空气）」这一读法的语义（见 §2 C-1） |

**逐格开销量级：给不出数字，没测过。** 出处：`LumioVoxelEngine/docs/evidence/decision-gates/VOX-D-003-query-budget.md` —— 吞吐、尾延迟、预算精度 "**not measured**… There is no production query planner… No imaginary latency/throughput values"。不编。

结构上也不需要该数字：**炸弹人整张图 61×61 只落在 4×4 = 16 个 Section**（起点对齐不巧最坏 5×5 = 25），两层都在 `y=0` 那一层 Section 内。一次批量读拿全图 = **16 条载荷 ≈ 64 KB**。48 次逐格根本不该过边界。

### ③ 1200 格 / 单 tick：能接受，且差着数量级——**不要做分帧提交**

| | 数字 |
|---|---|
| 整张图占几个 Section | **16** |
| 一条 Palette 载荷 | 4096 格 × 1 字节索引 ≈ **4.3 KB** |
| **全图方块数据总量** | **≈ 64 KB** |
| 1200 格连锁最坏铺满 | **16 个 Section = 整张图** |
| 一次最坏提交的代价 | **重写 64 KB** |

1200 格**不是 1200 次操作**，是「重发布 ≤16 条载荷」。稳态 100–300 格/秒、出生点 33 次/秒同理不构成压力。**分帧提交引入的复杂度（跨 tick 半提交状态、回放对账跟着分帧走）远大于它省的东西。**

**真正要盯的是派发，而契约现在有 `Delta`**（每条 6 字节 = 格内偏移 + 32 位 BlockId，必须带 `baseSectionRevision`）：

- 1200 格连锁 ≈ **7.2 KB / 客户端 / tick**（不是全量的 64 KB）
- 炸弹人的几何决定 **Delta 永远比全量便宜**：单 Section 最多 2 层 × 256 格 = 512 格 × 6 B = 3 KB < 全量 4.3 KB

**结论：不用下调火力上限，不用改连锁规则，不要做分帧提交。**

### ④ 逐字节确定性：编码器满足；但对齐目标错了

**满足的部分**——canonical 编码器有序（`BTreeMap`）、typed value、重复成员直接拒绝、`#![forbid(unsafe_code)]`；期望摘要由 `tools/canonical/canonical_encoding_oracle.py` **从书面规则独立实现**产出，不是从 Rust 输出里抄的。无哈希表遍历顺序问题。

**三个坑：**

1. **ADR-035 是 `Historical`**，其旧制度架构源与校验链已删除，那份 schema 永远不会再生成。
2. **它本周已断代**：Section 改名把 snapshot manifest 摘要从 `b513120c…` 打到 `1893afc9…`。
3. **今天不存在「地形 canonical 字节」**——有的是不透明载荷字节的 SHA-256 + 确定性 manifest 排序（因更正 1）。

**建议：把 StateHash 的地形那一半定义在 `LumioGame` 自己对 `ITerrainStore` 的 canonical 投影上**（61×61×2 逐格按固定顺序 emit `uint32` BlockId），别绑引擎存储格式。这样 Stage 2 换实现哈希天然连续、历史回放基线不作废——因为哈希基准从来不在引擎这边。引擎能承诺的是 **32 位 BlockId 的位段语义已进契约、机器可校验**。

### ⑤ 扁世界：没问题，不为其做任何事；VOX-D-001 已定

| | |
|---|---|
| 浪费 | 14 层空气也占索引位 = 64 KB 的 87.5% ≈ **56 KB** |
| 参照 | 浏览器客户端视距 6 的正常世界体素数据 ≈ **3 MB** |

为省 56 KB 引入「扁世界特化布局」= 多一条代码路径换一个测不出来的收益。**不做。**

**VOX-D-001 已决，编号随旧制度废止**：真值在契约 `limits` 段——Section 16³ = 4096 格、每 Chunk 16 层、世界高 256、调色板 256 项 / 索引 8 位。对方按 16³ 排设计是安全的，其 ADR 0016「不锁 chunk 尺寸」一条可以撤销。

### ⑥ 负面清单：四条里对三条半，漏了两个

| 模块 | 对方判断 | 答复 |
|---|---|---|
| `streaming` | 不需要 | **确认**，16 个 Section 全驻留 |
| `migration` | 不需要 | **确认**，每局独立地图无跨局存档 |
| `spatial` | 不需要 | **确认**，密铺规则体的空间索引就是坐标本身，引擎对谁都不建树 |
| `mesh-collision` | 不需要 | **对一半**。碰撞对；mesh 是「地形怎么变成三角形」，**若要引擎画这张地形，网格生成即在关键路径上**——待对方确认 |

**漏掉的两个，都在关键路径最前面**：① **方块目录本身**（BlockType 段表 + 两张稠密配表 + 材质类表）；② **Section 三态存储**（「方块到底存在哪」，`GetBlock` 的下一层）。

对方列的六个是**旧模块图**的名字。按新模块图，其关键路径是 **M1（分层与方块编码）+ M2（三态存储）+ M6（事务）+ 批量读**。

### ⑦ C ABI：实测对，结论错——不是缺口，是设计

零 `#[no_mangle]`、零 `crate-type = ["cdylib"]` 属实，**但这是架构定的**：

> NativeCore 和 VoxelEngine 不导出自己的根符号；SDK 聚合层负责把它们组合成一个 Native 库。
> —— [`knowledge/features/architecture.md`](../knowledge/features/architecture.md)

**导出点在本仓且已在跑**：单一根符号 `lumio_engine_get_api_v1`，现有 ping / CLR host / timer；**体素 slot 数 0**。加 slot 是本仓的卡，不是 Voxel 仓的。

**Runtime 侧第二条阻塞核实无误**：`LumioGameRuntime/modules/coordination/src/Lumio.GameRuntime.Coordination/Prepare/TxnPrepareCoordinator.cs:73` 是 `internal interface IVoxelWorldPort`，`VoxelAdapterSurfaceTests` 的反射断言在。该条归 `LumioGameRuntime` 路线图，本仓不代其立项。**注意它挡的不是 `GetBlock`（那个还不存在），挡的是 `Prepare / Commit / Abort / Query / ReadRevision` 五个跨域事务方法。**

**时间点：不给日期。** 体素卡（阶段 0 六张 + 阶段 1 垂直切片）已写好但**一张未派**；当前派活队列是 RM-00011 R5 收敛与 LumioPlatform MS-1。进队列即通知，在那之前对方按 Stage 0a 自有内存后端排期。

---

## 2. 本仓待补面（依赖顺序）

```
I-1 目录 → I-2 三态存储 → C-1/I-4 读法 → A-1/A-2 ABI slot → C# 面
                                              ↑
              Runtime 侧 IVoxelWorldPort 公开面在此汇合（不归本仓）
```

**契约面**（`engine/wire/voxel-world-v1.json`）

| # | 待补 | 挡炸弹人 |
|---|---|---|
| C-1 | 玩法侧「取矩形内全部格（含空气）的 BlockId」读法语义——`physicsQuery.overlap` 的过滤器是材质类掩码，空气不是材质类 | **挡** |
| C-2 | 官方方块目录表（`BlockType → {名称, 材质类, 素材引用}`）与铸号规程——今天只有段表**结构**，没有表 | **挡** |
| C-3 | 写入请求形状（`sectionKey` + 格内偏移 + 新 BlockId + expectedRevision）——今天只在 Rust 内部以字符串 map 形态存在 | **挡** |
| C-4 | 一条「y 是竖直轴、块坐标 0–255 无符号」的语义规则与 invalidCase——今天无机器校验会挡下「z 竖直」的误解 | 不挡，防下一个 |

**实现面**（`LumioVoxelEngine`，今天为零）：I-1 段表与两张稠密配表与材质类表与 BlockState 动态位段（卡 0-1/0-2/0-3/0-3b）；I-2 Section 三态存储与调色板槽压缩（卡 0-4）；I-3 Delta 编解码与 `baseSectionRevision` 校验；I-4 读法实现；I-5 mutation 落成真正的逐格写。

**ABI / SDK 面**（本仓）：A-1 `engine/abi/native-abi.json` 加体素 slot；A-2 `engine/native/modules/` 把 VoxelEngine 组进聚合根；A-3 托管入口。

---

## 3. `LumioGame` 需修订的判断

### A. 必须改，不改会做出错的东西

| # | 之前写下的 | 现在的事实 | 落点 |
|---|---|---|---|
| 1 | Voxel 已完成实现「一行也调不到」，解除 C ABI 即可 | 调不到属实，但没有块存储；**C ABI 不是前置，块存储才是** | A9 ⑦、A8 实测补充 |
| 2 | `MaterialId` 是不透明 uint16 | 32 位 `BlockId`，无符号，引擎要解释它 | ADR 0016 边界依据；`ITerrainStore` 签名改 `uint32` |
| 3 | `z = -1` 地面 / `z = 0` 砖层，z 是竖直轴 | 竖直轴是 **y、无符号 0–255**，负竖直坐标在键语法上不存在 | ADR 0016、`design.md` §5、G-1 / G-4 卡 |
| 4 | 16³ 叫 chunk → `ChunkRevision(chunkId)` | 叫 **Section**；Chunk 无数据无 revision | ADR 0016 三方法、G-0 契约 |
| 5 | 编码对齐 ADR-035 | ADR-035 是 `Historical`，本周已断代，且地形 canonical 字节今天不存在；改为自己的 canonical 投影 | ADR 0016、A9 ④、**G-6 卡** |

> 第 2、3、5 条现在改成本≈0；晚改代价分别是「全调用点改类型」「全量转档」「历史回放基线作废」。

### B. 判断没错，依据或数字要更新

| # | 之前写下的 | 更新为 |
|---|---|---|
| 6 | 方块目录 = 外观 + 怎么存 | 还包括**材质类**（四轴）；Liquid **不挡路** |
| 7 | chunk 尺寸未决（VOX-D-001），本 ADR 不锁 | **已决且冻结**，「不锁」一条可撤销 |
| 8 | 关键路径只有那六个模块 | 那是旧模块图；实为 **M1 + M2 + M6 + 读法**，漏了目录与存储 |
| 9 | 爆炸传播逐格读 48 次 | 不该过边界；一次批量读拿全图 16 条载荷 |
| 10 | 1200 格是尖峰，需引擎给阈值，超了就下调火力 | 存储层非问题（全图 64 KB）；派发有 Delta ≈ 7.2 KB/客户端/tick。**不用下调，不要分帧** |
| 11 | 分帧提交与每 tick 预算归 Voxel 侧决定 | 引擎给不出阈值（零实测）；按第 10 条该决定不需要等引擎 |

### C. 需对方确认

| # | |
|---|---|
| 12 | 「mesh-collision 不需要」——若要引擎画地形，网格生成即在关键路径上。**本文按「`LumioGame` 自己画」记** |
| 13 | 九种方块进**官方全局段**（跨房间恒定、玩法可写常量）还是房间局部段？**本文按官方全局段记**——起床战争与 duckoff 大概率复用同一批 |
