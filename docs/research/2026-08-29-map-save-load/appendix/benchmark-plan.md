# 定案前 Benchmark 与故障注入计划

本计划不预设通过数字。每个结果必须记录硬件、操作系统/浏览器、构建、commit、codec版本、输入hash和采样方法，最终由产品延迟/内存/磁盘预算填入 Gate。

## 1. 参数矩阵

- Page：`8³ / 16³ / 32³`。
- Chunk：`16³ / 32³ / 64³`，且 chunk 必须由整数个 page 构成。
- 表示：dense u32、palette 1/2/4/5/8/16/32 bit、sparse 1%/5%/20%/50%、全同质。
- Codec：None、Zstd 若干明确 profile、LZ4 明确 frame/block profile；有/无 immutable dictionary。
- Container：immutable 3D shard+WAL、SQLite WAL、候选 LSM/KV、Range projection shard。
- 平台：Rust server x64/arm64；.NET WASM Chrome/Firefox/Safari，桌面与至少一个受限移动设备。

## 2. 数据集

1. Minecraft Java 1.16、1.18、现代LZ4样本；负坐标、`.mcc`、entities/POI。
2. Bedrock现代 actor/subchunk样本。
3. 目标风格体素：全同质、层状地形、建筑、洞穴、随机高熵、极稀疏。
4. 编辑轨迹：单点热点、刷子、爆炸、均匀随机、Zipf热点、多玩家相隔很远。
5. 老化轨迹：块反复增大/缩小/删除、snapshot/WAL轮转、迁移与base overlay更新。

## 3. Codec/确定性

- Rust/C#各自编码两次，1/2/N线程，Debug/Release、不同locale/timezone；比较canonical bytes。
- 交叉读写固定点；合法/非法 corpus错误类别一致。
- 测 encode/decode/hash CPU、峰值临时内存、压缩比、随机单page解压。
- codec版本升级时只要求logical bytes/hash稳定；若physical profile承诺可复现，再比较compressed bytes。

## 4. 随机读与冷启动

- index cold/warm、NVMe/SATA或产品真实盘、网络Range冷/热cache。
- 分解：index、syscall/request、IO、decompress、hash、DataFix、runtime decode、commit wait、mesh/collider。
- 场景：出生点、直线高速移动、传送、十/百玩家分散轨迹。
- 输出 p50/p95/p99/max、读取bytes/request count、cache hit、过期候选丢弃数、SpawnSafe/ViewReady时间。

## 5. 写、WAL、快照

- 单block、单page、整chunk写；同步/批量fsync profile。
- 从commit到DurabilityAck的分布；WAL queue/backpressure；write amplification。
- capture在低/高mutation率、不同pin预算下的额外内存和tick p99。
- page diff与chunk diff的bytes、CPU、恢复链随机读；按累计diff/full比决定重做全量阈值。

## 6. 崩溃/损坏

在每个 open/write/short-write/flush/fsync/rename/current-pointer/ack/截断点 kill 进程；注入 ENOSPC、EIO、权限、校验失败、重复record、hash-chain断裂、索引重叠、`.mcc`缺失。恢复断言只能是上一有效世界或完整新世界，不能出现被激活的半版本。重复恢复/迁移仍幂等。

## 7. 浏览器

- 不同硬cap和AOI，跟踪WASM heap growth、GC pause、managed/native/JS buffers。
- IndexedDB/OPFS命中、配额回收、persist拒绝、隐私模式。
- Range 206、返回200整文件、ETag变化、断网/乱序/慢速；验证body上限和cache正确失效。
- 优先丢mesh/cache，确认LocalPendingCommands不丢。

## 8. Minecraft导入

- 1/2/N worker确定性；同输入两次输出完整manifest/page hash相同。
- 源/IR/目标逐块数量、坐标覆盖、palette/state/biome分布、block entity/entity/loss计数。
- 峰值RSS、总读写bytes、每GB/每百万voxel耗时只从测试报告产生，不预填。
- 在每partition和activation点中断，确认resume不重做已验证输出，目标旧版本不变。

## 9. 地图更新与overlay

- 稀疏/密集overlay、whiteout比例、1/5/20层、base移动建筑与冲突。
- 测 materialized read、rebase/compact、base GC roots、回滚后新mutation移植。
- 断言DeleteToAir在任意base更新后不因缺presence而复活。

## 10. 必须报告的统一指标

`logical bytes, physical bytes, allocated/live ratio, write amplification, space amplification, requests, bytes read, cache hit, decode/hash/migration/commit/mesh time, peak resident/pinned/COW/temp memory, durability latency, recovery time, index rebuild time, loss counters, deterministic mismatch count`。
