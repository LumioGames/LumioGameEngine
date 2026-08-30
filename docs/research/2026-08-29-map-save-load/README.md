# 体素大世界存档调研交付包

本包是围绕“怎么读、怎么写、怎么懒加载、怎么跨版本演进，以及怎么兼容 Minecraft 地图数据”的纯外部技术调研。目标画像为 Rust Authority + 浏览器 .NET WASM Replica，结论用于补齐已冻结公共契约之上的工程链路。

## 建议阅读顺序

1. 先读本README执行摘要。
2. 决策者直接读主报告 S 章，再回看 B/C/E/F/L/M 的证据。
3. Minecraft parser/importer 实现者读 B→C→K→L→P。
4. 存储/运行时实现者读 A→D→E→F→G→H→I。
5. 浏览器/内容分发实现者读 F→I→M→S。

## 包内文件

- `report/map-save-load-research-2026-08-29.md`：A–S 主报告。
- `sources.md`：逐条来源、定位、访问状态与支撑章节。
- `appendix/`：时序图源码、CSV对照表、benchmark计划、术语表与验证结果。

## 章节索引

- **A** — 谱系与总体形态
- **B** — Minecraft存档格式深挖
- **C** — Minecraft导入/转换管线
- **D** — 物理布局与容器
- **E** — 读档路径与性能
- **F** — 内存驻留、懒加载与卸载
- **G** — 写档路径
- **H** — 崩溃一致性与修复
- **I** — 客户端/服务器双画像
- **J** — 世界生成与存档
- **K** — 方块ID与Registry
- **L** — 版本管理与迁移
- **M** — 地图内容更新与分发
- **N** — 线上接口预留
- **O** — 玩家/世界边界
- **P** — 工具链
- **Q** — 实现深挖
- **R** — 批评与失败边界
- **S** — 完整性评估与建议

## 执行摘要全文

1. **规范字节与物理压缩必须拆层。** `[Verified]` Zstd 规定的是可互操作的解码格式，不是唯一编码；真实 issue 显示版本、实现或构建路径变化会改变压缩输出。只在 page 上写 `Zstd`/`Lz4`，不足以让 Rust 与 C# 在任何时候产出相同压缩字节。目标契约应把内容哈希绑定到未压缩的规范逻辑字节，物理容器再独立压缩；若坚持哈希压缩后字节，就必须冻结编码器版本、参数、线程模式、字典字节和黄金向量。〔S069–S075〕
2. **Authority 与浏览器 Replica 不宜共用同一物理容器。** Authority 需要事务写、WAL、checkpoint、compaction、精确 durability ack；浏览器更适合不可变、可 Range 读、内容寻址的 shard/cache。两端应该共享逻辑 page codec、坐标/版本语义和测试语料，而不是强制共享文件布局。Zarr sharding、COG、PMTiles 已证明“多个内块共用对象 + 索引 + Range”是成熟静态读取路线。〔S049–S055〕
3. **Minecraft Java 解析器必须按 DataVersion 分叉。** `[Verified]` 1.13 Flattening 抛弃数值 block data；1.16 起 palette index 不再跨 64-bit long；1.17 将实体移入独立 region；1.18 移除外层 `Level`，把 block states 与 biomes 都放进 section paletted container，并引入负 Y/384 高度。把这些规则揉成一个“Anvil parser”会产生静默错块，而不是整洁报错。〔S007–S014〕
4. **NBT 字符串是跨版转换的隐蔽雷。** `[Verified/Reported]` Java `DataInput/DataOutput` 使用 Modified UTF-8；Bedrock 工具链通常按标准 UTF-8。Chunker 已有真实 issue 表明错误处理会损坏文本 NBT。导入器必须在源 adapter 层把字符串解码语义显式化，不能让通用 UTF-8 库默默替代。〔S002–S004〕
5. **冷启动不是“把世界读完”，而是分级可用。** 应依次完成：release/world manifest 与 active checkpoint 校验、索引可用、认证 WAL 尾重放、世界元数据上线、关键 AOI 请求、异步 IO/解压/解码、唯一结构提交点原子安装、碰撞/网格/呈现。玩家只能在必需碰撞与权限数据 Ready 后进入；Tick 可先运行，但不能把 `NotLoaded` 当空气。完整世界后台加载对于无限/大世界既不必要也不可完成。
6. **异步加载完成必须作为“候选批次”进入唯一提交点。** IO 线程不得直接发布 chunk。提交点要重新验证 world/release、请求代次、绑定 revision、预算与邻接依赖；过期完成丢弃或重排。这样才能同时满足异步吞吐、`LatestAtBegin` 不重绑和单帧唯一结构提交。
7. **Replica 的“脏”必须拆成三类。** 服务器权威但尚未持久化的 dirty 与客户端无关；客户端缓存的权威 chunk 可以丢并按 hash/revision 补；只有尚未获服务器确认的本地预测/命令 overlay 不可丢。若沿用一个 `Dirty` 状态和 `DurableEviction` 语义，浏览器会被不必要地锁死在内存中。
8. **Minecraft 导入必须经过版本化语义 IR。** Java 与 Bedrock 的容器、字节序、字符串、block identity、biome、entity storage 都不同，无法安全做字节搬运。正确管线是：源读取器 → 版本归一化 IR → 显式映射 artifact → 目标 canonical page → 校验/报告 → staging 原子激活。映射版本、未知 block 策略、裁剪/平移和丢弃项必须成为导入证据，而不是日志里的一句 warning。〔S017–S034〕
9. **迁移 DAG 只是执行骨架，还缺版本语义和历史语料库。** 必须分开记录 container、voxel schema、snapshot/WAL、content registry、mapping set、worldgen 与 release compatibility；增加旧格式只读 adapter、迁移规划器、所有历史 fixture 的 golden corpus。改 chunk 尺寸、坐标语义或高度范围通常是全量重分区，不能假装成加字段式读时升级。
10. **当前画像最大的完整性缺口不是 WAL，而是“契约外工程面”。** 磁盘容器/索引、空间回收、索引重建、备份与修复、客户端缓存失效、block ID 语义、worldgen 版本、地图底图/玩家改动 overlay、历史迁移语料和诊断工具均未冻结。它们不补，系统会在首个大地图、首个浏览器内存压力、首个 Minecraft 非当前版本世界或首个内容更新时具体失效。

## 证据读取约定

- 正文的 `〔Sxxx〕` 回指根目录 `sources.md`。
- `Verified / Reported / Estimated` 含义见主报告开头。
- 本包不包含第三方源码、仓库下载、截图或二进制。

## 最重要的阅读提示

B/C/E/F是最厚的证据章；L/M没有因“线上发布不展开”而缩减；N严格保持接口预留。S章含40项完整性缺口、两张时序图、导入决策、双端驻留策略、未冻结项实验建议与风险清单。
