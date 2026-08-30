# 术语对照与歧义

| 本报告/目标术语 | 外部常见术语 | 必须避免的混淆 |
|---|---|---|
| Chunk | Minecraft chunk、voxel block、World Partition cell | 目标Chunk为signed XYZ三维单元；Minecraft chunk是16×16水平柱，内部有section |
| Page | Minecraft section、Bedrock subchunk、Zarr inner chunk | Page是目标hash/diff/compression子单元，不自动等于任何源尺寸 |
| Region / Shard | `.mca` region、Godot region、Zarr shard、PMTiles archive | 都是对象聚合，但索引位置、更新/事务和维度不同 |
| Snapshot | checkpoint、save image | 目标有Full/Partial/Diff与严格cut；客户端cache segment不是完整snapshot |
| Diff | delta、patch、overlay | Snapshot Diff引用严格base；player overlay还需要whiteout和base lineage |
| Presence | loaded flag、availability、fill value | `NotLoaded/Pending/Unavailable`绝不能变成air/fill value |
| Dirty | modified/unsaved | Authority未durable、Replica可丢cache、本地pending command是三种不同语义 |
| Canonical bytes | wire format、logical encoding | 不应与任意Zstd/LZ4物理输出混为一谈 |
| Container profile | database/file format/storage backend | 逻辑page相同可落入不同Authority/Replica容器 |
| DataVersion | game version、format version | Minecraft对象迁移标记，不等于所有容器/content/worldgen版本 |
| Registry | block ID table、palette | 全局semantic registry、每存档local registry、每page palette是三个层级 |
| Worldgen identity | seed | seed只是输入之一；还需算法、版本、配置、registry和随机语义 |
| Base map | generated terrain、published map | immutable来源；玩家修改存overlay，不等于当前materialized world |
| Whiteout | tombstone、explicit air | 表示上层明确删除下层值；没有它无法区分继承空气 |
| Ready | decoded、mesh-ready、spawn-safe | 目标chunk state Ready仍需具体能力gate；数据Ready不等于碰撞/视图Ready |
| Revision | file timestamp、DataVersion | 内容逻辑修订；加载驻留变化不应凭空推进内容revision |
