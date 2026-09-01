# 0009 · root-abi generator 的适配器边界与 §8.3 接口偏离：摘要链锚点必须落在 lock 上

- 日期:2026-08-29
- 状态:生效

## 背景

LCE-P0-005 要把架构源的 Root ABI 制品接进本仓。规格 §8.3 给了 `GenerateAbiRequest` /
`GeneratedAbiArtifacts` / `AbiCompatibilityReport` 的字段面，§4 定了「只消费架构源生成制品」，
§3.6 定了只读生成协议。落地时有三处规格没说、但一旦选错就会让整条摘要链变成自证的问题：

1. **compiler 身份怎么表达。** §8.3 写的是 `compiler_path: PathBuf` + `compiler_digest: Digest256`。
   但上游 `compiler_hash()` 的口径是 `sha256(tools/lumio_contract.py ‖ tools/lumio_generate.py)`
   ——身份由**两个文件**共同决定，单个路径表达不了；而把期望摘要做成入参，等于允许调用方
   自带答案来对账。
2. **摘要链的根锚在哪。** 上游 `root-abi-bundle.json` 声明了 compiler 身份、inputHash 与每份
   产物的期望摘要。它在只读镜像里，是一个**可被就地改写的本地文件**。
3. **本仓自产的登记文件谁来背书。** `metadata/native-managed-abi.json` 与
   `reports/layout-report.json` 不是上游产物，bundle 里没有它们的摘要。

第 2、3 两条在首版实现里都答错了，且**都不是靠推理发现的**：审查实测出两条绕过路径——
改一份无锚点产物、或改 bundle 里的三个 `outputFiles.digest`，再按同一规则重建 descriptor，
`just check-generated` 全部绿灯。本 ADR 记录修正后的边界，以及为什么必须这么定。

## 决策

### 1. 摘要链的每一环都要有**外部**锚点，descriptor 不得自证

链条自上而下：

| 环节 | 锚点 | 若无此锚点 |
| --- | --- | --- |
| 上游 bundle | `architecture.lock.json` 的 `requiredPathSha256["packages/abi/root-abi-bundle.json"]` | 改 bundle 的 outputFiles.digest 即可整体移动锚点，六份产物全部可替换 |
| compiler | bundle 声明的 `compiler.digest`（本仓复算两文件拼接） | 换 compiler 无人发现 |
| 输入集合 | bundle 声明的 `inputHash`（本仓按 `inputSet` **重算**，不照抄） | 照抄只证明「读到了这个数」，证明不了「镜像里就是那份输入」 |
| 三份上游产物 | bundle 声明的 `outputFiles[].digest` | — |
| `metadata/native-managed-abi.json` | 镜像里 `inputSet` 声明的同一份文件（逐字节相等，已被 inputHash 钉死） | 整份替换后重建 descriptor 即可全绿 |
| `reports/layout-report.json` | 由上游 `layoutProfile` **现算**的内容 | 同上 |
| `generated-contract-artifact.json` | 按同一规则**重建后逐字节比对** | 「它记的每一条都对得上」证明不了「它自己没被改」 |

**判据**：任何一份产物，如果它的唯一约束来自 descriptor，而 descriptor 又是从同一批盘上
字节重建出来的，那就是恒等式，不是校验。被背书者与背书者必须不同源。

### 2. §8.3 的两处接口偏离

- `compiler_path` + `compiler_digest` → **`compiler_directory: PathBuf`**。身份由目录下两个
  固定文件共同决定；期望摘要只从上游 bundle 取，不接受调用方传入。这是**收紧**：入参形式
  允许调用方自带答案，目录形式不允许。
- `build_plan: FrozenBuildPlan` → **`frozen_plan_path: Option<PathBuf>`**。计划经
  `composition::verify_frozen_plan` 读取（ADR 0006 第 8 条：消费者不得自建第二套解析器）。
  Root ABI 的输入集合**全部**来自上游 `inputSet`，与 BuildPlan 无交集；计划在这里的作用是
  交叉核对——它记的 architecture 基线与提交必须与本仓 lock 一致，否则「按 A 计划构建、
  按 B 基线生成 ABI」会一路无声走到运行时。
  **`Option` 意味着这条核对可被跳过**：给了才查，不给则不查。CLI 总是给（justfile 的
  `generate-abi` recipe 传 `--plan`），库调用方可以不给。这是相对规格的**弱化**，记在这里
  而不是只写在 doc-comment 里。

### 3. 上游 validator 必须真跑，报告不得声称没做过的检查

上游 `emit_root_abi()` 的第一件事是 `validate_abi_document()`（schema + ADR-040 语义），
其 docstring 明写「在写出任何一个输出字节之前拒绝非法 ABI 文档」。本仓的驱动脚本重写了
`emit_root_abi` 的主体来只取三个 emitter，**必须显式补回这次调用**——首版跳过了它，同时
`AbiCompatibilityReport` 的 `schema_valid` / `semantic_rules_valid` / `symbols_valid` 三个字段
写死 `true`。那是谎报做过的检查，而该报告会被下游多张卡消费。

规则：`AbiCompatibilityReport` 的每个字段只能反映**本次真的做过**的检查。做不到的项要么
补上检查，要么改成能表达「未做此项」的形式，不得填 `true`。

`verify_generated` 不跑 validator（回读校验必须能在没有工具链的机器上进行）。它的
schema/semantic 依据是「descriptor 以 `validatorRan: true` 重建后逐字节相符」——重建时传的是
**字面量** `true`，不是从 descriptor 读回来的值。

这个区别是本 ADR 第 1 节判据的直接应用，也是第一版修复踩过的坑：把 `validatorRan` 与
`entrySymbol` 从被校验对象自己取回去参与重建，逐字节比对对这两个字段就恒真，改它们
`verify-generated` 直接 exit 0。**回读期无法外部重建的字段，不要假装它受保护**——要么以
字面量参与重建（等于「取值不符即拒收」），要么换一个有外部真值的来源。`entrySymbol` 属
后者：它的外部真值是镜像里的 ABI 文档，已被 inputHash → bundle → lock 钉死。

### 4. 上游输出集合以上游为准

驱动脚本按 `module.ABI_OUTPUT_FILES` 取输出清单并断言与本仓适配清单集合相等。上游**新增**
第 4 份输出时必须在这里响亮失败，而不是被本仓写死的三条清单静默忽略——「输出集合精确
比对」若只对本仓清单精确，对上游就是不精确。

### 5. compiler 的运行根目录用镜像内容临时拼装

`lumio_contract` 在**导入时**就按仓库根布局读 fixture，只把 `tools/` 指过去不够。本仓在临时
目录里用**只读镜像**的 `schemas/` `fixtures/` `ids/` `packages/` 加锁定 `tools/` 拼一个一次性
contract root，用完即删。镜像本身不动（受 lock 约束且已置只读），**绝不使用架构源仓工作区**
——那是不受 lock 约束的可变输入。

## 后果

- 生成与回读各多读一次 lock 与镜像输入，代价是几个文件的 I/O，换来锚点不可被就地移动。
- `check-generated` 与 `check-contracts` 的职责仍然分离（后者管镜像整体完整性），但本卡不再
  依赖调用者「记得两条都跑」——bundle 对 lock 的校验在生成器内部完成。
- LCE-P0-014 / LCE-P0-008 等消费 `AbiCompatibilityReport` 的卡，可以按字段面信任它：每个字段
  要么对应本次真实执行的检查（`symbols_valid`、三项 layout、两个 hash），要么对应一条
  「不满足即在返回之前失败」的前置（`schema_valid` / `semantic_rules_valid`——它们为 true
  的唯一路径是 descriptor 以 `validatorRan: true` 重建成功）。没有字段是无条件常量。
- 本 ADR 不改变 ADR 0001—0004、0006、0008 的任何边界，不新增依赖边（`root-abi-generator ->
  composition` 在 ADR 0004 第 3 条冻结的允许边内），不定义任何公共 Schema / ID / FFI 语义。
