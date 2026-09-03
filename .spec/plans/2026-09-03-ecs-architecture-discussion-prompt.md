---
name: 2026-09-03-ecs-architecture-discussion-prompt
description: ECS 架构专题讨论的开工提示词——把 Owner 口述、已拍板项、待定项与文档回写义务整段交给新会话;启动 ECS 定稿会话时整段粘贴
metadata:
  type: doc
  status: 设计中
---

# ECS 架构专题讨论 · 开工提示词

> 用法：把「提示词正文」整段作为架构仓新会话的第一条输入。该会话只做**设计讨论与文档回写**，不写实现代码、不派 worker、不碰 Workflow。

## 提示词正文

你是 LumioGameEngine 架构仓的主会话，和 Owner 一起把 ECS 这套框架重新梳理清楚并定稿。背景：RM-00011 r3 交付被退回，最重的一条是 Runtime 内部开了三个世界、查询读的是私有字典里的常量，「单一 ECS」没有连起来；根因是「世界」本身没有 owner、没有定义。这次讨论的产出直接决定 r4 的 Runtime 卡（R4-05）与 Client 卡（R4-04）怎么写。

### 0. 讨论方式（Owner 明确要求，必须遵守）

- 一条一条过。每次只提一个问题，用 AskUserQuestion 弹选择框；问题正文里先用**大白话**讲清「是什么、问题在哪、原因是什么」，再给一个**游戏开发里的例子**，然后才是选项。不要堆术语，不要一次抛多个问题。
- 以**游戏开发引擎框架**的角度思考，不只是把 Owner 的口述抄下来；Owner 说「你最好以游戏引擎框架去考虑问题，先和我深刻讨论再定」。
- 遇到 Owner 说「先记个 TODO」就记下继续，不纠缠。
- 第一性原理：如无必要勿增实体。任何多一份实现、多一个特殊实体、多一条写路径的提议，先证明必要。
- 讨论结束前，把结论**回写到文档**（见第 4 节），Owner 说「最后的讨论结果也要更新到文档中」。

### 1. 开工前必读

1. `.spec/reviews/2026-09-03-rm-00011-r4-owner-discussion.md` 第 3 节——Owner 原话、主会话给出的「用户名」全链路、已拍板三条、待定项。这是本次讨论的起点，**不要重新问已拍板的三条**。
2. `.spec/knowledge/features/ecs.md`——现有 ECS 设计（M1 World 与身份、M2 EntityType、M3 结构事务与生命周期、M4 属性同步、M5 跨实体引用、M9 绑定与查询）。讨论中凡与它冲突之处都要指出并让 Owner 定。
3. `.spec/decisions/ADR-057-rm00011-r4-owner-rulings.md` 第 7 条与「兼容影响」。
4. `.spec/reviews/2026-09-03-rm-00011-r3-owner-review.md` P1-7 / P1-9 / P2-6 / P2-12——Runtime 现状事实（三个世界、`_values` 字典、标注不驱动注册、无界事件历史、Game 第二个 ChatComponent、NetEntityId 是 64 位计数器）。
5. Runtime 现有零件（只读，路径相对 LumioGameRuntime）：`modules/ecs/src/Lumio.GameRuntime.Ecs/EcsModule.cs`（CreateWorld）、`Storage/ComponentTypeRegistry.cs`（EntityTypeDefinition / ComponentTypeDefinition）、`Annotations/`（标注类型与扫描器）、`Ingress/ChatIngressWorld.cs`（手写注册的现状）、`modules/replication/src/.../Binding/EntityBindingQuery.cs`（两个空世界 + 字典）、`Chat/ChatCommandRuntime.cs`（第三个世界 + 命令路径）、`Snapshot/EcsPersistSnapshotPipeline.cs`。

### 2. Owner 已定（不再讨论，直接作为前提）

- 只有一个 GameWorld；进入 Game 时由 ECS World Manager 创建并管理；Entity 有类型，类型决定组件集；组件齐了走生命周期。
- 客户端也是一份 World（不叫 ReplicaWorld）；服务器创建 Entity → RPC → 客户端用同一套 ECS 创建；字段同步由标注决定。
- 存档触发 = 单例 WorldEntity + 组件（WorldSaveComponent 等）；存档是给 WorldEntity 下一条命令，提交相里的存档系统消费。
- 读权限 = 同步时按 Visibility 裁；客户端本地读不再判。
- 身份表入档 = NetEntityId 表是世界快照的一部分，恢复后 id 不变、永不复用。
- 私有字段允许，含义是「别人读不到」。
- 多房间后置。

### 3. 要逐条过的议题（按此顺序）

1. **「用户名」全链路 ①–⑦**（讨论记录 3.2）逐段确认：声明 → 建世界 → 创建 → 写 → 同步 → 读 → 存档。每段先讲现状零件有/缺什么，再问 Owner 这段的形态。
2. **改写入口**：客户端发起的字段修改是否统一走 InputCommand → 提交相（与聊天同一条路），还是另有 RPC 写路径。
3. **Room 是什么**：一个 Game 实例 = 一个 GameWorld？契约里的 `roomId`、需求 §6.10「第二个 Room 隔离场景」怎么改。主会话之前的提议：「Room 只是 Game 实例的编号，进 Room = 初始化一个 GameWorld，世界内部无 Room 字段，第二个 Room = 第二个 Game 实例」——Owner 未确认。
4. **标注 → 注册桥**：组件注册表、同步表、契约声明表三张表如何从同一份标注生成；EntityType 声明怎么写（组件集、出生自带子实体）；非法组合在哪一步拒。
5. **World Manager 的职责边界**：谁持有世界句柄；绑定、查询、聊天、存档如何拿到它；线程归属怎么判（不再靠空世界当牌子）。
6. **「私有字段」措辞**：主会话提议「实体字段只能是带标注的组件字段；模块可以有私有状态，但不能用它替代组件存实体数据」——Owner 未确认。
7. **NetEntityId 形制**：ecs.md 说 128 位不透明永不复用，现状是 64 位计数器补零；要不要改，谁发。
8. **客户端 World 与现有 Client 仓 ReplicaWorld 的关系**：现 ReplicaWorld 是字典 + 字符串属性袋、零 Runtime 引用；重做时 Client 是否直接引用 Runtime ECS 程序集。
9. **事件与历史**：chat.event 这类「消息通道」与属性同步的边界；服务器不保留事件历史（现状有无界 `_eventsByRoomTick`）。

### 4. 讨论结束时必须回写的文档

1. `.spec/knowledge/features/ecs.md`：按定稿改 M1（「每 Room 一份」→「World Manager 每 Game 实例一个 GameWorld」）、M4/M9 相关措辞、「ReplicaWorld」→「客户端 World」、新增「World Manager 与 WorldEntity」段。设计现状口吻，不写讨论过程。
2. `.spec/decisions/ADR-058-ecs-world-manager-and-annotation-registry.md`（Draft）：记录本次全部裁决与被否决的替代方案；`decisions/README.md` 加索引行；`docs/adr/` 加软链。
3. `.spec/plans/2026-09-03-rm-00011-r4-blueprint.md`：把 R4-05（Runtime 单一世界）与 R4-04（Client）从「待 ECS 定稿」改为可写，填入拥有范围、前置、验收 4 条。
4. `.spec/reviews/2026-09-03-rm-00011-r4-owner-discussion.md`：第 3.4「待新对话讨论」逐项标记结论（或改为「已定，见 ADR-058」）。
5. `node .spec/tools/spec-lint.mjs` 通过。改完向 Owner 交代改动清单与 lint 输出。

### 5. 不得做的事

- 不写实现代码，不改实现仓；不派 worker；不碰 Workflow。
- 不重开已拍板的三条；不把 Owner 的口述直接当结论——先按引擎框架推敲再问。
- 不一次问多个问题；不省略大白话与游戏例子。
