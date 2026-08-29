# 0010 · Root ABI 运行时对「上游明确未冻结」的语义保持缺位，只做不透明相等校验，不自造判定键

- 日期:2026-08-29
- 状态:生效

## 背景

LCE-P0-006（R-00020）要实现 `bind_root_api` 与 `RootApiTableView`。规格 §8.3 列出的运行时接口里有一项：

    pub fn supports(&self, capability: CapabilityId) -> bool;

`CapabilityId` 在本仓就是 `lumio-core-contracts` 对架构源 ID Registry `Capability` namespace 的 1:1 派生（LCE-P0-003）。要实现它，必须回答「`capability_bits` 的第几位对应哪个 `CapabilityId`」。

但锁定架构基线 `LGE-V1.4-2026-08-27` 的 ADR-040「What this bundle deliberately does not freeze」把这个问题**显式列为未冻结**：V1 既没冻结 `lumio_root_api.capability_bits` 是 bitmask 还是计数，也没冻结任何位位置；ID Registry 的 `Capability` numeric 是**枚举序数而不是位位置**（与 WebSocket transport profile 被拒发 `Capability` id 同一理由）。ADR-040 给出的处置是一句禁令加一条出路：

> A consumer must not derive a capability key from either source; a repository-private key is the only correct model until the semantics are confirmed.

于是 §8.3 的这一项与锁定架构合同直接冲突：照 §8.3 写出来的任何 `supports()` 都必然是「本仓自造的位映射」，正是 ADR-040 禁止的那件事。

同族的第二处：单张 API table 的 `version` 期望值只发布在 `metadata/native-managed-abi.json` 与 bundle JSON 里，**没有任何 Rust 可消费的常量**；运行时发布闭包（规格 §3.7）里为读它引入 JSON 解析依赖不成比例。

本仓已有同型先例：`lumio-core-contracts` 对 §6.1 中上游尚未把本仓列入 `consumers` 的类型，选择「保持缺位、不建同名临时 struct」并在 crate 文档写明 seam。

## 决策

**上游明确声明「未冻结」的语义，本仓一律保持缺位；能验证的部分退到不需要该语义的最强判据。**

1. **不提供 `RootApiTableView::supports(CapabilityId)`。** 不建同名临时方法、不造位映射、不用 alias 或本地枚举绕道。上游确认位语义后按独立需求卡补齐。
2. **`capability_bits` 只做精确相等校验。** 绑定期把它当**不透明 u64**，与已发布值逐位全等才通过，不匹配映射 `NativeAbiMismatch`(1004)。相等比较不需要知道它是 bitmask 还是计数——这是在语义未冻结前唯一不含假设的判据；子集判定 `(actual & required) == required` 已经假设了 bitmask，不可用。消费方只能经 `capability_bits()` 拿到原值。
3. **不校验单张 API table 的 `version`。** 绑定期读出并如实公开（`ApiTableView::version()`），但不与任何期望值比较；缺的是可消费真值，不是校验意愿。
4. **两处缺位都必须是显式行为，不是遗漏。** crate 文档「与 §8.3 的两处偏差」逐条写明理由与出处；第 3 条另配守卫测试
   `table_version_is_surfaced_verbatim_and_not_asserted`——将来若要开始比较，必须先有可消费的上游真值并改这条测试，不能顺手加个字面量 `1` 就算数。

判据边界：本决策只覆盖「上游写明未冻结 / 未发布可消费真值」的项。凡是上游已发布的（entry symbol、abi_version、struct_size 下界与对齐、指针宽度、endianness、slot 偏移、slot 非空），一律照常强校验，不借本条豁免。

## 后果

- **接受**：`RootApiTableView` 的公开面比规格 §8.3 少一个方法，属对源规格的显式偏差，已在交回物与 crate 文档声明；下游要做能力判定，当前只能自持仓内私有键（正是 ADR-040 指的 repository-private key）。
- **接受**：per-table `version` 目前是「读了但不判」，一张 version 被改坏的 table 能通过绑定。风险有界——它的 `struct_size`、slot 偏移与 slot 非空仍全量校验，而这三项才是会导致误读内存的部分。
- **换来**：本仓不产生第二套公共 ABI 语义。ADR-040 之所以点名 WebSocket transport profile，就是因为「消费方按序数推位位置」这类自造键一旦落地，上游再冻结真语义时会与既成事实冲突。缺位可以随时补，自造的键要先拆。
- **换来**：缺位是可机器发现的。两处都有测试或文档钉住，不会退化成「当初大概忘了写」。

参见：[0009](0009-root-abi-generator-adapter-boundary.md)（同族判据：不得声称没做过的检查）。
