---
name: 2026-09-05-voxelengine-w0-card-and-kickoff
description: VoxelEngine 补单草稿——W0「退出旧合同制」卡面（待授权建卡）、开工提示词与 15 张蓝图卡的验收跑批提示词；派 VoxelEngine 清理或验收活时查
metadata:
  type: doc
  status: 设计中
---

# VoxelEngine · W0 清理卡卡面、开工提示词与验收跑批提示词

> 来源：[`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.2 与 §6 D7 ~ D11（**尚未裁决**，本文按建议方向预写；Owner 改口则同步改）。卡面按 workflow-ops `card-spec`（背景 / 目标 / 验收 / 边界）。**未落单**：建卡须 Owner 逐次授权。与 NativeCore 一站的 [`2026-09-05-nativecore-w0-card-and-kickoff.md`](2026-09-05-nativecore-w0-card-and-kickoff.md) 同一格式。

## 一、卡面（RM-00003 · 已建：R-00474）

**标题**：[VoxelEngine·W0] 退出旧合同制：删复印件与 CI 校对、删 generated 树与 legacy_baseline、活代码不再用 V1.4 的类型与错误 id

**优先级 / 风险**：P0 / low。**Category**：技术需求。**Module**：LumioVoxelEngine。

### 背景

架构仓已按 ADR-059 / `architecture.md` §6–§7 转入 Living Architecture：Baseline、`schemas/`、`tools/lumio_contract.py`、生成源仓 `LumioGameEngineArchitecture` 全部不存在；体素公共语义唯一真值是 `engine/wire/voxel-world-v1.json`（ADR-062 / ADR-066），本仓已按 ADR 0013 消费它（`wire/` 副本 + `CONTRACT_SHA256` + 一致性测试）。但仓里另一套东西还在：README 与 CI 校对 `LGE-V1.4-2026-08-27` 基线，`docs/architecture/` 六版镜像，`crates/lumio-voxel-contracts/generated/` 420 KB 只读生成物（`lumio_core.h`、`RootAbi.cs`、C# 生成目录）由 `generated-lock.json` + `check-generated-clean` 锁住；活代码 33 个源文件用 `Generated*` 类型、21 个用 `STABLE_ERROR_IDS`、8 个用 `BASELINE_ID` / `SCHEMA_EPOCH`、6 个用 VOX-D 门常量；架构仓 `sdk-native/voxel.rs` 也在引用它们。错误 id 因此有两套命名空间（契约 snake_case 与镜像 `STABLE_ERROR_IDS`），这正是「第二份真值」。

### 目标

本仓只剩「消费活契约的纯 Rust 体素 crate」这一种形状：公共语义只从 `wire/voxel-world-v1.json` 取，错误 id 只有契约那一套，没有任何指向 `LGE-V1.x` / `LumioGameEngineArchitecture` / Root ABI bundle / CoreEngine 的文件、常量、CI 步骤或文档口径。

### 验收（逐条可机器判定）

1. `git grep -l -i -e 'LGE-V1' -e 'LumioGameEngineArchitecture' -e 'Root ABI' -e 'CoreEngine' -e 'lumio_contract.py'` 在已跟踪文件里只命中 `.spec/decisions/` 下的历史 ADR 正文（0007 / 0009 / 0013 等只加「被 NNNN 取代」，不改写）。
2. `docs/architecture/`、`docs/LumioVoxelEngine_Framework_Design_LGE-V1.3/`、`docs/plans/lve-v1.4-implementation-blueprint.md`、`docs/evidence/decision-gates/`、`docs/evidence/v1.4-generated-artifact-gate.md` 不存在；`.github/workflows/repository-policy.yml` 不再 grep 基线字符串、不再 `sha256sum -c`、不再跑 `check-generated-clean`。
3. `crates/lumio-voxel-contracts/generated/`、`tools/architecture/generated-lock.json`、`crates/lumio-voxel-test-support/src/generated_clean.rs` 与 `examples/check-generated-clean.rs`、`crates/lumio-voxel-contracts/src/legacy_baseline.rs` 不存在；`lumio-voxel-contracts` 只剩 `voxel_world` 模块（外加它自己需要的水管，如 SHA-256）。
4. `git grep -l -e 'Generated[A-Z]' -e 'BASELINE_ID' -e 'SCHEMA_EPOCH' -e 'STABLE_ERROR_IDS' -e 'is_stable_error_id' -e 'P0_DECISION_GATES' -e 'from_generated' -- 'crates/*/src'` 命中 0 个文件；错误 id 的唯一判定谓词只认契约 `errorCodes`。
5. `modules/README.md` 与各模块 README 按 `voxel.md` M1 ~ M10 模块图重写（只保留有代码的模块；`mesh-collision` / `migration` / `spatial` / `streaming` 目录删除）；`lumio-voxel-migration` 空壳 crate 删除，workspace members 与 `check-crate-dag` 同步。
6. README「架构基线」「Architecture Gate」「Generated Contract Dependencies」三节删除，改为一节「公共契约来源」指向架构仓 `engine/wire/voxel-world-v1.json` 与 `.spec/knowledge/features/voxel-section-chunk.md`；`.spec/AGENTS.md` 收口门槛改为本仓实际命令（fmt / clippy / check / test / check-crate-dag / spec-lint）。
7. `crates/lumio-voxel-test-support/src/{b0_harness,b2_harness,mvp_harness,reference_harness,fixture_runner}.rs` 里只依赖 V1.4 fixture / 决策门的部分删除；保留的 harness 不再引用任何被删类型。
8. `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo check --workspace --no-default-features`、`cargo test --workspace --all-features`（`LUMIO_ENGINE_WIRE_DIR` 指向架构仓 `engine/wire` 时上游比对也通过）、`cargo run -p lumio-voxel-test-support --example check-crate-dag`、`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs` 全部 exit 0；测试数不少于清理前的 392 减去随被删 harness 一起删除的用例数（交付时列出被删用例清单）。
9. 架构仓只读执行 `cargo build -p lumio-engine-native && cargo test -p lumio-engine-native`：若因本卡删除的类型编不过，停下标 BLOCKED 上报，由配套卡 A-W0 在架构仓改 `sdk-native/voxel.rs`；本卡不改架构仓任何文件。
10. 本仓 `.spec/decisions/` 新增一条 ADR 记录退出旧合同制（编号现查最高号），0007 / 0009 / 0010 / 0013 只加「被 NNNN 取代」行。

### 边界

- 不加功能、不改 `wire/voxel-world-v1.json` 语义、不动 `block.rs` / `block_storage.rs` / `block_read.rs` / `physics_query.rs` / `residency.rs` / `binding.rs` 的公开 API 与行为。
- 不恢复任何 Schema / Fixture / 镜像工具链；不写「先兼容、以后再清」的中间态，不留别名、不留兜底常量。
- 不改架构仓与任何上层仓文件；不在提交里夹带 `.DS_Store`、`target/` 或本任务之外的文件。

## 二、开工提示词（另开一个 Claude Code 窗口，工作目录 `~/LumioGames/LumioVoxelEngine`，整段粘贴）

```text
你是 LumioVoxelEngine 仓的体素内核工程师。这一轮只做一张卡：Workflow（lumiogamesengine）R-00474
「[VoxelEngine·W0] 退出旧合同制：删复印件与 CI 校对、删 generated 树与 legacy_baseline、活代码不再用 V1.4 的类型与错误 id」
https://lumiogamesengine.workflow.games/requirements/01a0724d-35ce-7b54-a285-e17ce638a901

【守门（第一步，任一不符立即停下回报 BLOCKED，不得继续）】
1. 本仓 origin/main 是 e5c056e 或其后继；工作区干净；.claude/worktrees/ 下没有别的会话在跑（有就先问）。
2. 架构仓 ~/LumioGames/LumioGameEngine 的 origin/main 是 4d6d2c3 或其后继，本地 main 与 origin 同步；engine/wire/voxel-world-v1.json 的
   SHA-256 与本仓 crates/lumio-voxel-contracts/wire/voxel-world-v1.json 相同（不同 = 契约又动了，先停）。
3. 架构仓 engine/native/modules/sdk-native/Cargo.toml 仍以路径依赖引用本仓 lumio-voxel-world / domain / ops / contracts 四个 crate
   （这是本仓唯一消费者的形状）。
4. 用 workflow-execute 读全 R-00474：正文 + 验收项 + 评论；读不到就停。

【指路】
- 卡面正文就是任务书：三层清理按验收 1 → 10 的顺序做。第 1 层是文档 / CI / 镜像 / 旧模块图 / 空壳 crate；第 2 层删 generated/ 树、
  generated-lock.json、check-generated-clean、legacy_baseline.rs、VOX-D 决策门与 V1.4 fixture 骨架；第 3 层让活代码不再用 Generated* 类型、
  BASELINE_ID / SCHEMA_EPOCH / STABLE_ERROR_IDS / P0_DECISION_GATES / from_generated，错误 id 只剩契约 snake_case 一套。
- 背景一句话：架构仓已按 ADR-059 废止「Baseline 合同 + 复印 + 校对」制度，体素公共语义唯一真值是 engine/wire/voxel-world-v1.json
  （ADR-062 / ADR-066），本仓 ADR 0013 已经在消费它；这张卡是把另一套东西彻底拆掉。设计现状与裁决见架构仓
  .spec/reviews/2026-09-05-engine-repos-progress-assessment.md §2.2 与 §6 D8，只读。
- 被删的「水管」（SHA-256、有界缓冲）如果活代码还要用，在本仓 crate 内自有实现或换标准依赖，不许保留 generated/ 里的那份。
- 第 3 层碰到架构仓 sdk-native/voxel.rs 编不过（验收 9），停下标 BLOCKED 上报，不改架构仓任何文件；配套卡 A-W0 在架构仓做。
- 本仓 .spec/decisions/ 新增一条 ADR 记录这次退出（编号现查最高号），0007 / 0009 / 0010 / 0013 只加「被 NNNN 取代」，不改写。

【立规】
- 领卡先经 Workflow 流转「实现中」并写 reason；改动在 feat/r-00xxx-exit-legacy-contract 分支，先 push 再回写证据。
- 小步提交，每层一个提交；每次提交前 cargo fmt --all -- --check、cargo clippy --workspace --all-targets --all-features -- -D warnings、
  cargo check --workspace --no-default-features、cargo test --workspace --all-features（LUMIO_ENGINE_WIRE_DIR 指向架构仓 engine/wire）、
  cargo run -p lumio-voxel-test-support --example check-crate-dag、node .spec/tools/spec-lint.mjs 全部 exit 0。
  测试证据必须是本机实跑的命令与输出，cargo check 不算；被删用例逐条列清单。
- 交付 = 改动清单 + 验证证据（命令 + 关键输出）+ known gaps + 沉淀落点（本仓新 ADR），写成 PR 描述并同步为 R-00xxx 的证据评论，
  评论只引用已推送 origin 的提交号；做完流转「验收中」，「已完成」由总调度核验后流转。走 PR，不直接推 main。
- 遇到 bug 或测试失败先找根因再改；同一问题修三次不成，停下上报。

【禁区】
- 不加任何功能；不改 wire/voxel-world-v1.json 一个字节；不动 block / block_storage / block_read / physics_query / residency / binding 的
  公开 API 与行为；不重排 crate 依赖方向。
- 不改架构仓与任何上层仓的文件；不恢复任何已删的 Schema / Fixture / 镜像工具链。
- 不写「先兼容、以后再清」的中间态，不留别名、不留兜底常量；历史 ADR 正文不改写。
- 不在提交里夹带 .DS_Store、target/ 或本任务之外的文件。
```

## 三、验收跑批提示词（V-QA · 15 张蓝图卡的 93 条验收项，另开窗口；D9 授权并完成 V-SYNC 回写后再派）

```text
你是 LumioGamesEngine 的独立 QA（写 ≠ 判）。任务：对蓝图 voxel-impl-2026-09-04 的 15 张卡逐条实跑验收项并回写，不改任何代码。
卡号：R-00434 / 435 / 436 / 437 / 438 / 440 / 447 / 448 / 452 / 458（LumioVoxelEngine，66 条）；R-00439 / 443 / 445 / 456（LumioGameEngine，27 条）；
R-00441 已 6/6 passed，不重跑。

【守门】
1. LumioVoxelEngine origin/main = e5c056e 或其后继；LumioGameEngine origin/main = 4d6d2c3 或其后继；两仓本地 main 与 origin 同步。
2. 两仓各建一个只读快照（git archive 物化到 ~/LumioGames/.qa-<仓>-<短号>/），验证全部跑在快照里，不在主工作区跑构建（另一会话可能在用）。
3. 用 workflow-execute 四路读全每张卡（正文 + 验收项 + 评论 + 附件）；任一张状态不是「验收中」就停下回报，不自行流转。
4. GET /projects/<projectId>/acceptance/types 现查验收状态 id，不猜。

【怎么跑】
- 每条验收项 = 一条可执行断言：能用 cargo test / node --test / dotnet test 定位到的就跑那条测试并贴输出；要 grep 代码或契约的就贴 grep；
  跑不出来的标 blocked 并写明缺什么（例如 R-00440 验收 2「超出驻留预算的 pin 当场失败」在契约无预算常量时只能 blocked，不得凭本地上限判 passed）。
- 通过的验收项逐条改 passed，失败的改 failed 并附输出；每张卡一条汇总评论（实跑命令 + 通过 / 失败 / blocked 计数 + 快照对应的 origin 提交号）。
- 全部 passed 的卡流转「已完成」并写 reason；有 failed 的卡不流转，回报主 loop。

【禁区】
- 不改两仓任何文件、不 push、不建卡；不替 Owner 决定契约缺陷怎么修；不把 blocked 写成 passed。
```
