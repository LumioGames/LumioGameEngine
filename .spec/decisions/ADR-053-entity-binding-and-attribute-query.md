# ADR-053：连接绑定与 NetEntityId Attribute Query 公共契约（开发态冻结）

状态：Accepted（2026-09-01，RM-00011 Room Review Rulings「契约先行、五仓并行」原则；关闭决策日志 Open Decision「Freeze the common connection-to-Entity binding and ECS Attribute Query API, including visibility, permission, revision and stale-entity failure semantics」）
取代：无

## 背景

RM-00011（ECS Formal Entity and Chat Vertical Slice）的三张实现卡——R-00347（Runtime 通用绑定/查询）、R-00349（Client ReplicaWorld 映射与呈现）、R-00350（Server 五分钟重连与过期）——消费同一张连接绑定与属性查询公共面。该面此前只存在于决策日志散文（`../reviews/2026-09-01-ecs-formal-entity-chat-decision-log.md` 的「Common Entity Binding and Attribute Query」「Entity Interaction Contract」两节），没有任何可对齐的真值载体；Owner 裁决（2026-09-01）：先冻协议再并行，各仓靠冻结契约与用例对齐、禁止互相等待。

本仓已于 59866ec 删除旧基线契约系统（schemas/ids/fixtures/packages/tools），现行开发态公共契约载体是 `engine/wire/` 下的自包含契约 JSON（ADR-052 先例）。经 Owner 2026-09-01 裁定，本契约按「ADR → 契约定义 JSON → 正反用例」交付，落点 `engine/wire/entity-binding-and-query-v1.json`；登记与索引同步由主 loop 串行合并时完成。

## 决策

1. **契约真值载体**：`engine/wire/entity-binding-and-query-v1.json`（contractId `lumio.entity-binding-query.v1`，version 1）是绑定与属性查询语义的唯一真值；下游实现不得另写一份语义真值。契约措辞宿主无关：当前 C# MVP 宿主实现，后续切片级最小 Rust 宿主复跑同一语义（Room Review Rulings 宿主轨道）。
2. **绑定五元组与不变量**：每条已准入连接的绑定记录是 `AccountId + RoomId + NetEntityId + EntityType + ConnectionGeneration` 五元组，且仅此五字段。一个账号同一时刻至多在一个 Room 活跃；重连重绑与顶号重绑（顶号语义冻结于 R-00357 / `lumio.account-port.v1`）继承同一实体：NetEntityId 不变、connectionGeneration 严格递增；AccountEntity 对象引用任何方向不跨入 Game World，只允许 AccountId 值。
3. **受控 Attribute Query 面**：查询按已声明 `AttributeId`（文法 `<ComponentType>.<attributeName>`）寻址单实体的单属性；成功结果携带类型化值与 `observedRevision` / `observedTick`（stale 读检测的事实来源）。调用域二分：`server-authoritative`（限定 Room 范围、Simulation Owner Thread 语义角色、按服务端策略可读任何已声明属性）与 `client-replica`（限定本端 ReplicaWorld，只能读已复制且可见且已实际送达的属性）。不存在按任意属性名查找、SQL 表达式、存储寻址或跨 Room 的通路——这些形式一律显式拒绝，**拒绝分类顺序固定**：①存储寻址识别器（特征下限：括号行/槽寻址、`Storage.` 前缀、存储路径分隔符）→ `storage_access_forbidden`；②文法 → `invalid_attribute_id`（SQL 表达式与自由格式名显式归此码）；③已声明判定 → `undeclared_attribute`。识别器特征是下限：实现可对未列用例更严格，不得更宽，且不得改判 `invalidCases` 已钉死的 `expectedRejection`。
4. **五结局失败矩阵全部显式**：`non_existent` / `stale_generation` / `invisible` / `unauthorized` / `tombstoned`。destroyed 或 tombstoned 引用永不解析到替代实体（`resolvesToReplacement` 恒为 false）；未知、已销毁、墓碑实体永不复活。invisible（不可见）与 unauthorized（无权限）严格分层：先判实体/属性可见性，再判可见性主体资格。
5. **每属性三维独立声明**：`persistence`（ephemeral|persistent）、`replication`（not-replicated|replicated）、`visibility`（server-only|room-public|aoi-scoped|claim-scoped）。客户端可读 ⇔ replicated 且可见性允许且已实际复制进其 ReplicaWorld；persist-only 与 server-only 字段对客户端以 `invisible` 结局不可达。属性声明表随各组件公共契约冻结（首个租户 ChatComponent 声明在 R-00355 / `lumio.gameplay-envelope.v1`），本契约只定义声明结构与判定规则。
6. **宿主无关措辞审计**：契约全文只使用领域语义角色（Simulation Owner Thread、ReplicaWorld、权威存储）与宿主无关数据形状；无任何宿主类型名、线程 API、委托或指针语义进入公共面。整数以 `u64` 位宽记法表示（沿 hello-wire-v1 先例，指无符号 64 位位宽，不绑定任何宿主类型系统）——此为唯一记法豁免。审计结论记录于契约 `wordingAudit` 节。

## 替代方案

- **SQL 式 / 按属性名自由寻址**：决策日志「Entity Interaction Contract」明确排除——查询面不是数据库 API，不允许直连存储。
- **复活旧 Schema/ID/Fixture/Baseline/镜像体系承载本契约**：该体系已随 59866ec 删除，Owner 裁定不重建；开发态以 `engine/wire/` 契约 JSON 承载（ADR-052 先例）。
- **并入 hello-wire-v1**：RM-00011 边界明确不扩展 hello-wire-v1；且绑定/查询是 API 语义而非 Hello 传输消息。
- **由各实现仓自行定义再对齐**：违背「契约先行」裁决——三仓并行将失去对齐基准，联调期才发现语义漂移。

## 接口

- 契约文件：`engine/wire/entity-binding-and-query-v1.json`（sections：identityModel / binding（record+invariants+operations：selfLookup、resolveByConnection、resolveByNetEntityId）/ attributeQuery（addressing、callerScope、request、success、failure 双变体）/ attributeDeclarations（structure、dimensions、readRules）/ outcomes / errorCodes（含 classification 顺序、存储寻址识别器特征、多违例单码裁决）/ fieldSemantics / limits / wordingAudit / testCases（10 例，五结局各≥1）/ invalidCases（10 例，含路径分隔符存储寻址与多违例裁决）/ boundary）。
- 关键词表：结局码 `non_existent` `stale_generation` `invisible` `unauthorized` `tombstoned`；请求错码 `invalid_attribute_id` `undeclared_attribute` `cross_room_reference` `storage_access_forbidden` `binding_not_found` `invalid_binding_shape` `scope_violation`。
- 上游依赖：无（本契约不依赖其他 RM-00011 契约的先期冻结；对 `lumio.account-port.v1` 与 `lumio.gameplay-envelope.v1` 只作引用占位，方向为「消费其结果」，无字段级耦合）。
- 下游消费者：R-00347（实现）、R-00349（客户端范围）、R-00350（重连/过期语义）；`AccountId`/`NetEntityId` 语义同时被 R-00346、R-00348 间接消费。

## 失败语义

- 实体解析与属性查询的语义失败都以五结局矩阵之一显式返回，绝不静默、绝不返回替代实体；请求形错误走 `requestError` 变体，不占用五结局码。
- `stale_generation`：携带过期 connectionGeneration（或等价纪元）的引用在新纪元生效后必得此结局；消费方义务是重新 selfLookup，而非重试旧引用。
- `invisible` 与 `unauthorized` 的判定顺序固定：先可见性后权限，二者不混用；`invisible` 不泄露「实体存在」之外的事实。
- `tombstoned`：墓碑保留期内恒为此结局；过期遗忘后为 `non_existent`；墓碑永不复活。
- 失败结果是**双变体单码模型**：结局类失败 `{outcome: 五结局码}`（outcome 即分类，不携带 code）；请求形错误 `{outcome: request_error, code: requestErrorCodes 之一, detail}`（文法、未声明、跨 Room、存储寻址、绑定形状、作用域违规）。两码表不相交，`expectedRejection` 单码即完整可判定。
- `cross_room_reference` 的判定依据是实体归属表（服务端权威归属或客户端 ReplicaWorld 映射），**不解析 netEntityId 字符串本身**（该字符串不编码 Room）。
- attributeId 三步分类（存储寻址→文法→已声明）与实体归属/请求形状判定互不抢占：实现必须全部检查，命中项写入 `detail`，对外只返回最高优先级单码。多违例单码裁决顺序（左高右低）：`invalid_binding_shape` > `scope_violation` > `cross_room_reference` > `binding_not_found` > attributeId 分类链。请求形错误优先于结局类失败。
- 存储寻址识别器特征是下限；`invalidCases` 已钉死 `expectedRejection` 的输入不得改判（SQL 表达式归 `invalid_attribute_id`）。
- 契约内嵌 `testCases`（含五结局逐例）与 `invalidCases`（逐例声明 `violates` 与 `expectedRejection`）是失败语义的可执行口径；统一校验器（`eng/verify-wire.mjs`，随 R-00355 落地）执行全部用例。

## 兼容影响

- 开发态新契约：无存量消费者，无兼容窗口义务；hello-wire-v1 与 `engine/abi/native-abi.json` 不受影响（`node eng/generate-abi.mjs` 保持零差异）。
- 下游影响：R-00347 / R-00349 / R-00350 的公共 API 以本契约为字段与失败语义基准；实现仓各自落地契约一致性测试（正反用例从本 JSON 派生）。
- 属性声明表内容（ChatComponent 等具体声明）不在本 ADR 冻结范围，随 R-00355 冻结；本契约的声明结构是其唯一合法载体结构。

## 迁移方案

无需迁移（新能力）。正式硬化阶段若将 wire 契约升级为版本化公共合同，以本 JSON 为源承接，语义不变；届时失败矩阵与三维声明词表作为兼容判定的基线。

## 验证

- `node -e` 自检（本卡 worktree 内执行）：JSON 合法、必含节齐全、`contractId`/`version` 正确、五结局各≥1 个 testCase、全部 invalidCases 带 `violates`/`expectedRejection` 且引用码存在于词表、AttributeId 文法与示例一致——通过（输出见交付报告）。
- `node eng/generate-abi.mjs` 零差异（ABI 面不受影响）。
- `node .spec/tools/spec-lint.mjs`：本 ADR 的 README 索引登记由主 loop 串行合并时统一完成（四张 Wave 0 契约卡并行，README 属共享文件）；草稿分支上第 3 项（decisions 登记覆盖）预期失败，登记后复跑通过——已知且已上报。
- 统一校验：`eng/verify-wire.mjs`（R-00355 交付）合入后对本 JSON 执行全部内嵌正反用例；主 loop 合并时验证。
