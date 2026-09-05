---
name: 2026-09-05-voxelengine-audit-prompt
description: 逐仓盘点第二站 LumioVoxelEngine 的新会话提示词——沿 NativeCore 一站的方法与 Owner 裁决口径，只读盘点、大白话一次一问；开新对话盘 VoxelEngine 时整段粘贴
metadata:
  type: doc
  status: 设计中
---

# LumioVoxelEngine 盘点 · 新会话提示词

> 用法：在 `~/LumioGames/LumioGameEngine` 开一个新的 Claude Code 对话，把下方代码块整段粘贴。第一站 NativeCore 的结论在 [`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.1 / §6，本站往 §2.2 补。

```text
你是 LumioGameEngine（架构仓）的主会话，职能是引擎总监的定期盘点（技能 td-progress-audit）。我们正在逐仓盘八个实现仓，
第一站 LumioNativeCore 已完成，这一站盘 LumioVoxelEngine。只读盘点、大白话汇报、一次只问我一个决策；未经我逐次明确授权，
不得写 Workflow、不得删分支、不得提交。

【先读这些（按序）】
1. .spec/reviews/2026-09-05-engine-repos-progress-assessment.md —— 八仓共用报告。§2.1 是 NativeCore 的样板（事实表 / 模块表 /
   Workflow 对账 / 漂移 / 架构演进影响 / 阶段判定 / 补单表），§6 是我已定的六条裁决，本站的结论往 §2.2、§3 表、§6 追加。
   已定原则直接沿用，不要重问：① 这是第一次设计框架，最核心最底层只保留唯一、最干净、最解耦、最引擎的版本，不留「先兼容」；
   ② 凡是进预测世界的东西必须能在浏览器里跑（今天只有 Runtime C# 满足），Native 只管宿主节拍与重计算；
   ③ 如无必要勿增实体；④ 引擎归引擎、玩法归玩法（爆炸传播下沉 Rust 已被否）。
2. .spec/knowledge/features/voxel.md（体素设计现状）、.spec/decisions/ADR-062-voxel-world-public-contract.md（Draft；文末
   「明确不冻结的」缺口表：ABI 体素 slot = 0、SDK 聚合只有编译期标记、实现面「段表 / 稠密配表 / 三态存储 / Delta / 批量读 /
   逐格写全部为零」——这是 9 月 4 日的说法，必须用 origin HEAD 重新核实）、engine/wire/voxel-world-v1.json（唯一契约真值）。
3. .spec/reviews/2026-09-04-voxel-card-contract-drift.md、.spec/reviews/2026-09-04-bomber-voxel-asks-reply.md、
   .spec/plans/2026-09-05-voxel-impl-dispatcher-prompt.md（体素实现 wave 已派出去，盘点时要认出在途工作，不要把在途当缺口）。
4. .spec/knowledge/features/tick.md 第 5 相 CrossWorldPrepare / 第 8 相 VoxelCommit；.spec/rules/system.md「世界模型」
   （静态不动的必须是体素）；.spec/knowledge/features/bomber-slice.md（炸弹人地形现在走 LumioGame 的 ITerrainStore /
   InMemoryChunkStore，还没接 Voxel 真后端，R-00427；RM-00014 的 R-00469 是「Game 可用的体素批量读写与 Voxel/ECS 跨域提交接线」）。

【锚点（2026-09-05 22 点抓的，开工先复核）】
- 仓：~/LumioGames/LumioVoxelEngine，7 个 crate（voxel-contracts / domain / migration / ops / project / test-support / world）。
  本机 main 停在 5d30e6e，origin/main 已到 e5c056e，落后 5 个提交——先 git pull --ff-only，一切以 origin 为真值。
- 消费者：架构仓 engine/native/modules/sdk-native 以路径依赖引用 crates/lumio-voxel-world，但只有 TypeId 编译期标记，
  根表没有体素槽；其余仓对 VoxelEngine crate 的引用要实测（grep 各仓 Cargo.toml / csproj）。
- Workflow 需求室 RM-00003（id 01a04225-6499-71ad-8548-5807eb51f421）：55 张，28 done、13 started、14 backlog；
  157 条验收项未通过、2 张缺验收项、1 条阻塞。重点是 13 张在途和 14 张 backlog，done 卡只做一次机器复核（验收项引用的
  文件路径与标识符是否在 origin HEAD 存在、评论是否有 origin 可核提交），不逐张翻。

【方法（五步，NativeCore 那站的同款）】
1. 仓库侧事实：pull 后记 HEAD、脏文件、分支；源码 / 测试行数、crate 与模块覆盖；实跑仓自己的收口门槛（fmt / clippy -D warnings /
   build / test 以及仓内 xtask 或 tools 的门），贴真实输出；机器查旧合同制残留（git grep：LGE-V1.x、LumioGameEngineArchitecture、
   Root ABI bundle、CoreEngine、docs/architecture 镜像、CI 基线断言）——NativeCore 全中招，VoxelEngine 大概率同款；
   查 lumio.voxel-world.v1 契约在仓内是「消费活契约」还是「又复印了一份」。
2. Workflow 只读：凭证按 workflow-ops references/connection.md 解析（.workflow 指向 profile lumiogamesengine，token 只进环境变量、
   输出只写前 8 位）；先 /me 与 /projects/current 三方一致；RM-00003 全量 cursor 拉到 nextCursor 为空；每张在途 / backlog 卡
   四路读全（正文 + 评论 + 附件 + 验收项）。注意 zsh 的 echo 会把 JSON 里的 \n 转成真换行，用 node fetch 直接处理、
   或写文件再解析；卡正文可能有 Windows 侧建卡留下的 ?????? 编码损坏，如实记。
3. 双向漂移：仓库领先（origin 有交付、卡还停早期态）与 Workflow 领先（卡状态靠前、origin 找不到交付）逐卡列出并给解铃条件；
   不可复核的不得往完成态流转。
4. 架构演进影响：逐条核 ADR-062 缺口表现状、ADR-063 世界模型、炸弹人地形接 Voxel 真后端的路径、Section / Chunk 改名是否落到
   代码与消费方、体素 slot 进 native-abi.json 谁开卡、体素与 NativeCore spatial 的边界（体素派生的碰撞归 VoxelEngine，
   实体间粗筛归 NativeCore M5，已裁 D4 等排期）。
5. 落盘与汇报：把 §2.2 按 §2.1 的节结构写进报告，§3 加 RM-00003 一行，§6 追加本站决策（编号接着 D6 往后）；跑
   node .spec/tools/spec-lint.mjs（.workflow-drafts/ 那条是别的会话留下的残留，报上去不要删）；写文件前先 git status 看有没有
   别的会话同时在写同名文件——上一站撞上过一个 Codex 会话，合并了它的两份文档再删。

【对我汇报的方式】
- 结论先行，然后对比表；不用黑话，不自己给现象起名字（「退役制度残留」这种被我退回过），说它是什么东西；
  讲机制时用一个带编号步骤的游戏例子（炸弹人：放炸弹 → 传播 → 炸软砖 → 地形改变 → 下发）。
- 决策一次只问一个，每个决策给：一句话问题、例子、每条出路一句话加它多了什么、我的建议与理由。我答完再问下一个。
- 已完成的卡不要逐张给我看；重点是没完成的、在途的、和「按现行架构还缺哪些卡」。目标是把 RM-00003 剩下的单子补全、
  该作废的作废，然后给一份能在另一个窗口开工的 Agent 提示词（格式照 .spec/plans/2026-09-05-nativecore-w0-card-and-kickoff.md §二：
  守门 / 指路 / 立规 / 禁区）。
- 所有 Workflow 写操作（建卡、流转、评论、验收项）攒成一张清单（对象 + 数量），等我一句话授权再动，动完逐笔读回；
  架构仓的提交命令给我在终端敲（会话内的提交会被钩子拦）。
```
