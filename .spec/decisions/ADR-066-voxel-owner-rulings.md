# ADR-066：R-00434 体素 BlockType 解析域与目录校验 Owner 裁决

状态：Draft（2026-09-05）
关联：[ADR-062](ADR-062-voxel-world-public-contract.md)、[`voxel-world-v1.json`](../../engine/wire/voxel-world-v1.json)、[`voxel.md`](../knowledge/features/voxel.md)

## 背景

R-00434 的契约复核发现三处公共语义不能留给实现仓自行解释：系统哨兵 BlockType 的占位含义、未登记类型是否进入普通材质 / 行为模板解析、以及官方目录行缺失字段与未知材质 token 的失败优先级。Owner 已授权本 ADR 将三项裁决落到 `lumio.voxel-world.v1`；契约 JSON 是唯一机器真值，本文只记录决策与迁移边界。

## Owner 裁决

### D1：BlockType=2 是 ECS 占用，BlockType=3 是结构占位

`BlockType=0` 是空气，`1` 是错误块，`2` 是 ECS occupancy placeholder，`3` 是 structure placeholder；R-00434 的 `entity_occupancy_placeholder` 用例固定写 `BlockType=2`。这四个值是 typed built-in sentinels，不经过官方目录或房间局部映射的普通材质 / 行为模板解析。

### D2：解析域采用 typed-sentinel + ordinary-row hybrid

- `0..3` 只能按契约登记的 typed built-in sentinel 解析。
- `4..255` 是系统预留、不可解析；任何被接纳但无法解析为哨兵、已登记官方目录行或已映射房间局部行的 BlockType，统一以新增稳定错误 `unregistered_block_type` 拒绝。
- `256..8388607` 的普通材质类 / 行为模板解析只允许已登记的官方 `blockCatalog` 行。
- `8388608..16777215` 的普通解析只允许随存档提供的已映射房间局部行。
- `room_local_type_without_mapping` 仍保留给存档映射完整性规则；本 ADR 的 admission/resolution 失败语义使用 `unregistered_block_type`，不改写既有错误 ID。

### D3：结构优先，未知材质只针对非空 token

目录行校验先检查六个必需字段（`blockType`、`name`、`materialClass`、`behaviorTemplate`、`assetRef`、`stateLayout`）是否缺失、为 null 或空字符串。任一字段缺失 / 空值时返回 `block_catalog_row_incomplete`；只有结构完整且 `materialClass` 为非空但不在声明表中的 token 才返回 `unknown_material_class`。因此「未知材质 + 另一个缺失字段」由结构错误优先。

## 迁移边界

这是开发态公共契约变更，无已部署体素消费者，不开兼容窗口。架构仓先更新 `engine/wire/voxel-world-v1.json`，再由 Voxel 实现仓复制同一源并更新其稳定错误映射；不得在实现仓另造 sentinel、解析域或校验优先级。`unregistered_block_type` 追加在既有错误列表末尾，既有错误 ID 与数值映射保持不变；ABI 生成物只能由 `eng/generate-abi.mjs` 从更新后的 `native-abi.json` 与 wire 源重生成。

## 验证 Fixture

- `entity_occupancy_placeholder` 断言 occupancy sentinel 为 `BlockType=2`。
- `blockId.resolution.testCases` 覆盖四个 sentinel、预留段、已登记 / 未登记官方段和已映射 / 未映射房间局部段。
- `blockId.resolution.catalogValidationCases` 覆盖 materialClass 缺失、空值、非空未知 token、与其他缺失字段并存时的结构优先级，以及完整已知行。
- `node eng/verify-wire.mjs` 执行上述机器断言；ABI 输出的 source SHA、错误列表和 52/57/110 计数由生成器校验。
