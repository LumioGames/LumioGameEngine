# ADR-054：Account Server 第三服务拓扑与账号/准入端口契约

状态：Accepted（2026-09-01，RM-00011 Wave 0 契约冻结卡 R-00357 / C-3；依据 Room Review Rulings 2026-09-01 裁决 #3、#3b、#5）
取代：无

## 背景

RM-00011「ECS Formal Entity and Chat Vertical Slice」要求 100 个 Bot 账号与 1 个浏览器账号经独立账号服务 login-or-register 取得凭证并进入同一房间。需求室审查（`docs/reviews/2026-09-01-rm-00011-room-review.md`）发现四处悬空：账号服无目标仓归属（P0-4，目标仓虚指 "Account Server integration owner"）；第三服务拓扑无 ADR；「Bot 工具上下文」只是一句话、不是可验证的凭证；「Bot 工具凭证 claim 与失败码」从未冻结。Owner 于 2026-09-01 逐题裁决：Account Server 是 LumioServer 仓 `account-server//` 目录下的**正式独立服务**（真实 login-or-register 协议、哈希凭证存储、签名过期不透明准入凭证、持久账号库、真 Bot 工具凭证 claim 六件事）；AccountEntity 走 ECS；重复准入定为顶号（单行为）。本 ADR 把这些裁决连同端口字段、凭证格式与失败码一并冻结。

## 决策

1. **第三服务拓扑**：新增 Account Server——`LumioServer` 仓 `account-server/` 目录下的独立 C# 进程，每部署一个中心实例；持有**持久账号库**（同一 AccountId 跨服务重启稳定，禁止纯内存实现）。Game Server 不创建账号、不接受用户名口令替代 Account Server 准入，只消费离线验证端口。跨服转移与多实例不在本切片。
2. **AccountEntity 走 ECS**：Account Server 自有低频 ECS World；账号身份数据建模为专用组件（`AccountIdentityComponent`：accountId + loginName + createdAt）。login 加载或创建 AccountEntity，logout 只结束会话。**凭证材料绝不进普通组件**：只允许存在于凭证库（静态哈希）或 persist-only 且永不复制的专用声明；绝不返回给客户端或 Game Server。Game 侧集成只携带 AccountId 值，绝不携带 AccountEntity 对象引用跨 World（与 [ADR-004](ADR-004-entity-identity.md) 的身份分层一致：AccountId 是持久业务身份，NetEntityId 是运行时身份）。
3. **login-or-register 端口**（幂等、不覆写、并发收敛、不预置）：用户名不存在→创建（含 AccountEntity）；存在且口令正确→成功返回同一 accountId；存在且口令错误→`wrong_password` 拒绝且**零覆写**；同一不存在用户名的并发首次请求收敛为恰一个 AccountEntity 与同一 AccountId。Bot01–Bot100 由启动器循环生成、经同一端口即时创建，账号库不预灌。口令只进请求，静态 Argon2id 哈希落盘；测试档案默认口令 `123456` 是配置字面量（契约 `passwordProfile.testProfile`），生产口令策略另行决策。
4. **准入凭证**：Account Server 每次成功登录签发**签名、过期、不透明**的 admission credential。canonical payload 按 LumioBinV1（[ADR-047](ADR-047-lumio-bin-canonical-profile.md)，Draft——本契约是其正式消费者），签名按 LumioSignatureV1（[ADR-042](ADR-042-signature-trust-profile.md)，Draft；trustDomain `account-admission` / payloadType `admission-credential-v1`）。Game Server 以部署面分发的公钥（keyId 对应）**离线验证**，不回调 Account Server。nonce 保证签发唯一可审计；在线吊销不在本切片。
5. **Bot 工具凭证 claim**：Bot 启动器以**真凭证**（同样 LumioBinV1 + LumioSignatureV1，trustDomain `bot-tool`，scope `bot-namespace`）认证其注册/登录上下文；签发面在部署/测试档案，不入库。`^Bot[0-9]+$` 命名空间对普通客户端在四个触点全部关闭：register、claim（凭证缺失/无效/过期）、login（含**口令正确**的已存在 Bot 账号——审查 P1-5 补测路径）、admission（凭证 `botToolContext=false`），各有独立失败码。
6. **顶号（单行为）**：同一账号第二条已认证准入踢旧连接（显式终止通知 `TakeoverNotice`：`reasonCode=connection_superseded` + `reconnectEligible` + `issuedAt`），随后经重连重绑机制（R-00350）把同一保留实体重绑给新连接，NetEntityId 不变。通知的**字段形状与语义在本契约冻结**；其在游戏连接消息集内的注册与信封由 R-00355 契约（`lumio.gameplay-envelope.v1`）承载，不另写第二真值。
7. **交付载体**：以上全部字段、失败码、limits、进程边界（readiness/退出码/审计事件词表）与正反用例的唯一真值是 [`engine/wire/account-port-v1.json`](../../engine/wire/account-port-v1.json)（contractId `lumio.account-port.v1`），按 [ADR-052](ADR-052-ms00002-hello-wire-and-clr-host-abi.md) 确立的开发态 wire 契约先例交付；消费方不得另写协议真值。契约措辞宿主无关：C# MVP 宿主先行，切片级最小 Rust 宿主随后复跑同一考卷。

## 替代方案

- **Game Server 内建账号表 / 假账号映射**：Owner 裁决明确否决——Account Server 是正式组件，不做旁路。
- **共享外部身份提供方（OAuth/OIDC 等）**：超出受控切片 profile，边界排除。
- **在线 introspection 验证准入凭证**：引入 Game Server 对 Account Server 的同步运行时依赖；切片选离线验证（过期 + 验签 + 一次性 nonce 已满足考卷），生产撤销机制另行决策。
- **自造 JWT 类凭证格式**：拒绝——凭证编码与签名复用 ADR-047/ADR-042 已冻结 profile，不产生第二套密码学编码真值。

## 接口

- 契约真值：`engine/wire/account-port-v1.json`（唯一字段真值；两操作 `login_or_register` / `verify_admission`、14 个失败码、8 项 limits、进程边界、7 正例 + 11 负例冻结用例）。
- 传输参考绑定：WebSocket 子协议 `lumio-account-v1`（与 `lumio.hello-wire.v1` 同栈、独立消息名空间）；`verify_admission` 是 game-server 进程内端口。
- 消费方：R-00344（LumioServer `account-server/` 独立进程）、R-00346（准入 + 顶号）、R-00349（客户端登录/通知处理）、R-00352/R-00354（Bot 启动器与集成考卷）。

## 失败语义

- 全部失败码与逐码语义冻结于契约 `errorCodeSemantics`（invalid_request/invalid_username/invalid_password/wrong_password、bot_namespace_{register,login,admission}_forbidden、bot_tool_credential_{malformed,invalid,expired}、admission_credential_{malformed,invalid_signature,expired}、takeover_notice_invalid）。
- 硬语义：wrong_password **零覆写**；并发首次登录**必收敛**；verify_admission 验证顺序固定（解形→验签→过期→Bot 上下文）；TakeoverNotice 校验失败按 `takeover_notice_invalid` 显式拒绝，不得静默忽略。
- 凭证材料与口令（含哈希）绝不进响应、审计、日志或普通组件。

## 兼容影响

- 不修改 `engine/wire/hello-wire-v1.json`（子协议与消息名空间独立）、不修改 `engine/abi/native-abi.json`（`node eng/generate-abi.mjs` 保持零差异）。
- 开发态交付：无 Baseline/七仓镜像义务（预上线 Living Architecture；ADR-052 同口径）；下游各仓按契约手写类型并做契约一致性测试。
- 解锁关系：R-00344（实前置本卡）、R-00346/R-00350（顶号语义）、R-00349/R-00354（登录路径与考卷）。R-00218（验票 Port）降为架构参考，不是本端口的替代。

## 迁移方案

- 新能力，无需迁移。keyId 预留密钥轮换位（流程属部署决策）。正式硬化阶段若将本契约升级为版本化公共合同，本 ADR 记录的语义约束（凭证规则、零覆写、并发收敛、顶号单行为、四触点关闭）继续有效；生产口令策略与凭证撤销届时另立 ADR。

## 验证

- 本卡：`node .spec/tools/spec-lint.mjs`（本 ADR 未登记 decisions/README.md 索引是**预期**单条报错——登记由主 loop 串行合并时统一处理，其余项通过）；`node eng/generate-abi.mjs` 零差异；契约 JSON 自检（parse + 必含节 + 用例引用的失败码全部存在于 errorCodes + 正负用例齐备）通过；合并时由 `eng/verify-wire.mjs`（R-00355/C-1 交付）纳入统一校验。
- 下游验收钩子：R-00344「账号服重启后 Bot01 重登返回同一 AccountId」；「普通客户端用默认口令登录已存在 Bot 账号被拒」（P1-5）；R-00346 顶号踢线 + NetEntityId 不变；R-00354 集成考卷按 `testCases`/`invalidCases` 逐条取证。
