# UE 术语 → 通用表述
| UE 术语 | 通用表述 | 备注 |
|---|---|---|
| Actor | 网络可识别 gameplay object | 目标环境更可能是 Entity/Chunk/Object |
| ActorChannel | 对象生命周期+状态消息流 | 不必真的“一对象一通道” |
| NetDriver | transport/connection adapter + network runtime入口 | 可拆成更细端口 |
| NetConnection | peer transport connection state | 不等于 session identity |
| Bunch | 通道级序列化消息片段/批次 | 具体 framing 可替换 |
| Replicated Property | latest-state field | 不等于事件 |
| RPC | remote event/command invocation | 要区分幂等与可靠性 |
| Relevancy | per-connection eligibility | 不等于 AOI Enter/Leave |
| Dormancy | 跳过高频复制扫描的静态优化 | 不等于 unload |
| NetPriority | bandwidth scheduling weight | 应叠加 aging |
| ReplicationGraph | persistent interest candidate index | 可映射空间索引/分类索引 |
| Iris Replication State | 网络中间状态副本 | 与 canonical network snapshot 类似 |
| AutonomousProxy | 本地输入驱动预测副本 | 不是 authority |
| SimulatedProxy | 远端插值/外推副本 | 只消费权威更新 |
| SeamlessTravel | 保持连接的 world transition | 不是 release hot update |
| DemoNetDriver | 复用网络管线的观察回放 | 不等于 authoritative WAL |
