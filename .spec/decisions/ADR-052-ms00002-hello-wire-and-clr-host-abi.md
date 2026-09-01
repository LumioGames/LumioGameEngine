# ADR-052：MS-00002 开发态 Hello wire 契约与 CLR 装载 ABI

状态：Accepted（2026-08-31，随 MS-00002 Hello World 里程碑验收）
取代：无（ADR-049 的 V1.5 基线化路线未被采纳执行，见下）

## 背景

MS-00002（Hello World）要求在 Windows x64 本机以真实进程链（Rust Server + SDK Native DLL + CoreCLR/C# Runtime 权威 Tick + 真实 Chromium 浏览器 + 独立 Headless Bot）完成双向 Hello World。2026-08-31 用户交付指令把旧方案（R-00336 原卡面：把 ADR-049 的 InputCommand/stateBlocks 升为 V1.5 生效基线 + Fixture 门 + 下游 repin）收敛为开发态最小契约：主线已切换为预上线 Living Architecture（不执行 Baseline/Schema/Fixture/镜像/发布门），且「如果现有 V1.4 契约足够，优先复用，不要为了 HelloWorld 扩大公共协议」。

同时，CoreEngine（本仓 engine/native）此前没有到 CoreCLR 的装载链：Server 侧需要经 SDK 公共 ABI 启动运行时并调用固定托管入口，且不得为此复制第二套 Loader/ABI。

## 决策

1. **开发态最小 wire 契约**：Hello World 的 WebSocket 消息（Handshake/HandshakeAck/FullSnapshot/BaselineAck/InputCommand/Delta/Error/Shutdown）、字段语义（sender/sequence/revision/payloadSha256/latency）、错误码、limits、进程边界（readiness/shutdown/退出码）与审计事件词表的唯一真值是 `engine/wire/hello-wire-v1.json`；校验入口 `node eng/verify-hello-wire.mjs`。消费方（Rust Server、C# Runtime、Browser、Bot、集成验收）不得另写协议真值；浏览器与 Bot 按契约 required 文法动态校验。该契约是开发态里程碑产物，不是 Baseline；进入正式硬化阶段再按治理顺序升级为版本化公共合同。
2. **CLR 装载链进 SDK 根 ABI**：`engine/abi/native-abi.json` 的根表 v1 追加三个 C ABI 函数——`create_clr_host(hostfxr_path, runtime_config_path, assembly_path, entry_spec, out_handle)`、`clr_host_call(host, input, len, output, cap, out_written)`、`destroy_clr_host(host)`；状态码追加 `ClrInitFailed/ClrEntryFailed/BufferTooSmall`。实现位于 `engine/native/modules/clr-host`（手写 kernel32+hostfxr FFI，零新增外部依赖），sdk-native 根表三槽直转，无第二套装载实现。
3. **两个实测语义钉进定义**：entry_spec 第二段必须是**托管方法名**（不是 `[UnmanagedCallersOnly]` 的 EntryPoint 别名；传别名 hostfxr 返回 0x80131513 MissingMethod）；CoreCLR 每进程只能成功初始化一次（hostfxr_close 不卸载已启动运行时，二次 create 在 initialize 步返回 0x80008081），宿主在进程生命周期内至多调用一次 create_clr_host。
4. **字节协议边界**：托管入口固定原生签名 `int32_t(*)(const uint8_t*, int32_t, uint8_t*, int32_t, int32_t*)`，输入/输出缓冲均由调用方提供（输出容量不足返回 BufferTooSmall 并携带所需长度），不跨 ABI 转移内存所有权；op 词表（enqueue/tick/snapshot/shutdown）与 wire 契约同字段词汇。

## 替代方案

- **V1.5 生效基线路线（ADR-049 全面基线化）**：被用户指令收敛否决；旧候选（约 122 路径未提交）留档于隔离 worktree，未合入未删除。
- **Server 仓自带 CoreCLR hosting**：会把装载链复制进消费者，违背「单一 Loader/ABI」；且 R-00339 的装载链职责在 CoreEngine。
- **runtimeconfig additionalProbingPaths 解决组件依赖**：实测对组件式装载（initialize_for_runtime_config）无效，不采用。

## 接口

见 `engine/abi/native-abi.json`（ABI）与 `engine/wire/hello-wire-v1.json`（wire）；架构正文 §3.2/§3.3/§4.1。

## 失败语义

- 装载失败（hostfxr 缺失/坏 runtimeconfig/入口解析失败）→ `ClrInitFailed`，完整回滚无半初始化句柄；Server 以退出码 1 拒绝服务。
- 身份不匹配（abi_hash/build_id/binarySha256）→ Loader（C#）与 Server（Rust）双侧稳定拒绝。
- 域级拒绝（重复序列/坏哈希等）→ 托管入口返回 `{"ok":false,"code":<契约错误码>}`（rc=0），Server 转发为 wire Error 消息。

## 兼容影响

根表 v1 追加槽位使 struct_size 88（x64）；C# Loader 校验放宽为 `StructSize >= 新布局` 且三 CLR 槽非空。预上线允许破坏式变化，无兼容窗口义务。

## 迁移方案

无需迁移（新能力）。正式硬化阶段若升级 wire 契约为版本化公共合同，以新契约文件 + 生成面承接，本 ADR 记录的语义约束（方法名、单次初始化、字节协议边界）继续有效。

## 验证

- `node eng/generate-abi.mjs` 零差异；`node eng/verify-hello-wire.mjs` 9/9（含失败探针）。
- `cargo test -p lumio-clr-host`：双名 fixture 反例探针（别名→MissingMethod）+ 真实 hostfxr 端到端。
- 端到端证据：LumioGame `integration/hello/evidence-run1/`（两轮 SUCCESS，非 Echo 链由 audit 证明）。
