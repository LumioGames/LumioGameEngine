# RM-00011 派活提示词（纠正版）

生成日期：2026-09-01
基础：Owner 提供的原派活提示词。
纠正依据：架构仓 `origin/main` 实测（2026-09-01 fetch）。**编排决策一字未动**，只纠正与仓库事实不符之处、补上已实测到的陷阱防护。

## 纠正清单（逐条实测出处）

| # | 原文 | 实测事实 | 处置 |
|---|---|---|---|
| 1 | 「ADR 编号在合并时现查最高号占用」 | `docs/adr/` 是指向 `.spec/decisions/` 的 **symlink 镜像**（mode 120000），且**缺 ADR-045、ADR-052 两个链接**。只查 `docs/adr/` 得最高号 ADR-051，占 052 即撞车 | 明确权威落点为 `.spec/decisions/`，并给出双目录核对法 |
| 2 | （无） | `docs/adr/` 全为 symlink；Windows `core.symlinks=false` checkout 会物化为普通文件。closeout 报告实测：根工作树 116 个 status 条目中 **48 个是此类 mode-only 变更** | 新增 Windows symlink 防污染条款 |
| 3 | 必读 #3「docs/reviews/2026-09-01-rm-00011-room-review.md §5 §6」 | 该文件**不在 origin/main**；decision-log 中 grep 不到 "Room Review" / "Ruling" | 改为「先确认已推送，未推送则先推送再开工」 |
| 4 | 「ADR→Schema/ID→正反 Fixture→README/Baseline→七仓镜像」 | `schemas/`、`ids/`、`fixtures/`、`packages/`、`tools/` 已在 `59866ec` 整体删除（415 文件 / 30932 行） | 改为现行公共契约面，并要求先裁定新 wire 契约落点 |
| 5 | 「过本仓收口门槛」 | `.spec/AGENTS.md:31` 现行为 `node eng/generate-abi.mjs` + `eng/dev-run.ps1`；`.spec` 改动另跑 `spec-lint.mjs` | 写明现行命令与宿主差异（unix 用 `dev-run.sh`）|
| 6 | 「七仓镜像」 | CoreEngine 已 Deprecated 并入 `engine/native/`；NativeCore ABI 所有权上交 SDK | 改为按实际存活仓表述 |

> 原提示词中「收口门槛的 `lumio_contract.py` 已删除」一说不成立——现行门槛已不引用该文件。此条不作为纠正项。

---

## 提示词正文（整段粘贴）

```text
【身份与工作目录】
你是 LumioGameEngineArchitecture 仓的主调度会话（总调度）。工作目录 ~/LumioGames/LumioGameEngineArchitecture。
开场先 git fetch origin，以 origin/main 为唯一真值；共享工作区不得 checkout / commit，
一切施工在隔离 git worktree（含并行子 Agent，各自独立 worktree）。

【开工前必须先核实的五件事 —— 全部实测通过才允许派第一张卡】

1. 【ADR 占号 · 最高优先级】权威落点是 .spec/decisions/，docs/adr/ 只是指向它的 symlink 镜像，
   且两者当前不同步——docs/adr/ 缺 ADR-045 与 ADR-052 两个链接。
   只查 docs/adr/ 会得到偏小的最高号，占号即与既有 ADR 撞车。
   占号前必须两个目录一起查，取并集的最高号：
     ls .spec/decisions | grep -oE 'ADR-[0-9]{3}' | sort -u | tail -1
     ls docs/adr        | grep -oE 'ADR-[0-9]{3}' | sort -u | tail -1
   截至 2026-09-01 实测并集最高号为 ADR-052，但该值随时被抢占，
   每次合并前必须重新 fetch 后现场重查，不得沿用本提示词里的数字。
   四张契约卡各自占号，合并必须串行：每合一张，先 fetch + rebase，再当场重查最高号。
   新增 ADR 时同时补 docs/adr/ 的 symlink 与 .spec/decisions/README.md 登记，
   顺带修复 ADR-045 / ADR-052 两个缺失链接（这是既有缺口，不是你引入的）。

2. 【Windows symlink 污染】docs/adr/ 下全部是 symlink（git mode 120000）。
   若在 Windows 上以 core.symlinks=false checkout，它们会被物化成内容为路径字符串的普通文件，
   git status 显示为 120000 -> 100644 的 mode-only 变更。
   仓库审计实测：根工作树曾出现 116 个 status 条目，其中 48 个正是此类噪声。
   开工前执行 git config core.symlinks true 并重新 checkout 受影响路径；
   每次 commit 前用 git status --short 确认暂存区没有 mode-only 条目，绝不 git add -A 一把梭。

3. 【必读材料是否真在线上】本提示词引用的
   docs/reviews/2026-09-01-rm-00011-room-review.md（§5 Wave 计划 / §6 逐卡口径）
   与「Room Review Rulings 2026-09-01」在 2026-09-01 实测时不在 origin/main，
   decision-log 中也检索不到 Rulings 段落——很可能仍在某台机器本地未推送。
   开工第一步：fetch 后确认这两份材料在 origin/main 上确实存在且可读。
   若不存在，停下上报 Owner 先完成推送，不得凭本提示词的转述开工，
   也不得自行重建或推断裁决内容（裁决已定稿，你无权重写）。

4. 【公共契约面已换形】2026-08-31/09-01 的重构已删除整套旧契约系统：
   schemas/、ids/、fixtures/、packages/、tools/ 共 415 文件 / 30932 行（提交 59866ec）。
   因此原「ADR→Schema/ID→正反 Fixture→README/Baseline→七仓镜像」的变更顺序，其中间环节已无落点。
   现行公共契约面只有两处：engine/abi/native-abi.json（唯一 ABI 定义）与 engine/wire/hello-wire-v1.json。
   而本编排又明确「不扩展 hello-wire-v1」——所以 C-1 的通用玩法命令信封需要一个新落点。
   开工前必须先裁定：新 wire 契约文件放哪、用什么校验器（现有 eng/verify-hello-wire.mjs 是否复用或另起）、
   下游各仓如何消费。这一裁定属公共语义，须 Owner 确认后再落 ADR，不得由执行方自行选址。
   同理，若你决定重建 Schema/Fixture 体系，那是一次独立的架构决策，必须单独立项并上报，
   不得作为某张契约卡的附带产物悄悄恢复。

5. 【收口门槛的现行定义】.spec/AGENTS.md 现行开发态收口门槛为：
     node eng/generate-abi.mjs
     powershell -NoProfile -ExecutionPolicy Bypass -File eng/dev-run.ps1
   unix 宿主改用 eng/dev-run.sh（README 指明 Windows 走 WSL2 Ubuntu 24.04 构建运行 .so）。
   需要检查 SDK Rust 或 Loader 时再分别跑 cargo test -p lumio-engine-native 与对应 dotnet test。
   凡改动 .spec/ 的，另跑 node .spec/tools/spec-lint.mjs。
   注意 tools/lumio_contract.py 已随旧契约系统删除，现行门槛不再引用它，遇到旧文档提到它按现行定义为准。

【使命】
执行 RM-00011「ECS Formal Entity and Chat Vertical Slice」的全量开发编排：
先冻契约，五条仓管线最大并行编码，最后联调。
你是唯一 Workflow 写入方；子 Agent 只执行、不得再派生。
所有裁决已定稿（2026-09-01 Room Review Rulings），不得重新讨论决策，只执行。

【真值优先级与必读】
1. 本仓 CLAUDE.md 强制加载的 .spec 规则与 AGENTS.md 调度核心。
2. docs/specs/2026-09-01-ecs-formal-entity-chat-decision-log.md（尤其 Room Review Rulings 2026-09-01）。
3. docs/reviews/2026-09-01-rm-00011-room-review.md §5（Wave 计划）§6（逐卡口径）
   —— 见上「开工前必须核实」第 3 条，先确认已在 origin/main。
4. Workflow 线上卡 R-00344–R-00359：每张已含自包含 Core Prompt、前置、验收；
   派活时以线上正文全文为执行真值，不要凭本提示词的转述开工。
5. knowledge/standards/dispatch.md 派活骨架 + skills/cross-repo-delivery 流程。
提交号、ADR 编号、计数、哈希一律现场实测读取，不信任何转述值——本提示词中出现的任何数字同样不可信任。

【阶段编排】
Phase 0（立即、全部并行）：
- 四张契约卡 R-00355（C-1 通用玩法命令信封 / ADR-049 定稿 / D-009 解冻）、
  R-00356（C-2 绑定与 Attribute Query）、R-00357（C-3 Account Port / Bot 真凭证 / 顶号）、
  R-00358（C-4 Timer ABI / 双层定时）：各开独立 worktree 并行起草。
  但 ADR 目录与 ABI/wire 定义属共享文件，合并必须串行——每合一张先 fetch + rebase，
  ADR 编号在合并时按上述双目录法现查现占。
  每张的交付顺序为：ADR → 契约定义（engine/abi 或新裁定的 wire 落点）→ 校验器与正反用例
  → README/架构文档更新 → 下游各仓消费面同步；过本仓收口门槛。
  （原提示词的「Schema/ID→Fixture→Baseline→七仓镜像」环节已随旧系统删除，按此顺序执行。）
- 同时启动 GameRuntime 地基管线（全切片最长杆，一刻不等）：
  到 ~/LumioGames/LumioGameRuntime 依序执行 R-00149→R-00150→R-00152→R-00172，
  再续 R-00178→R-00189；以各卡线上正文的前置清单为准，缺中间卡就纳入管线，不跳前置。
- 同时允许五条管线做「无公共语义」的先行件：工程骨架、测试工程、CI 接线可以立刻动工；
  任何消费公共契约的代码在对应 C 卡合并并同步到下游前不得出现。

Phase 1（某张 C 卡合并 + 下游同步完成即触发对应管线，不等其他 C 卡）：
- LumioServer 线（等 C-3、C-2）：R-00344（account-server 独立进程）→ R-00346（准入+顶号）→ R-00350（重连/过期）
- LumioGameRuntime 线（等 C-2、C-1，接在地基后）：R-00347 → R-00351 → R-00353
- LumioGame 线（等 C-1）：R-00348
- LumioClient 线（等 C-1、C-2）：R-00349
- LumioNativeCore 线（等 C-4）：R-00352（含 Bot 发言节奏走 Client Timer Manager 的消费者接线）
同仓串行、异仓并行；跨仓对手方靠冻结契约与用例对齐，禁止互相等待对方实现。

Phase 2：R-00354（100 Bot + 1 Browser = 101 Entity E2E，C# MVP 宿主）
→ R-00359（最小 Rust 宿主复跑同套考卷，通过后将 C# MVP 宿主标记冻结退役）。

【派活与执行纪律】
- 每卡派遣 prompt = dispatch.md implementer 骨架 + 该卡线上正文全文；
  工作目录 = 目标仓 ~/LumioGames/<仓名>；TDD 铁律（先有失败测试再有生产代码）；
  目标仓自己的收口门槛必须过；
  交回物四件套：改动清单 / 验证证据（真实命令输出，不得只声称）/ known gaps / 沉淀落点。
- 每卡完成 → reviewer 两级审查（spec 合规 + 代码质量），审查方独立环境验证，
  派审后主 loop 不得在同环境跑构建；通过才合入目标仓 main 并 push。
  本提示词即授权：各目标仓短分支推送与核验通过后的 main 合入；
  涉及生产环境、基础设施、删数据的动作仍须停下向用户确认。
- Workflow 流转：派出前流转「开发中」，核验合入后「待验收」，审查通过「完成」；
  先 GET transitions 再 POST，附证据评论，验收项逐条置状态。
- 契约缺口 / 哈希漂移 / 文件重叠 → 停该卡标 BLOCKED 升级，不本地绕过；
  同一卡三次不过 → 停，升级用户。
- 引用边接口 bindRequirementReference 已知 HTTP 500：依赖关系以卡内 Preconditions 文字为准，
  绑边失败不阻塞任何工作。
- 报告一律落本地仓 markdown（docs/reviews/），不以链接交付；每完成一个阶段出一份进度报告并提交。

【边界】
不改 RM-00010；不扩展 hello-wire-v1；公共契约只经架构仓按上述变更顺序更新；
测试档案（123456、Bot01–Bot100、Bot 工具凭证）只按卡内约束使用；凭证/密钥不入库、不进 prompt、不进日志。

另有一条并行线在建 ~/LumioGames/LumioConfig（配表系统），与本编排零交集：
它不碰架构仓任何文件、不占 ADR 号、不写 Workflow、不碰五个实现仓。
你也不需要为它做任何事；若发现它触碰了上述面，停下上报 Owner。
仅一处远期耦合：若你决定重建 Schema/Fixture 体系（见「开工前必须核实」第 4 条），
会与配表的产物容器与指纹设计产生重叠，届时需 Owner 重新对边界——发生时上报，不要自行协调。
```
