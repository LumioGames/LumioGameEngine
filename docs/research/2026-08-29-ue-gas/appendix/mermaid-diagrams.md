# Mermaid 图表源码

> 主报告中的图在此集中收录，便于文档系统单独渲染。

## GAS 对象图

```mermaid
flowchart TD
  Owner[Owner Actor] --> ASC[Ability System Component]
  Avatar[Avatar Actor] --> ASC
  ASC --> Specs[Granted Ability Specs]
  Specs --> Ability[Gameplay Ability]
  Ability --> Tasks[Ability Tasks]
  Ability --> Spec[Gameplay Effect Spec]
  Spec --> Def[Gameplay Effect Definition]
  Spec --> Context[Effect Context]
  ASC --> Active[Active Effects Container]
  Active --> Instance[Active Effect Instance]
  Instance --> Spec
  Active --> Aggregator[Attribute Aggregators]
  Aggregator --> Attributes[Attribute Sets]
  ASC --> Tags[Owned Tag Counts]
  ASC --> Cues[Gameplay Cues]
  ASC --> Prediction[Prediction Keys]
```

## 预测成功收敛

```mermaid
sequenceDiagram
  participant C as Client
  participant S as Server
  C->>C: Generate key K; optimistic Ability/Effect/Cue
  C->>S: Activation/TargetData + K
  S->>S: Revalidate and commit authority state
  S-->>C: State delta + catch-up K
  C->>C: Dedupe predicted effects; keep authority result
```

## 目标引擎分层

```mermaid
flowchart LR
  ECS[Committed ECS State] --> Commit[Ordered Gameplay Commit]
  Commit --> Owner[Owner-private projection]
  Commit --> Public[Public projection]
  Commit --> Minimal[Minimal projection]
  Owner --> Delta[State delta/baseline]
  Public --> Delta
  Minimal --> Delta
  Commit --> Events[Reliable results / transient presentation]
  Delta --> Staging[Client staging]
  Events --> Staging
  Staging --> Apply[Atomic apply by commit sequence]
  Apply --> Hash[State hash]
  Apply --> Overlay[Prediction overlay]
```
