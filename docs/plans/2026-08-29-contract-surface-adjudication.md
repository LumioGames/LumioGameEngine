# 2026-08-29 · 契约面裁决(委任):D-9 / D-3 / D-4 / D-015 定音,D-1 归 V1.5 批

> 裁决人:总调度(用户 2026-08-29 委任「你有自主决策的权利,加快开发效率」;委任纪律见 R-00257 先例——证据支撑什么才裁什么,裁决成文落库、事后报备)。
> 裁决底账:[`../reviews/2026-08-28-gate-p0-delivery-and-escalations.md`](../reviews/2026-08-28-gate-p0-delivery-and-escalations.md);实证输入:GameRuntime 8 条验收项受阻、Client netstandard2.1 编译墙、CoreEngine S3「无可消费 Rust ContractTypes 制品」(R-00015 BLOCKED 评论)、NativeCore R-00083(ADR-040 §7)。
> 本文只记**裁决与理由**;规范正文由执行卡落 ADR(编号现查抢占,下述编号为占位意向)。

## 裁决一(D-9):立二进制 canonical profile——`LumioBinV1`

- **裁决**:在 `CanonicalJsonV1` 之侧新增二进制 canonical profile(意向 ADR-047):little-endian、定宽整数(u8/u16/u32/u64/i32/i64)、数组与字节串 u32 长度前缀、字符串 UTF-8 + u32 字节长、无对齐填充、字段序 = schema 声明序、无浮点默认(需浮点的域 ADR 单独申报口径)。发布物:`packages/binary/lumio-bin-profile.json`(机器可读声明,含 normalization 同款结构化规则)+ 自校验 Golden(正反例)。
- **理由**:ADR-010:20 的「same canonical codec rules」当前无指向物;ADR-035 冻结了 voxel payload 的域内排序/偏移/哈希但缺 primitive 层;snapshot payload 字节是公共的("every conforming encoder")。**additive 发布,不动任何既有必需集 → V1.4 内可落。**
- **被拒替代**:MessagePack 等第三方编码(依赖审计、跨实现字节不确定性、AOT 成本;Runtime 的 `MessagePack 3.1.8` 依赖裁定移除);「各域自定 primitive」(公共字节无权威规范 = 复读 D-1 的病灶);varint(定宽换确定性与实现简单)。
- **连带**:ADR-010:20 改引用 LumioBinV1;`CHECKSUM_DOMAIN.md` B 档补 Golden 与 domain tag(gate 文档 D-9 附带项)。

## 裁决二(D-3 + D-4 + D-015):generated 面从 catalog-only 升级为可消费本体

- **D-3 裁决**:发布 closed contract set 的**类型本体 + 可执行 validator**(意向 ADR-048)。范围:`ConfigTable / ProcessorDescriptor / TxnJournalRecord / CommandLogRecord / WalRecordEnvelope / EntityIdentity / ReplicationEnvelope / SessionRevisionVector` 八类的 C# 与 Rust 本体(由 schema 生成,字段序 = schema 声明序),Protocol/Permission Validator 从目录表升级为可执行校验(ADR-022 的原意);**ordinal 权威 = ids/index.json + schema 声明序,生成器是唯一发射者**。
- **理由**:catalog-only 已被三仓独立实证不可消费(Runtime 定义数全为 0 的 8 条验收项、Client 编译期无类型、CoreEngine S3 只能字节内嵌自派生)。「不得自行发明公共合同」+「必须用 generated validator」这对约束在 catalog-only 下无解——解铃只能是上游发本体。
- **D-4 裁决**:`packages/csharp/*` 全部多目标 `netstandard2.1;net8.0`(Unity 与 .NET Host 共同面)。包装形态改动,契约字节不变,additive。
- **D-015 裁决**:capability key 派生**放行且收口到生成器**——由架构生成器从 Capability 注册表(9 值)发射常量(Rust/C#/C 三形态),下游只消费生成物、维持禁手写;ADR-040 §7 补裁决记录节。解锁 NativeCore R-00083。
- **落地形态**:全部 additive(新增生成物 + consumers 登记 + 既有 Golden 不变)→ **V1.4 内可落,不跳基线**。

## 裁决三(D-1):方向定音,执行归 V1.5 跃迁批

- **方向裁决**:下行状态载荷 = ReplicationEnvelope 的 typed body 扩展(FullSnapshot 增 `stateBlocks`、Delta 增 `changedBlocks`),内容按 mappingSet 声明序以 **LumioBinV1** 编码、`payloadHash` 绑定、长度受 ADR-045 既有上界约束;上行输入 = 新 MessageType `InputCommand`(ids 注册表新值,owner GameRuntime)+ 独立 input envelope schema(同样 LumioBinV1)。
- **为何不即刻落**:closed body 的必需集扩字段与 MessageType 枚举增值均属基线事件(schemas/README 变更规则 + ADR-045 恰好等于断言 + 七仓镜像)——**归 V1.5 跃迁批,只跳一次基线**(D-11 口径),同批:R-00009(LoadBackend/Packaging 枚举对齐)、ADR-040..048 Draft 转 Accepted、~~OperationId namespace 发布(需 NativeCore 提值)~~**(已于 2026-08-29 裁决出批,见 V1.5 批规划项 4)**、D-5 tag/冻结点、trust P2 台账两条(signedAt preimage、时间窗比较)。
- **先行项**:D-1 的 ADR 草案与 Schema/Fixture 草稿在跃迁批前成文备好;A1-β/GameRuntime replication 卡在 V1.5 落地后解锁。

## 执行编排(模块并行总图,按重要度 × 复杂度)

| 车道 | 形态 | 内容 | 前置 |
|---|---|---|---|
| 架构执行 | 单会话(卡 A→B 串行,同仓工具链耦合) | 卡 A:LumioBinV1(裁决一);卡 B:generated 面升级(裁决二) | 无——立即 |
| V1.5 规划 | 总调度自做 | 卡 C:跃迁批清单/顺序/D-1 ADR 草案 | 卡 A/B 定型后 |
| Server | 单会话(已派 SA) | A1-α 冻结面联机骨架 | 无 |
| GameRuntime | 单会话(B 落地后派) | W1 六卡(config/persistence/ecs) | 卡 B |
| Client | 单会话(B 落地后派) | 10 卡 + 5 缺陷(多目标解编译墙) | 卡 B(D-4 部分) |
| Voxel 收尾 | 双 QA 道(SV-α 在跑/SV-β 待点) | 14 张验收卡核销 | 无(低优先) |
| Game 内容 | 单会话(即派) | 三方向比稿材料备齐(方向终裁归用户) | 无 |

风险与自检:裁决一/二均 additive,失败面 = 生成器改动波及 compilerHash(CoreEngine 门禁已按 R-00263 与 tools 解耦,波及面可控);D-1 押后不阻塞 W1/Client/A1-α;9 周时间盒下 V1.5 批是唯一大动作,预算一周窗口。
