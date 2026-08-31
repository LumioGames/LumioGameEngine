# 信息源可达性与检索日志

- 检索日期：2026-08-29
- 原则：UE 源码只尝试委托方指定的 `Go1c/UnrealEngine`；不 clone、不 Download ZIP、不切换到 `EpicGames/UnrealEngine` 或其他镜像冒充证据。

## 指定 UE 源码镜像

| 尝试 | 定位 | 结果 | 对报告的影响 |
|---|---|---|---|
| 仓库根页 | `https://github.com/Go1c/UnrealEngine` | 抓取返回 Cache miss，无法读取仓库内容 | 不能确认默认分支、commit 或源码版本 |
| GameplayAbilities 目录 | `.../tree/5.6/Engine/Plugins/Runtime/GameplayAbilities` | ref/目录不可确认 | 不能给源码路径与行号 |
| GitHub 仓内代码搜索 | `repo:Go1c/UnrealEngine UAbilitySystemComponent` | 抓取返回 Cache miss | 不能在线定位函数体 |

**结论**：全部源码级机制整体降为 `Reported`，除非 Epic 官方 API Reference/文档本身明确写出该语义；报告没有伪造任何 `Go1c/UnrealEngine@commit:path#Lx-Ly` permalink。

## Epic 官方资料

可读取：UE 5.6 GAS、Ability、Attribute、Effect、GameplayTag、Lyra、Enhanced Input、Mass、CharacterMovement、FastArray、PredictionKey 等文档/API。部分 API 页面只提供声明/成员说明而无函数体，因此只能验证类型和公开语义，不能验证内部先后顺序。

## 官方样例

- Lyra：官方样例文档可读；本次没有取得样例工程本体，因此没有源码路径/行号。
- ActionRPG：legacy 文档/摘要可读；本次没有取得工程本体。
- Fortnite/Paragon：没有生产源码；只使用 Epic 定位、Lyra 公开模式和明确标注的社区/演讲资料。

## 社区资料

- `tranek/GASDocumentation`：可读，使用明确行号；其主线版本自述约 UE 5.3，所有结论标 `Reported`。
- `tranek/GASShooter`：可读，但版本较旧且仓库自述非生产就绪，仅作流程例子。
- Outriders/Slitterhead 公开演讲与报道：可读到公开说明，适合证明生产使用和工程补层，不用于推断未公开源码细节。
