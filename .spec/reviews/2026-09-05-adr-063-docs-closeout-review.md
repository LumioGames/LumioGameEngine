---
name: 2026-09-05-adr-063-docs-closeout-review
description: ADR-063 文档收口深审报告——五份概要 / tick.md / bomber-slice.md / rules 世界模型节的逐条证伪与裁决;派 R5-01 或 Runtime 缺口卡前查
metadata:
  type: doc
  status: 已交付
---

# ADR-063 文档收口 · 深审报告（2026-09-05）

审查对象：工作树相对 `HEAD`（`f980a09`）的完整改动——16 个已跟踪文件 + 4 个未跟踪文件（ADR-063、`tick.md`、`bomber-slice.md`、`docs/adr/ADR-063-*` 符号链接）。范围外：`.spec/plans/2026-09-04-platform-ms1-dispatcher-prompt.md`（改动前已存在）。审查基准：Owner 已批准的计划（`~/.claude/plans/smooth-floating-sparkle.md`）+ ADR-063。级别：深审（触碰 `rules/system.md` 红线面）。

## 结论：退回

三条 P1：① 新增的 spec-lint 8d 校验看不见 ADR-063「兼容影响」自己点名的 7 份文档中的 4 份（裸文件名写法逃过正则，负例实测不报错）；② ADR-058 第 14 / 15 条仍写「普通字段…存档打 `[Persist]`」而不在修订记录内，ADR-063 把这段措辞错归到第 1 / 7 条；③ `ds-server.md` 客户端子图仍说「ECS + GAS + 体素同一确认 / 回滚单元」，与 ADR-063 第 7 条「体素不进预测世界」直接打架。其余为 P2，可随同一次退回顺手修。

## 覆盖声明（深审七维）

| 维 | 审了什么 | 结果 |
|---|---|---|
| 1 验收标准 | 计划「一、裁决清单」15 行逐条对 ADR-063 决策段与知识文档落点；计划「四、验证」四项 grep 与链接检查；交付方 5 条声称逐条重跑 | 落点齐；声称 1 / 3 / 5 成立，2 / 4 部分成立（见证据核验） |
| 2 正确性 | 六份概要 + tick.md + bomber-slice.md 之间关于预测 / 墓碑 / `[Persist]` / 一进程一房间 / 网格不倒退 / 帧内三层的交叉一致性；tick.md 对 Runtime `TickPhase.cs` / `PhaseContractTable.cs`；bomber-slice.md 对 LumioGame ADR 0015 / 0017 与 `stage0-kernel-contract.md`；spec-lint 8d 逻辑 + 隔离副本负例 | P1-1 / P1-3 / 多条 P2 |
| 3 安全 | 纯文档，无对外暴露面、无密钥；`rules/system.md` 新节为设计约束非鉴权面 | 不适用，已确认无新增暴露面 |
| 4 护栏与规范 | `node .spec/tools/spec-lint.mjs` / `node eng/verify-wire.mjs`；frontmatter（name / description ≤ 120 / type / status 枚举）；README 导航行 = description；decisions/README 索引；`docs/adr` 镜像符号链接；`rules/system.md` 无 frontmatter、口径「必须 / 只能 / 不得」；生成物未手改（`engine/wire/*.json` 零改动）；未提交 | 过；P2-8（口诀属「怎么做」） |
| 5 测试 | 文档交付无测试；8d 校验交付未附负例证据，由本审在隔离副本补做（tick.md / workflow.md 去掉回指 → 报错；gas.md 去掉回指 → 静默） | P1-1 |
| 6 提交卫生 | 未提交；一批改动一类事（ADR-063 落地）；计划外改动三处（P2-14） | 可接受，须在交回物声明 |
| 7 沉淀 | `knowledge/README.md` 登记 tick.md / bomber-slice.md；`decisions/README.md` 登记 ADR-063；AGENTS.md 指针；`lessons.md` 未记本次暴露的「兼容影响裸文件名逃过 lint」 | 过；建议补 lessons 一行 |

## Findings

### P1

**P1-1　spec-lint 8d 对 ADR-063 自己点名的文档有 4/7 不校验（`.spec/tools/spec-lint.mjs:172`；`.spec/decisions/ADR-063-…md:82`）**
- 证据：8d 只匹配 `knowledge/(features|standards)/<名>.md` 全路径；ADR-063「兼容影响」第 82 行写法是「`knowledge/features/ecs.md`（…）、`gas.md`（…）、`voxel.md`（…）、`ds-server.md`（…）、`ecs-entity-chat.md`（…）…新建 `knowledge/features/tick.md`、`knowledge/features/bomber-slice.md`」——只有 ecs / tick / bomber-slice 三份被看见。ADR-060 / 061 点名 `ecs-entity-chat.md` 同样是裸名。
- 失败场景（隔离副本实测）：把 `gas.md` 里全部 `ADR-063` 替换掉 → `spec-lint: OK`；同样操作 `tick.md`（全路径点名）→ `✗ tick.md: 被 ADR-063「兼容影响」点名,但正文未回指 ADR-063`。ADR-063 验证 Fixture 11 对 gas / voxel / ds-server / ecs-entity-chat 不成立；交付声称 4「对现有 ADR-060 / 061 / 062 / 063 通过」对这些文档是空过。
- 修法：正则加一档——在「## 兼容影响」段内同时抓反引号里的裸 `<名>.md`，按 `features/` → `standards/` 顺序解析（都不存在才报「文档不存在」）；或反过来把 ADR-063 第 82 行全部改成全路径（但 ADR-060 / 061 正文按本轮口径不改写，所以改正则才是根治）。修完把 gas.md 负例跑一遍作证据。

**P1-2　ADR-058 第 14 / 15 条仍是「普通字段打 `[Persist]`」，不在修订记录内；ADR-063 把该措辞错归到第 1 / 7 条（`ADR-058-…md:45-46`；`ADR-063-…md:30, 79`；ADR-058 修订记录 `:122-130`）**
- 证据：ADR-058 第 15 条原文「私有字段 = 组件上不是 `Sync<T>` 的普通字段（…存档打 `[Persist]`）」；第 14 条样例「`[Persist] AccountId`（Server.cs）」。第 1 条原文只说「忘打 `[Persist]` 是使用者 bug」、第 7 条只说「标注集修订为 `[EcsComponent]`、`[Persist]`」——都没有「普通字段可打 `[Persist]`」这句。ADR-063 第 6 条「修 ADR-058 第 1 / 7 条中『普通字段可打 `[Persist]`』的措辞」指错了条目；修订记录只追加了第 1 / 5 / 7 条。
- 失败场景：worker 读 ADR-058 第 15 条（唯一定义「私有字段」的地方）拿到的仍是「普通字段存档打 `[Persist]`」，与 ecs.md:625 红线「`[Persist]` 打在普通字段上 = 永不」相反；lint 抓不到（ADR 之间不校验）。
- 修法：ADR-063 第 6 条与「兼容影响」改为「第 1 / 7 / 14 / 15 条」；ADR-058 修订记录追加第 14 条（样例改 `[Persist] Sync<string> AccountId = new(Scope.None)`）与第 15 条（私有字段不再可打 `[Persist]`，要存档改 `Sync<T>(Scope.None)`）两行。

**P1-3　`ds-server.md` 客户端仍把体素放进「确认 / 回滚单元」（`ds-server.md:74`；`:60` ③）**
- 证据：`:74` 客户端子图节点「副本应用 + 预测<br/>ECS + GAS + 体素同一确认/回滚单元」；`:60` 体素接缝「③ 同一个确认/回滚单元（防『钻石进包了方块还在』的鬼状态）」。ADR-063 第 7 条 / gas.md:230 M7② / tick.md:94 / ecs.md:380 M10① 全部改为「预测世界只含 ECS 实体，**体素不进预测世界**、不拍快照」。
- 失败场景：Client 实现方按 ds-server.md 图给体素拍每帧快照进预测重建（正是 ADR-063 替代方案段被否的「整帧倒带含体素」），与 GAS / ECS 文档冲突，收口审查两边各有依据。
- 修法：`:74` 改「副本应用 + 预测<br/>ECS + GAS 预测世界从确认世界重建；体素只进确认世界」；`:60` ③ 改「同一个提交点、服务器同一个 fail-stop 单元（客户端预测不含体素，见 tick.md §6）」。

### P2

- **P2-4　gas.md M7 残留旧口径**（`gas.md:232`「预测被拒时整帧回滚后只有错的那件事被修正」；`:231`「重入义务由框架的整帧重放吸收了」）：同模块 ①② 已改「预测世界重建」，做完标准与不干什么没跟着改。修法：「预测被拒时预测世界重建后…」「…由预测世界重建 + 重放吸收」。
- **P2-5　ds-server.md 底线 3 与 M7① 打架**（`ds-server.md:24` vs `:219`）：底线仍是无条件「最多欠 N 帧必发」，M7① 已改「在本帧配额允许的前提下…不作无条件时限承诺」。底线是「将来砍任何功能都不能碰」的句子，应与 M7① 同口径：「关键类别有硬饥饿上限（配额允许时最多欠 N 帧必发；装不下即进阶梯）」。
- **P2-6　ecs.md M4④ 残留「每连接只有一个游标（书签）」**（`ecs.md:310`）：ADR-063 第 11 条 / ds M6 已改「书签 + 有界的可见性 / 进度 / 确认元数据」。改为「每连接只有书签 + 有界元数据，绝不存世界副本或组件值副本」。
- **P2-7　ecs.md M9① 写法 `[Persist] AccountId`**（`ecs.md:371`）：与 §4.5 样板 `:424` `[Persist] Sync<string> AccountId = new(Scope.None)` 不一致，字面上就是 `:625` 红线「`[Persist]` 打在普通字段上」。改为 `[Persist] Sync<string> AccountId(Scope.None)`。
- **P2-8　rules/system.md 四问口诀顺序与本节例子冲突，且属「怎么做」**（`rules/system.md:17`）：口诀「要不要服务器逻辑？要 → 实体。不要且不动 → 体素。只有画面 → Local Entity」——本节 `:14` 的例子「UI 假人」不要服务器逻辑、不动，按口诀第二问落成体素，按第四条应是 Local Entity（首匹配歧义）。另外文件头 `:4` 声明「不写怎么做」，判定流程放这里违反本文件自述。修法：口诀第二问改「只有画面 → Local Entity」、第三问「不要且不动 → 体素」；或按计划原落点把口诀移到 `architecture.md` §1.1（该节现在只复述模型、没有口诀）。
- **P2-9　ADR-063 引用「GAS 8a / 8b」，gas.md 没有这个编号**（`ADR-063-…md:25, 31, 51`）：gas.md 全文无 `8a` / `8b`；对应的是 M7 ①（不发小票）②（服务器永不回退）。改为「GAS M7 ①②」。
- **P2-10　ADR-060 修订记录第 7 条漏记快照字段**（`ADR-060-…md:126`）：ADR-060 第 7 条原文还有「快照只存『下一个号』」，已被 ADR-063 第 3 条「已占到哪」取代，修订记录该行只记了「客户端不推导」。补一句。
- **P2-11　bomber-slice.md 引信数字标错来源**（`bomber-slice.md:12, 96`）：「2.5 秒引信…取自 LumioGame 现行口径」——LumioGame `docs/specs/bomber/stage0-kernel-contract.md:120` 冻结 `fuseMs = 2100`（A/B 1800 / 2400）；`design.md:225` 的 2.5 秒是被 v0.4 替换的旧推定值。其余四个数字（61×61、火力 2、3.5 格/秒、6 个半心）核对一致。改 2.1 秒或去掉「现行口径」四字。
- **P2-12　bomber-slice.md 样板让玩法传 `reason`**（`bomber-slice.md:146` `World.Commands.Destroy(人.Self, reason: terminated)`）：`reason` 由引擎派生——结构销毁一律 `terminated`，`left_aoi` 只在投影层出视野时产生（ecs.md:351 M6、ADR-063 第 2 条）。玩法 API 带这个参数会诱导游戏代码传 `left_aoi`。改为 `World.Commands.Destroy(人.Self)` 并注释「销毁记录 reason = terminated 由引擎盖」。
- **P2-13　交付声称 2 收窄了计划的 grep**：计划「四、验证」要求「Room 多槽」「把特效搬到正式实体」零命中；声称改写成「Room 多槽：进程内」并删去后者。实际 `ds-server.md:251 / 310 / 320` 与 `ecs.md:382` 仍含原短语（均为「只是回图触发项」「不存在…这一步」的否定语境，语义符合裁决）。内容可接受；但声称口径与计划口径不一致须在交回物明说，不能静默改写验收 grep。
- **P2-14　计划外改动未声明**：`workflow.md:16` 加 ADR-061 链接（隔离副本实测：去掉后 8d 报错，即为过新 lint 而加）；`ecs-entity-chat.md:94 / 114` 加 ADR-060 #8 / #10 注（核对 ADR-060 原文准确）；`save-load.md:157` M2③ 计划两处标「不动」但加了一句（与 ADR-063 第 6 条一致）。三处无害，交回物应列「改动清单」并说明理由。
- **P2-15　nit**：`bomber-slice.md:30`「LumioGame 答复 §①」缺右括号；`tick.md:18`「前 9 步里业务只做三件事」——业务实际只在第 3 / 4 相（表内第 1 / 2 / 5–9 相「不跑业务」），建议「第 3 / 4 相里业务只做三件事」。

## 证据核验

实际运行（工作树只读；负例在 scratchpad 隔离副本 `lintcopy/` 上做，改完即复原）：

| 声称 | 命令 / 检查 | 结果 |
|---|---|---|
| 1 lint / wire | `node .spec/tools/spec-lint.mjs` → `spec-lint: OK`，exit 0；`node eng/verify-wire.mjs` → `tests 41 / pass 41 / fail 0`，exit 0 | 成立。注：`engine/wire/*.json` 零改动（`FullSnapshot` 23 处、`appliedInputSequence` / `left_aoi` 0 处），verify-wire 是基线证据，不是 C-1 两字段的证据 |
| 2 grep | 「一张字典」「不存在「整图加载」」「只与书签大小相关」「整块内存拷贝」「没人看不记账」：五份概要 + tick / bomber-slice 零命中；「Room 多槽」3 命中（ds 251 / 310 / 320）、「把特效搬到正式实体」1 命中（ecs 382，否定语境）；tick.md 被 ecs 1 / gas 1 / ds 1 / voxel 2 / save-load 1 处链接；§4.5 样板 `[Persist]` 六处（410 / 413 / 418 / 424 / 455 / 456）全在 `Sync` / `SyncList` 上；`Friends` 为 `SyncList<string>` + 注释「元素是 AccountId」（同样板 `AccountId` 即 `Sync<string>`，语义成立；计划字面 `SyncList<AccountId>` 未按字面落） | 按声称口径成立；按计划口径见 P2-13 |
| 3 落点 / 追加 | 计划 15 行裁决逐条对 ADR-063 第 1–15 条与文档行号；`git diff` ADR-058 / 060 只有文末 `+` 段，正文无 `-` 行 | 成立；但修订记录覆盖不全（P1-2、P2-10） |
| 4 8d | 头注释 `:29-31` 已登记；工作树 lint 通过；负例：tick.md 去回指 → 报错 ✓；workflow.md 去 ADR-061 → 报错 ✓；gas.md 去回指 → **静默** ✗ | 部分成立（P1-1） |
| 5 rules 口径 | 新节 `:7-17` 六条对 Owner 原话逐句：两种东西 ✓ / 静态→体素 ✓（多「不需要服务器逻辑」限定，计划草案已如此、Owner 已批）/ 动→实体 ✓ / 特效→Local ✓（多「UI 假人」例）/ GAS 组件可预测可建实体 ✓ / 箱子两半（计划草案）✓ + 「不得在体素里存业务数据」与 voxel.md:154 一致；「服务器回滚」在 ADR-063:25 如实标「待 Owner 复核」，rules 用中性词「预测回滚」；无 frontmatter ✓ | 成立；口诀见 P2-8 |
| tick.md vs Runtime | 13 相名 / 顺序 / 可写域 / 失败类 / 可取消点（第 8 相起 NotCancellable）/ 可见性（第 10 相起 AfterCommit）/ 唯一提交点 `GasAndEventFinalize`，逐格对 `TickPhase.cs` 与 `PhaseContractTable.CreateDefault()` | 全部一致 |
| bomber-slice vs LumioGame | 炸弹 = CS 实体、引信 → 爆炸态 → 留火 → 销毁、四臂到达长度、半心点：对 ADR 0017 与 kernel-contract §「炸弹兼任它自己的爆炸」一致；「取自 LumioGame、只作例子」声明在 `:12` | 一致；引信数字见 P2-11 |
| 结构 | tick.md / bomber-slice.md description 73 / 58 字符、status `设计中`；README 导航行与 description 逐字相同；decisions/README 有 ADR-063 行；`docs/adr/ADR-063-*` → `../../.spec/decisions/…` 与 ADR-062 同式；链接可达由 lint 覆盖 | 全过 |

未核实：LumioGame ADR 0018 内容（只核 0015 / 0017 + kernel-contract）；五组验收场景的可执行性（设计层，暂无实现）；Owner 对「服务器回滚」措辞的最终意图（ADR 已如实标待复核，归 Owner）。

## 方案疑虑（不阻塞，交主 loop）

- 8d 的根治口径：ADR「兼容影响」写法应有一条硬规矩（全路径或裸名二选一），否则每张 ADR 都可能以另一种写法逃过校验——建议记 `lessons.md` 一行，并在 spec-lint 头注释 8d 里写明两种写法都抓。
- `rules/system.md` 新节把「四问口诀」放进红线文件，与该文件「不写怎么做」的自述冲突（P2-8）；主 loop 定：改文件自述，还是把口诀挪回 `architecture.md` §1.1。

## 复核（第二轮，2026-09-05）

复核对象：工作树相对 `HEAD`（`012e792`）的全部改动（同上：16 个已跟踪 + ADR-063 / tick.md / bomber-slice.md / `docs/adr/ADR-063-*` 符号链接；`plans/2026-09-04-platform-ms1-dispatcher-prompt.md` 范围外）。级别：复核——退回项逐条证伪 + 一遍回归扫描。环境：工作树只读；8d 负例在 scratchpad 的 rsync 副本（排除 `.git`）上做，副本 ADR-063 复原后与工作树 `cmp` 一致，工作树 `git status` 条目数前后同为 22。

### 结论：放行

三条 P1 全部成立地修好；P2-4 … P2-12、P2-15 与追加的 Scope 封闭枚举均核实。回归扫描无 P0 / P1；新发现 6 条 P2（其中两条由修复本身带入），不阻塞放行，可随下一次改动顺手处理。

### 退回项逐条复核

| 项 | 结论 | 证据 |
|---|---|---|
| P1-1 8d 裸名 | **成立** | `spec-lint.mjs:160-188` 同时抓全路径（strict）与反引号裸名（features → standards → 知识根解析；解析不到忽略）；头注释 `:29-31` 写明两种写法。副本负例：gas / voxel / ds-server / ecs-entity-chat / ecs 五份各自去掉全部 `ADR-063` → 各报「被 ADR-063「兼容影响」点名,但正文未回指」exit 1；全路径改 `tickk.md` → 报「兼容影响点名的文档不存在」；加 `nonexistent-doc.md` 裸名 → OK（静默）；加 `lessons.md` / `workflow.md` 裸名 → 分别对知识根 / standards 报错（解析链成立）。误报检查：ADR-058 / 060 / 061 兼容影响段裸名 `ecs-entity-chat.md` / `README.md`（跳过）/ `README.en.md`（正则不匹配、知识根亦无此文件）在工作树上 lint OK，无误报 |
| P1-2 ADR-058 第 14 / 15 条 | **成立** | ADR-063 `:4` 取代行、`:30` 第 6 条、`:79` 兼容影响均已写「第 1 / 5 / 7 / 14 / 15 条」；ADR-058 修订记录 `:131` 新增第 14 / 15 条一行（`[Persist] Sync<…>(Scope.None)`，仍放 `.Server.cs`），与 ecs.md `:371` / `:424` 同口径 |
| P1-3 ds-server 客户端体素 | **成立** | `ds-server.md:74` 节点改「确认世界 + 预测世界重建（体素不进预测世界）」；`:60` ③ 改「服务器侧同一个提交点、同一个整帧作废单元…客户端预测世界不含体素（gas.md M7）」；与 gas.md `:230` / tick.md `:94` / ecs.md `:380` 一致 |
| P2-4 gas.md M7 | 成立 | `:231`「预测世界重建 + 重放吸收」、`:232`「预测世界重建后只有错的那件事被修正」；gas.md 全文「整帧」零命中 |
| P2-5 ds 底线 3 | 成立 | `:24`「配额允许下最多欠 N 帧必发，装不下就进阶梯」，与 `:219` M7① 同口径 |
| P2-6 ecs M4④ | 成立 | `:310`「每连接只有书签 + 有界的可见性 / 进度元数据，绝不每连接存世界副本或组件值副本」 |
| P2-7 ecs M9① | 成立 | `:371` `[Persist] Sync<string> AccountId = new(Scope.None)`（`.Server.cs`），与 §4.5 `:424` 字面一致 |
| P2-8 口诀落点 | 成立 | 口诀移至 `architecture.md:29-33` §1.1，顺序 ① 只有画面 → Local ② 需要且会动 → CS ③ 需要但静态 → 体素 ④ 开 ADR；「UI 假人」按 ① 首匹配落 Local，歧义消除。`rules/system.md:7-17` 只留六条「必须 / 不得」+「判定必须按 §1.1 顺序」一句，链接可达 |
| P2-9 GAS 8a / 8b | **部分成立** | `:31` 改为「GAS 裁决流水 8a / 8b（reviews/2026-08-30-gas-architecture-decisions.md，即 gas.md M7 ①②）」、`:51` 改「8a「不发小票」（gas.md M7 ①）」；**`:25` 仍是裸「GAS 8b 定「服务器永不回退」」**，且出现在 `:31` 定义之前（见下 P2-A）。流水 `:58-59` 确有 8a / 8b 行 |
| P2-10 ADR-060 第 7 条 | 成立 | 修订记录 `:109` 补「快照存发号器「已占到哪」，崩溃后从已占段之后继续（ADR-063 第 3 条）」 |
| P2-11 引信 | 成立 | `bomber-slice.md:12`「2.1 秒引信 = LumioGame kernel contract `fuseMs = 2100`」、`:96` `毫秒换帧(2100)`；全文「2.5」零命中（`fuseMs = 2100` 本身沿用第一轮对 LumioGame `stage0-kernel-contract.md:120` 的核对，本轮未重开） |
| P2-12 `Destroy` | 成立 | `:146` `World.Commands.Destroy(人.Self)` + 注释「reason 由引擎盖」；五份概要 + tick + bomber 中 `Destroy(…reason` 零命中 |
| P2-15 nit | 成立 | `bomber-slice.md:30` 右括号已补；`tick.md:18`「业务只在第 3 / 4 步跑，只做三件事」 |
| 追加：Scope 封闭枚举 | 成立 | `ecs.md:297`「封闭枚举五种：Room / Aoi / Owner / Claim / None」；ADR-060 第 5 条列 Room / Aoi / Owner / Claim、第 12 条 Claim + `claimBy` 写法、ADR-063 接口段「Scope 枚举增 None」三者合起来正是这五值；§4.5 样板用到 Room `:410` / Owner `:413` / Claim `:418` / None `:424, 455, 456`，`ClientRpc(Scope.Room)` `:449`，无枚举外取值；`Friends` 为 `SyncList<string>`（AccountId）与 ADR-060 修订记录第 5 / 12 条「持久名单存 AccountId」一致 |

### 回归扫描新发现（均 P2，不阻塞）

- **P2-A　ADR-063 `:25` 残留裸「GAS 8b」**：P2-9 只改了 `:31` / `:51`；`:25` 第 1 条末句「GAS 8b 定「服务器永不回退」」在 `:31` 给出定义之前出现，读者第一次遇到「8b」无处可查。改「GAS 裁决流水 8b（gas.md M7 ②）」即可。
- **P2-B　ADR-058 修订记录 `:126`「以下三条的措辞」已失真**（修复引入）：追加第 14 / 15 条后是四个条目、覆盖五条，「三条」是旧计数。改「以下各条」。
- **P2-C　`ds-server.md:60` 接缝 ② 与 ③ 重复**（修复引入）：② 已是「同一个提交点」，③ 改后开头又写「服务器侧同一个提交点、…」。③ 去掉「同一个提交点、」只留「服务器侧同一个整帧作废单元…」。
- **P2-D　ADR-063 `:82` 兼容影响漏列 `save-load.md`**：`save-load.md:157` M2③ 已回指 ADR-063 第 6 条（第一轮 P2-14 记的计划外改动），但兼容影响「已按本 ADR 改写」清单没有它；8d 只查「点名的文档有没有回指」，抓不到「改了的文档有没有被点名」这个方向。补一个 `save-load.md`（M2）。
- **P2-E　ADR-063 `:30` 仍把『普通字段可打 `[Persist]`』用引号归到第 1 / 7 条**：ADR-058 第 1 条只是隐含（「忘打 `[Persist]` 是使用者 bug」）、第 7 条无此句，引号读作原文引用不成立；`:79` 兼容影响的表述（「第 1 条（未标注字段 / `[Persist]` 措辞）、第 7 条（`Sync<T>` 与 `[Persist]` 关系）」）已准确，`:30` 照它改成转述即可。
- **P2-F　`voxel.md:28 / :225 / :305`「同一个提交点、同一个回滚单元」**（非本轮改动，HEAD 已有）：指服务器整帧作废单元，与 ds-server `:60` ③ 改后的「整帧作废单元」是同一件事，但 ADR-063 第 7 条之后「回滚」一词更容易被读成客户端预测回滚（体素恰恰不进）。建议下次动 voxel.md 时统一为「整帧作废单元」；本轮不要求。

### 证据核验（实际运行）

| 命令 | 结果 |
|---|---|
| `node .spec/tools/spec-lint.mjs`（工作树，追加本节前后各一次） | `spec-lint: OK`，exit 0 |
| `node eng/verify-wire.mjs` | 7 contracts green；`tests 41 / pass 41 / fail 0`，exit 0（`engine/wire/*.json` 零改动，仍是基线证据） |
| 8d 负例（副本，见 P1-1 行） | a 基线 OK；b 五份去回指各报错 exit 1；f 全路径不存在报错；g1 解析不到的裸名 OK；g2 / g3 知识根 / standards 裸名可解析并报错；h 复原 OK，副本 ADR 与工作树 `cmp` 一致 |
| 回归 grep（五份概要 + tick + bomber + architecture） | 「整帧回滚 / 整帧快照 / 三家一起倒 / 同一确认 / 最大号推导 / 整块内存拷贝 / 只有一个书签 / 没人看不记账 / 只许 Sync / 2.5 秒 / 假实体换真实体」：命中仅 gas.md:270 与 bomber-slice.md:159 的否定语境（「没有「搬特效」」「无认领 / 搬特效代码」）、ecs.md:285「整帧快照 + 日志」（服务器帧作废，非客户端预测）、ecs.md:686 黑话「下一个号」（指已占段之后，语义正确）、voxel.md 三处「回滚单元」（P2-F）；无残留旧口径 |
| `docs/adr/ADR-063-*` | 符号链接 → `../../.spec/decisions/ADR-063-…md`，与 ADR-062 同式；lint 8b 覆盖 |

未核实：LumioGame `fuseMs = 2100` 未重开源文件（沿用第一轮）；P2-13 / P2-14 交回物口径按主 loop 指示不核；Owner 对「服务器回滚」措辞的最终意图仍归 Owner。

### 方案疑虑（不阻塞）

- 8d 的裸名只在反引号内抓，且对「不动」语义的点名同样要求回指（g2 / g3 实测）：兼容影响段若写「`save-load.md`：不动」会被要求回指。这是刻意的严格面，但应在 `lessons.md` 或 8d 头注释里写一句「兼容影响只点名改了的文档；不动的不要写文件名」，否则下一张 ADR 会踩。
- 8d 是单向校验（点名 → 回指），「改了但没点名」（P2-D）没有机器兜底；可考虑反向：Draft ADR 编号出现在 `knowledge/features|standards` 文档正文里、但该 ADR 兼容影响没点名它 → 报错。留主 loop 判断是否值得加。
