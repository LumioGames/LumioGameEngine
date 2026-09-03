---
name: 2026-09-03-ecs-sample-owner-rulings
description: Owner 复审 ECS 样板示例的九条问题与裁决流水——Kind、TypeOf、EntityType 继承、归属标注、变化钩子、客户端建世界、WorldEntity;回溯 ADR-058 第二轮修订时查
metadata:
  type: doc
  status: 已交付
---

# ECS 样板示例复审 · Owner 裁决流水（2026-09-03）

> 这是裁决流水。正式条款见 [ADR-058](../decisions/ADR-058-ecs-world-manager-and-annotation-registry.md)「修订（2026-09-03 第二轮）」段与各条，设计现状见 [ecs.md](../knowledge/features/ecs.md) §4.5，样例代码在 LumioGameRuntime `modules/ecs/samples/username/`（分支 `docs/ecs-username-sample-r2`）。

## 起因

Owner 复审 `modules/ecs/samples/username/`，认为样例没把「建世界 → 建 WorldEntity → 建 PlayerEntity → 同步 → 读取」串起来。最小 Demo 定为：建一个世界 → 世界上建一个 PlayerEntity → 实体有 Identity + Chat 两个组件 → Chat 取到自己实体的名字、发消息，消息 = 名字 + 内容 → 两端 log 验证。讨论方式：先收集全部问题，再一条一条过。

## 裁决

| # | 问题 | 裁决 | 被否的路 |
|---|---|---|---|
| Q1 | `IdentityComponent.Kind`（`Sync<EntityKind>`）有没有意义 | 删掉不补。EntityType 已决定类型，`Kind` 是同一件事的第二份；文档「entity kind」统一改「EntityType」 | 删掉后换一个只为演示存在的服务器写 Sync 字段 |
| Q8 | 怎么按 Entity ID 取类型，ID 设计要不要有讲究 | 加 `world.TypeOf(id)`；NetEntityId 保持「实例 ID + 计数器」 | 类型编进 ID（编号须跨版本稳定，热重载改类型集时旧 ID 说谎） |
| Q9 | EntityType 有继承关系怎么办 | 用 C# 继承：声明类 `static class` → `abstract class`，子类型 `: PlayerEntity`；组件集 = 基类 ∪ 自己；`TypeOf(id).Is<PlayerEntity>()` 对子类型为 true | `[Extends]` 标注（多一个词、两套表达）；只用组合 |
| Q2 | Chat 怎么拿到自己的名字、消息 = 名字 + 内容 | 两端都演示同实体跨组件读 `Get<IdentityComponent>().Name`：先 log 名字再发 / 写；服务器 `SendMessage` 体内读 Name，`OnChatMessage(name, text)` 带名字下发 | — （Owner：两端都行，重点是演示取自己的名字） |
| Q3 | 客户端 `Window` 是不是消息历史存档 | 不是，是会话内显示缓冲；从样例删掉，`OnChatMessage` 客户端体只打 log，窗口归 UI 层 | 挂 Self 实体（每实体白分配空 List）；挂 WorldEntity 客户端组件 |
| Q4 | `[Persist] LastMessageText` 在共享文件里，客户端上是什么状态 | 死字段（永远默认值，读到假值不报错）。挪到 `ChatComponent.Server.cs`；**`[ServerOnly]` / `[ClientOnly]` 标注删除，文件后缀是唯一归属声明** | 保留标注做双重校验 |
| Q5 | 客户端自己改名的完整流程；属性变化怎么得知 | 改名：owner 写 `.Value` 本地生效并自动上行 → `OnClientWrite` 校验 → 通过写入下发 / 被拒推回。**变化回调必须有**：生成器为每个 Sync 字段产可选 partial 钩子 `OnXChanging / OnXChanged(old, new, reason)`（容器 `ListChange` / `DictChange`）；**默认只收对端来的变化**（`Sync` / `Correction`），自己改自己不收，字段声明第三参数 `Notify.All` 可选打开（`Local`）；改前只通知不否决；首次填值不触发；整包先写入再统一触发。**WhenAll 式多字段组合器本轮不加，写进设计文档为后置** | 不要回调靠读字段；单入口 `OnSyncChanged(in SyncChange)`（n 个分支、容器 payload 越做越厚）；每字段 `+=` 事件（struct 上随拷贝丢失）；本端写默认触发（Owner：最傻的行为） |
| Q5′ | 客户端服务器同进程（单机 / 本地联调）回调怎么走 | 两个 Manager + 内存环回代替网络，语义与联网零差异；ecs.md 新增「同进程双端」定义 | 共用一个 World（要第三种编译配置，partial 两端体相撞） |
| Q6 | 客户端进游戏是否也建 WorldManager、怎么建 | 是。同一个 `WorldManager.Create(GeneratedRegistry.Instance)`，客户端不传 `instanceId`；欢迎消息经 `Enqueue` 进来绑 `World.Self`；第一条创建记录是 WorldEntity；同进程双端下 `server.outbox → client.Enqueue` 同一行 | 客户端专用 `CreateClient(registry, instanceId, selfId)` |
| Q7 | `Single<WorldSaveComponent>()` 里的 WorldEntity / 组件哪来的 | 游戏在 `EntityTypes/WorldEntity.cs` 声明，`[EntityType(Mode.CS, World = true)] [Has(typeof(WorldSaveComponent))]`；引擎只提供组件；`World = true` 恰好一个 | 引擎内置固定 WorldEntity（游戏没处放世界级状态） |

## 回写落点

- ADR-058：第 2 / 4 / 5 / 7 / 8 / 12 / 14 / 17 / 18 条、接口 / Schema、失败语义、替代方案、验证 Fixture。
- ecs.md：规范词表、M1a ③④⑥⑨、M2 ①②、M4 ①⑤⑦、M9 ①、§4.5、§5 TODO 0-2 / 0-9 / 0-11、§6、§7。
- ecs-entity-chat.md「entity kind」→「EntityType」；r4 蓝图 R4-05 行与 R4-05 卡片正文。
- Runtime `modules/ecs/samples/username/`：全部源文件与 README 重写，新增 `EntityTypes/WorldEntity.cs`、`Host/ClientBootstrap.Client.cs`。

## 审查与修正（同日快审，reviewer 退回 1 P1 + 6 P2，全部核实后处理）

| 严重度 | 问题 | 处理 |
|---|---|---|
| P1 | `OnChatMessage(name, text)` 需要 C-1 `chat.event` 有名字字段，而契约已冻结、无此字段 | 取不改契约的路：服务器把「名字: 内容」拼进 `text`，签名改回 `OnChatMessage(string line)`。Owner 在 Q2 明确「两端都行，重点是演示取自己的名字」；ADR-058 接口段记为第二轮裁量 |
| P2 | `Commands.Create(PlayerEntity.Type)` 与「声明类无成员 / 不生成隐形成员」矛盾 | 改 `Commands.Create<PlayerEntity>()`，ADR 接口段列出 |
| P2 | `$"{Name}"` 插值走 `ToString()` 不走隐式转换 | log 里显式 `Name.Value` |
| P2 | §4.5 改名示例 old / new 写反 | 改为 `name <旧名> -> ABCD (Sync)` |
| P2 | R4-04 卡仍写 `WorldMode.Client` | 改为同一 `Create` 不传 instanceId |
| P2 | R4-01 卡五元组仍写 `kind` | 改为 `TypeOf` 派生的 entityType |
| P2 | `WorldMessage` 类型名三处不对齐 | ADR 接口段定名 `Enqueue(WorldMessage)`，卡片同步 |
| 方案疑虑 | `partial bool OnClientWrite(in SyncWrite)`：带返回值的 partial 在 C# 必须有实现（CS8795），做不到「不写 = 接受」 | 改 `partial void OnClientWrite(in SyncWrite w, ref bool accept)`；样例、§4.5、ADR 第 6 条、R4-05 卡同步（既有行，本轮显化并修正） |
| 方案疑虑 | `[ServerRpc]` / `[ClientRpc]` 的 partial 声明在另一端无用户实现，缺生成的发送桩 | ADR 第 7 条 / 接口段、ecs.md M2 ④、README「生成三件」补「RPC 发送桩」 |
| P2（复核新增） | 校验卡 `text` 而发出的是 `name: text`，可能超 C-1 `chat.event.text` 的 512 UTF-8 字节 | 改为对拼好的 `line` 按 `Encoding.UTF8.GetByteCount` 卡 512；样例与 §4.5 同步 |

复核结论：放行（reviewer 第二轮）。
