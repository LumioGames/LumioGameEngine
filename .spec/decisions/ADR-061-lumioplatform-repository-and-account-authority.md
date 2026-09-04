# ADR-061：LumioPlatform 仓库加入与账号权威归属——一库两端口、PostgreSQL 持久真值、launch 端口与 `LumioServer/account-server/` 退役

状态：Draft（2026-09-04，Owner 逐题裁决：账号权威归属、网页端栈、存储、仓名、AccountWorld 去留；本 ADR 随 P0–P5 卡实现验证后转 Accepted）
取代：部分取代 [ADR-054](ADR-054-account-server-topology-and-port.md)——第 1 条「Account Server 是 `LumioServer` 仓 `account-server/` 目录下的独立 C# 进程」的**归属**与第 7 条交付载体中的目录声明；ADR-054 其余条款（login-or-register 语义、AccountEntity 走 ECS、准入凭证格式、Bot 工具凭证、顶号）原文继续有效
Owner：`LumioGameEngine`（裁决与契约真值）、`LumioPlatform`（账号权威、大厅、反馈、后台的唯一实现）、`LumioServer`（离线 `verify_admission` 消费扩展后的绑定声明；`account-server/` 退役）、`LumioClient`（游戏页改经 launch 端口取地址与凭证）、`LumioGame`（集成考卷改起平台进程）

## 治理原则

- 沿用 ADR-056：**第一性原理——如无必要，勿增实体。** 一个账号库、一份口令哈希、一份凭证签发、一份协议真值。
- 沿用 ADR-060：**彻底清理，不留兼容。** 账号服迁移不留过渡态；`LumioServer/account-server/` 在退役条件满足后整目录删除，不保留镜像或别名。
- 新增：**账号域对客户端无关。** 浏览器、Bot 启动器与将来的 Steam / iOS / Android / 真机客户端都只经同一账号端口拿身份与准入凭证；平台不是账号系统的前提，而是它的第一个宿主。

## 背景

Owner 要做对外的「游戏平台」：网页账号系统（邮箱注册、ID、用户名、系统默认头像）、游戏大厅（展示自研浏览器联机游戏、点开即玩）、反馈（bug / 建议 + 飞书 / QQ 群跳转）、运营后台（用户、活跃度、埋点、反馈、登录记录）。

现状：ADR-054 已冻结 Account Server——`LumioServer/account-server/`，ASP.NET Core 10，用户名 / 口令 login-or-register，Argon2id 凭证库，Ed25519 签发 300 秒不透明准入凭证，Game Server 离线验票；JSON 文件持久、只绑环回、WS 子协议 `lumio-account-v1`；无邮箱、头像、HTTP 会话、后台。全组织没有数据库、Docker、邮件与部署配置；浏览器游戏页是无构建静态 ES module，从 URL `?ws=` 取服务器地址（`LumioClient/modules/web/`）。Owner 另一产品 GameFlow 已有 Go + Postgres + React 19 的平台栈可作先例。

若平台自建账号库而账号服保留，同一个人会有两个 AccountId、Bot 命名空间两处设防、后台看不到账号服里的用户——凭证格式「兼容」掩盖不了身份分裂。Owner 2026-09-04 逐题裁决如下。

## 决策

1. **第八实现仓 `LumioPlatform`**（`github.com/LumioGames/LumioPlatform`，PUBLIC，Apache-2.0）。职责：唯一账号权威、游戏目录与大厅、launch 端口与房间分配器接口、静态游戏页托管、反馈、运营后台、埋点、平台数据库。不负责：任何引擎内部语义、Game Server 验票与房间模拟、游戏页协议逻辑、集成验收尺子。架构仓的「七个实现仓」口径改为八个。
2. **账号权威归属**：Account Server 的全部职责与原码（`LumioServer/account-server/src/Lumio.Server.Account`）迁入 `LumioPlatform/src/Lumio.Platform.Account`，WS 协议宿主（`AccountProtocolServer`）迁入 `LumioPlatform/src/Lumio.Platform.App`。平台是 Lumio 唯一账号权威；Game Server 仍不创建账号、不收口令、只消费离线 `verify_admission` 及其六元绑定上下文（公钥经部署面分发，`keyId` 语义不变）。
3. **一库两端口**：同一 Kestrel 进程、同一账号库。端口一 = WS `lumio-account-v1` 挂 `/account` 路径，`login_or_register` 返回六元字段为 unbound sentinel 的 `accountAuthCredential`，仅作账号认证、不可入 Room；端口二 = HTTP `/api/*`，Launch 接受浏览器 Cookie 或 `Authorization: Bearer <accountAuthCredential>`，后者供 Bot、工具和 RM-00011 集成消费者交换 Room-bound admission credential。两种调用都只提交 game slug；allocationContext 完全由服务端 allocator 生成并同步更新所有消费者。
4. **持久真值 PostgreSQL（从第一天）**：`DurableAccountStore` JSON 文件存储删除；账号、口令哈希、邮箱验证、登录记录、游戏目录、反馈、事件、设置、审计全部在一个 PostgreSQL 库；本地 Docker，CI service container。
5. **AccountWorld 保留**（Owner 明确）：ADR-054 §2 的低频 ECS World 与 `AccountIdentityComponent` 继续作为账号域**运行态模型**（登录加载 / 创建 AccountEntity、登出只结束会话），数据库是持久真值；账号的一切写入只经 `AccountRuntime` 一条路径，后台读走数据库只读投影。理由：将来 Steam / iOS / Android / 真机客户端要走同一账号域，不能把账号绑死在网页平台上。
6. **账号身份扩展**：`accountId`（不变，进凭证）、`loginName`（= 用户名，ADR-054 grammar 与 Bot 命名空间规则不变，进凭证）、新增 `email`（HTTP 登录标识，唯一，需验证；Bot 与 `test` profile 账号可空）、`uid`（公开数字 ID，从 100000 起）、`avatarId`（系统默认头像集编号，不支持上传）、`role`（player | admin）、`status`（active | banned）。Admission credential v1 canonical payload 在既有字段后固定追加 `serverAudience`、`gameId`、`gameReleaseId`、`contractId`、`roomId`、`allocationId`；Game Server 必须逐字段比对当前分配上下文。
7. **注册策略 profile**（`PLATFORM_REGISTRATION_PROFILE`）：`test` = WS `login_or_register` 照 ADR-054 对任何合法 loginName 登录即注册（集成考卷、开发）；`production`（默认）= WS 端口只允许 Bot 命名空间 + 有效工具凭证注册，普通新 loginName 拒 `registration_requires_platform`，人类账号只能经 HTTP 邮箱注册；已存在的人类账号仍可在 WS 端口用 loginName + 口令登录取 `accountAuthCredential`。生产不得开 `test`。该失败码加入 `account-port-v1.json` 的 `errorCodes`（本 ADR 授权的唯一 WS 契约扩展）。
8. **launch 端口**：`POST /api/games/{slug}/launch` 接受 Cookie 或 WS account-auth Bearer，返回 `{ wsUrl, subprotocol, serverAudience, gameId, gameReleaseId, contractId, roomId, allocationId, admissionCredential, admissionExpiresAt, accountId, loginName }`；调用方除 path slug 外不得提交分配 claim，凭证不进 URL/日志且每次新签发。服务端 allowlist 按 allocationId 精确绑定 scheme/host/port/path 与六元上下文，禁止 userinfo、query、fragment、redirect 和 wildcard host；公网必须是 allowlisted `wss://`，`ws://` 仅显式 test profile + loopback record。多房间时只换服务端分配器实现，端口不改。
9. **网页端栈**：React 19 + TypeScript + Vite 单页应用，构建产物进 ASP.NET `wwwroot`，大厅 / 反馈 / 后台同一 SPA 按角色路由；DTO 真值在 C#，OpenAPI 文档由宿主 `openapi-export` 子命令导出入库并生成 TS 类型（平台内部决策 `LumioPlatform/.spec/decisions/0001`）。
10. **进程边界**：平台单进程 readiness 行 `PLATFORM_READY {"port","pid","listen","database":"postgresql","accountPort":"/account","contractIds":[...]}` 取代 `ACCOUNT_SERVER_READY`；退出码词表不变（0 / 1 / 2 / 3）；`--store-path` 废止，连接串只经 `PLATFORM_DB_CONNECTION_STRING`。
11. **退役条件与顺序**：平台在 CI 上通过 `account-port-v1.json` 全部当前冻结用例（含 unbound WS credential、allocationContext mismatch 与 `production_profile_plain_register_rejected`），且 RM-00011 集成考卷（R4-09 口径）指向平台 `/account` 端口全绿 → 同一 Gate 内删除 `LumioServer/account-server/` 整目录及其 CI job / README 条目。不设并存期。
12. **拓扑待调研**：v1 部署假设「平台 + Game Server + PostgreSQL 同一台机器、各一容器」是否成立、进程 ↔ 房间 ↔ 容器映射、单机双容器成立规模与拆机信号，由 `plans/2026-09-04-platform-topology-research-prompt.md` 的调研给出结论后另立 ADR；本 ADR 不定拓扑。

## 替代方案

- **平台自建账号库、账号服保留、凭证靠 keyId 兼容**：被否——身份分裂（同一人两个 AccountId、Bot 命名空间两处设防、后台不可见）。
- **平台照 GameFlow 栈（Go + Postgres）并由 Go 重写签发**：被否——引擎组织新增语言，签发实现从一份变两份，公钥分发与契约一致性测试重做。
- **平台只做前端，账号服留在 LumioServer 内扩展邮箱 / 后台 API**：被否——LumioServer 职责膨胀成 Web 后端，且「Server 仓不拥有账号」是 ADR-054 的前提之一（Game Server 只验票）。
- **删 AccountWorld，账号只有数据库一份**：主会话建议，Owner 否决——账号域要服务将来的原生客户端，保留运行态模型；数据库仍是持久真值，不构成两份真值。
- **Razor Pages 服务端渲染 / 前后端分进程**：被否——前者无先例且后台交互全手写，后者多一个部署单元与 CORS。
- **SQLite 起步、信号触发换 Postgres**：主会话建议，Owner 否决——从第一天 PostgreSQL，与 GameFlow 同引擎，多实例与分析查询一次到位。
- **保留 `LumioServer/account-server/` 并存一段时间**：被否——违反「不留兼容」；退役条件满足即删。

## 接口 / Schema

契约镜像基线：`c9f017b` 仅是 PR #77 已合入时的历史基线，不包含本次 Gate-0 扩展。新 source revision 是分支 `docs/2026-09-04-platform-route` 的 Gate-0 commit，合入后由主线实际 merge SHA 固定并供下游 CI 镜像/漂移检查；不得把未合入分支描述为当前 main。v1 凭证采用可离线强制的有界 bearer replay policy：Room credential 仅接受六元分配上下文绑定，WS credential 使用 unbound sentinel 且不可入 Room；WSS/TLS 与审计降低暴露风险。nonce 仅用于唯一性与审计，不引入在线 nonce-consumption 表，也不宣称全局单活跃会话，300 秒 TTL 保持不变。

- **新增** [`engine/wire/platform-port-v1.json`](../../engine/wire/platform-port-v1.json)（`lumio.platform-port.v1`）：HTTP 绑定；操作 `request_code` / `register` / `login` / `logout` / `me` / `set_avatar` / `list_avatars` / `launch`；会话（Cookie `lumio_platform_session`，HttpOnly、SameSite=Lax、Secure 随 https、14 天滑动）；`registrationProfile`；`Profile` 形状；失败码与 HTTP 状态映射；limits；正反用例。
- **修订** [`engine/wire/account-port-v1.json`](../../engine/wire/account-port-v1.json)（本 ADR 授权）：`purpose` / `roleSemantics.account-server` / `topology.accountServer.{repository,directory,processModel,durability}` 改为 LumioPlatform 与 PostgreSQL；`process.accountServer.{listen,readiness,shutdown,exitCodes}` 改为平台进程边界（`PLATFORM_READY` 行、`PLATFORM_LISTEN_URL`、PostgreSQL 语义的关闭与退出码）；新增 `registrationProfile` 节、`registration_requires_platform` 失败码及其语义与反用例；admission canonical payload 追加六元分配绑定字段并冻结 verifier 比对语义。消息形状、Bot 凭证、顶号、TTL limits 保持既有语义。
- **Game Server 侧**：`verify_admission` 仍离线验签；输出增加六个分配绑定声明，随后逐一比较当前分配上下文。公钥分发与 `keyId` 语义不变。
- 平台内部 API（反馈、后台、埋点）不进 `engine/wire`：前后端同仓，其真值是 C# 生成的 OpenAPI 文档。

## 失败语义

- `wrong_password` / `invalid_credentials` 零覆写；并发首次注册收敛为一个 AccountId（数据库唯一约束 + 事务重试）。
- Bot 命名空间四触点（register / claim / login / admission）只在账号域一处设防；HTTP 注册对 Bot 命名空间一律 `bot_namespace_register_forbidden`。
- `production` profile 下 WS 端口对普通新 loginName 拒 `registration_requires_platform`；任何环境不得把 `test` profile 带上生产。
- 缺 `PLATFORM_DB_CONNECTION_STRING` 即启动失败（退出码 1）；SMTP 未配置即注册请求 503 `email_unconfigured`，不静默退回。
- 口令、哈希、凭证原文、私钥不进响应、审计、日志、组件、证据。
- 源码出现第二份账号库 / 口令哈希 / 凭证签发 / 协议真值 / 手写 DTO——结构断言失败，收口审查退回。

## 兼容影响

- ADR-054：第 1 条归属与第 7 条目录声明被本 ADR 取代；正文不改写，追加「修订记录（2026-09-04，ADR-061）」段。
- `knowledge/features/architecture.md` §1 拓扑与 §2 仓库职责加 `LumioPlatform` 行、`LumioServer` 行去掉账号服；`README.md` / `README.en.md` 同表；`knowledge/standards/repository-architecture.md` 加 consumer 分类。
- `knowledge/features/ds-server.md` M2 与 §2 主线、`ecs-entity-chat.md` §2：账号服归 LumioPlatform，持久库 PostgreSQL，端口两绑定；语义不变。
- `.spec/AGENTS.md`、`knowledge/standards/workflow.md`、`skills/cross-repo-delivery`、`skills/td-progress-audit`：「七个实现仓」→「八个」。
- `.github/workflows/repository-policy.yml`：LumioPlatform 有可构建代码（P0-1）后追加 checkout 与契约镜像校验；本 ADR 不改 CI。
- 下游：`LumioServer`（`account-server/` 退役、README / `.spec` 条目、CI job）、`LumioClient`（游戏页经 launch 端口）、`LumioGame`（集成启动器起平台进程，需 PostgreSQL service）、Workflow `lumiogamesengine` 项目新增 `LumioPlatform` 作为 `module` 值并新建需求室。

## 迁移方案

按 Gate 逻辑执行：G0 契约/治理冻结（含 audience-bound admission 与 allocator allowlist）→ G1 安全账号纵切（WS unbound credential、CSRF/OTP/session hardening）→ G2 安全 Launch 与准入（server-owned allocationContext、WSS allowlist、六元比对）→ G3 真实引擎网络/持久化纵切 → G4 容量与故障演练 → G5 生产上线门（集成考卷、退役旧账号服、限流与镜像定稿）。各 Gate 内可沿用平台卡片的逻辑 Wave DAG；跨仓卡按 `cross-repo-delivery` 派。

## 验证 Fixture

1. **一库**：平台源码只有一份 `Argon2idPasswordHasher`、一份 account-auth / Room-admission credential signer、一个 `accounts` 表；`LumioServer` 仓 `grep -r account-server` 零命中（旧账号服退役 Gate 后）。
2. **两端口同库与可执行交换**：经 HTTP 注册的账号可在 WS 端口用 loginName + 口令登录并取得同一 `accountId` 的 `accountAuthCredential`；Bot/工具/集成消费者用该 Bearer 调 Launch（仅提交 slug），由服务端 allocator 生成上下文并换得 Room-bound credential，随后 `verify_admission` 成功。
3. **契约冻结**：`account-port-v1.json` 与 `platform-port-v1.json` 的 admission binding 字段序/类型完全一致，所有消费者以 `docs/2026-09-04-platform-route` Gate-0 合入后的实际 merge SHA 为镜像源原子更新；平台通过其全部冻结用例，测试名与用例名一致。
4. **Game Server 原子升级**：`LumioServer` 的 `verify_admission` 实现与测试必须与本契约同步更新，读取 server-owned `allocationContext` 并验证六元绑定；不得继续接受 WS unbound credential 进入 Room。
5. **profile 设防**：`production` 下 WS 普通新 loginName 拒 `registration_requires_platform`；`test` 下照 ADR-054 创建；HTTP 注册 Bot 名拒 `bot_namespace_register_forbidden`。
6. **launch 中立**：换分配器实现（固定端点 → 假登记表）后 `launch` 应答形状与游戏页代码不变（LumioClient 页面 `git diff` 为空）。
7. **持久**：平台重启后同口令重登返回同一 `accountId`（PostgreSQL）；仓内无 JSON 账号文件。
8. **集成考卷**：RM-00011 R4-09 考卷完成 `/account` 登录 → account-auth Bearer 调 Launch → Room-bound credential 经 server-owned allocationContext 验证的全链路，证据含 `PLATFORM_READY` 行；同一 PR 删除 `LumioServer/account-server/`。
9. **登记完整**：架构仓 spec-lint 通过；`architecture.md` / README 双语 / skills / AGENTS 中「七」全部改为「八」（grep 零命中「七个实现仓」）。
