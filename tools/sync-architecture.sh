#!/usr/bin/env bash
# tools/sync-architecture.sh — 架构源锁定镜像同步工具（LCE-P0-002，规格 §3.4/§3.5/§3.6）。
# POSIX sh 兼容（bash / dash / Git Bash 均可执行）。
#
# 用法：
#   bash tools/sync-architecture.sh                # 默认：按 lock 重建只读镜像，绝不改 lock
#   bash tools/sync-architecture.sh --update-lock  # 显式重生成 lock + 镜像；必须单独 PR 提交
#   bash tools/sync-architecture.sh --fetch-tools  # 按 pin 提交号提取架构仓 tools/** 到
#                                                  # build/architecture-tools/<commit>/（不提交，
#                                                  # 不入镜像；R-00263/D-5）
#
# 行为契约（规格 §18 LCE-P0-002「实现要求」）：
#   * 架构源内容一律经 `git show <pin-commit>:<path>` 从对象库提取，绝不读源仓工作区，
#     也绝不读本仓 docs/architecture/（它不是生成输入）。
#   * 镜像先写临时目录，全量验证（required path 存在 + 逐文件 SHA-256 与 lock 一致）后
#     原子 rename 替换；任一验证失败即退出非零，lock 与既有镜像均不被触碰。
#   * 镜像文件置为只读（chmod a-w）并断言生效。
#   * 若架构源内容与 lock 不同：命令失败且不更新 lock（规格 §3.5）。
#   * lock 只能被显式 --update-lock 重写；pin 的 repository/commit/baseline 由本工具常量
#     钉死，不会因源仓或相邻仓出现新基线（如 V1.4）而静默升级——升级 = 改本文件 = 单独评审。
#   * 每次执行写带时间戳的执行记录到 build/reports/architecture-sync.json（build/ 不提交）。
#
# 源仓定位（按序）：环境变量 LUMIO_ARCHITECTURE_REPO → <仓根>/../LumioGameEngineArchitecture
#   → <仓根>/../../LumioGameEngineArchitecture（覆盖主仓与 git worktree 两种布局）。
#   源仓 origin 远程必须与 lock.repository 一致（允许 .git 后缀差异）。
#
# 镜像投影（源路径 → generated/architecture/LGE-V1.4-2026-08-27/ 下相对路径）：
#   schemas/** → schemas/**；ids/** → ids/**；fixtures/** → fixtures/**；
#   .spec/decisions/** → decisions/**；packages/ 按 packages/index.json 的 consumers 关系
#   投影（含 index 自身；仅条目 consumers 含 LumioCoreEngine 的所指文件纳入——范围是关系
#   的函数而非路径快照，上游增删本仓消费面时镜像自动跟随；解析依赖 node，与收口门槛
#   spec-lint 同一既有依赖）。tools/** 是实现而非契约，不入 requiredPaths 与镜像
#   （R-00263/D-5：上游改工具不再打断本仓门禁）；需要架构仓校验器时用 --fetch-tools。
#   镜像根另有两个工具生成的元文件：generation-record.json（§3.6 生成记录，其 SHA-256 钉在
#   lock 的 generatedArtifactDescriptorSha256）与 .gitattributes（`* -text`，保证任何
#   core.autocrlf 配置下检出不改字节）。
#
# Hash 定义（与 tools/verify-architecture-lock.sh 同一口径）：
#   逐文件 SHA-256 = 文件字节摘要；Input/Output Hash = 按 source path 字典序（LC_ALL=C）
#   拼接 `<path> <sha256>\n` 的字节流整体 SHA-256。镜像为原样复制，二者恒等，
#   verify 端从镜像字节重算复核。
#
# 退出码：0 成功；1 失败（源仓身份不符 / pin 提交不可复现 / 内容与 lock 漂移 /
#         只读断言失败 / 替换后校验失败（自动回滚））。

set -eu
export LC_ALL=C

repo_root=$(cd "$(dirname "$0")/.." && pwd)
lock_file="$repo_root/architecture.lock.json"
mirror_parent="$repo_root/generated/architecture"
mirror_root="$mirror_parent/LGE-V1.4-2026-08-27"
mirror_rel="generated/architecture/LGE-V1.4-2026-08-27"
descriptor_rel="$mirror_rel/generation-record.json"
expected_schema_version=1
expected_repository="https://github.com/LumioGames/LumioGameEngineArchitecture"
expected_commit="1f2ead332b3dfc3042e1495bfbe6febb8699df7e"
expected_baseline="LGE-V1.4-2026-08-27"
compiler_rel="tools/sync-architecture.sh"
compiler_version=1
record_argv="bash tools/sync-architecture.sh"
record_target_platform="host-independent"

fail() {
    echo "tools/sync-architecture.sh: FAIL: $1" >&2
    exit 1
}

# 源路径 → 镜像相对路径 投影（stdout 输出；不可投影返回 1）。
project() {
    case "$1" in
        .spec/decisions/*) printf 'decisions/%s\n' "${1#.spec/decisions/}" ;;
        schemas/* | ids/* | fixtures/* | packages/*) printf '%s\n' "$1" ;;
        *) return 1 ;;
    esac
}

# packages/ 的 consumers 投影：stdin 喂入 packages/index.json，stdout 输出本仓消费的
# packages/ 前缀路径（含 packages/index.json 自身，已排序）。规则：
#   * 条目（rootAbi / trust / canonicalDigest / evidence / loader / artifacts[]…）的
#     consumers 含 $consumer_name 才纳入，取其 bundlePath / profilePath / outputFiles[].path；
#   * outputFiles 中落在某个 artifacts[].packagePath 目录下的文件归属该 artifact 条目
#     （其 consumers 单独裁决），不随宿主条目纳入；
#   * 消费面以 packagePath 目录声明（artifacts 条目）而含本仓时，目录级投影未实现——
#     立即失败上报，不静默猜测。
# 与 tools/verify-architecture-lock.sh 内同名函数保持同一口径。
consumer_name="LumioCoreEngine"
consumers_projection() {
    node -e '
        let s = "";
        process.stdin.on("data", (d) => (s += d));
        process.stdin.on("end", () => {
            const idx = JSON.parse(s);
            const me = process.argv[1];
            const artDirs = (idx.artifacts || []).map((a) => a.packagePath).filter(Boolean);
            const inArtifact = (p) => artDirs.some((d) => p.startsWith(d));
            const entries = Object.entries(idx).filter(
                ([, v]) => v && typeof v === "object" && !Array.isArray(v) && Array.isArray(v.consumers)
            );
            for (const a of idx.artifacts || [])
                if (Array.isArray(a.consumers)) entries.push(["artifacts:" + a.artifactId, a]);
            const out = new Set(["packages/index.json"]);
            for (const [key, e] of entries) {
                if (!e.consumers.includes(me)) continue;
                if (e.packagePath) {
                    console.error(
                        "consumers 条目 " + key + " 以 packagePath 目录声明消费面，目录级投影未实现，不得静默猜测"
                    );
                    process.exit(3);
                }
                for (const p of [e.bundlePath, e.profilePath]) if (p) out.add("packages/" + p);
                for (const f of e.outputFiles || [])
                    if (f.path && !inArtifact(f.path)) out.add("packages/" + f.path);
            }
            for (const p of [...out].sort()) console.log(p);
        });
    ' "$consumer_name"
}

# ── 参数 ───────────────────────────────────────────────────────────────────
update_lock=0
fetch_tools=0
while [ $# -gt 0 ]; do
    case "$1" in
        --update-lock) update_lock=1 ;;
        --fetch-tools) fetch_tools=1 ;;
        *) fail "未知参数：$1（仅接受可选 --update-lock / --fetch-tools）" ;;
    esac
    shift
done
[ "$update_lock" -eq 1 ] && [ "$fetch_tools" -eq 1 ] &&
    fail "--update-lock 与 --fetch-tools 不可同时使用"

# ── 临时目录与清理（§15.1 口径：列表/装配 scratch 在 build/.tmp 下；
#    镜像临时目录必须与最终位置同文件系统，保证 rename 原子） ────────────────
scratch="$repo_root/build/.tmp/architecture-sync-$$"
tmp_mirror="$mirror_parent/.tmp-sync-$$"
old_mirror="$mirror_parent/.old-sync-$$"
mkdir -p "$scratch" "$mirror_parent"
rm -rf "$tmp_mirror" "$old_mirror" 2>/dev/null || :
mkdir -p "$tmp_mirror"
cleanup() {
    rm -rf "$scratch" 2>/dev/null || :
    chmod -R u+w "$tmp_mirror" 2>/dev/null || :
    rm -rf "$tmp_mirror" 2>/dev/null || :
}
trap cleanup EXIT INT TERM

# ── 定位并校验架构源仓（只读；绝不写源仓） ──────────────────────────────────
src_repo=${LUMIO_ARCHITECTURE_REPO:-}
if [ -z "$src_repo" ]; then
    for cand in "$repo_root/../LumioGameEngineArchitecture" \
        "$repo_root/../../LumioGameEngineArchitecture"; do
        if git -C "$cand" rev-parse --git-dir >/dev/null 2>&1; then
            src_repo=$cand
            break
        fi
    done
fi
[ -n "$src_repo" ] || fail "未找到架构源仓：请设置 LUMIO_ARCHITECTURE_REPO 或将源仓放在 ../LumioGameEngineArchitecture（worktree 布局为 ../../）"
git -C "$src_repo" rev-parse --git-dir >/dev/null 2>&1 ||
    fail "LUMIO_ARCHITECTURE_REPO 指向的不是 git 仓库：$src_repo"

origin_url=$(git -C "$src_repo" remote get-url origin 2>/dev/null || true)
[ -n "$origin_url" ] || fail "架构源仓 $src_repo 未配置 origin 远程，无法核对 lock.repository"
strip_git_suffix() { printf '%s\n' "${1%.git}"; }
[ "$(strip_git_suffix "$origin_url")" = "$(strip_git_suffix "$expected_repository")" ] ||
    fail "架构源仓身份不符：origin=$origin_url，lock 固定 $expected_repository"

git -C "$src_repo" cat-file -e "${expected_commit}^{commit}" 2>/dev/null ||
    fail "锁定提交 $expected_commit 在架构源仓中不存在（V1.4 基线不可复现，立即阻塞上报）"

# ── --fetch-tools：按 pin 提交号另行获取架构仓校验器工具链（R-00263/D-5）。
#    tools/** 是实现而非契约，不入 requiredPaths 与镜像；需要 lumio_contract.py 等
#    校验器时提取到 build/（不提交），与 pin 提交严格同源。 ─────────────────────
if [ "$fetch_tools" -eq 1 ]; then
    tools_dest="$repo_root/build/architecture-tools/$expected_commit"
    rm -rf "$tools_dest"
    mkdir -p "$tools_dest"
    git -C "$src_repo" archive "$expected_commit" tools | tar -x -C "$tools_dest" ||
        fail "从 pin 提交 $expected_commit 提取 tools/ 失败"
    tools_count=$(find "$tools_dest" -type f | wc -l | tr -d ' ')
    [ "$tools_count" -gt 0 ] || fail "pin 提交 $expected_commit 下 tools/ 为空（异常，立即阻塞上报）"
    echo "tools: 已按 pin $expected_commit 提取架构仓 tools/**（$tools_count 个文件）到 build/architecture-tools/$expected_commit/（不提交，不入镜像；R-00263/D-5）"
    echo "tools/sync-architecture.sh: OK"
    exit 0
fi

# ── 读取 lock（默认模式） ──────────────────────────────────────────────────
if [ "$update_lock" -eq 0 ]; then
    [ -f "$lock_file" ] || fail "缺少 $lock_file（首次生成请显式运行 --update-lock）"
    lock_json=$(tr -d '\r' <"$lock_file")

    lock_str_field() {
        printf '%s\n' "$lock_json" |
            sed -n "s/^[[:space:]]*\"$1\":[[:space:]]*\"\([^\"]*\)\".*$/\1/p" |
            head -n 1
    }
    lock_commit=$(lock_str_field commit)
    lock_baseline=$(lock_str_field architectureBaselineId)
    lock_repository=$(lock_str_field repository)
    lock_descriptor=$(lock_str_field generatedArtifactDescriptorPath)
    lock_descriptor_sha=$(lock_str_field generatedArtifactDescriptorSha256)
    lock_schema_version=$(printf '%s\n' "$lock_json" |
        sed -n 's/^[[:space:]]*"schemaVersion":[[:space:]]*\([0-9][0-9]*\).*$/\1/p' | head -n 1)

    [ "$lock_schema_version" = "$expected_schema_version" ] || fail "lock schemaVersion 非 $expected_schema_version"
    [ "$lock_commit" = "$expected_commit" ] ||
        fail "lock.commit=$lock_commit 与工具钉死的 $expected_commit 不符（改基线须改本工具常量并单独 PR）"
    [ "$lock_baseline" = "$expected_baseline" ] || fail "lock.architectureBaselineId=$lock_baseline 与工具钉死的 $expected_baseline 不符"
    [ "$lock_repository" = "$expected_repository" ] || fail "lock.repository=$lock_repository 与工具钉死的 $expected_repository 不符"
    [ "$lock_descriptor" = "$descriptor_rel" ] ||
        fail "lock.generatedArtifactDescriptorPath=$lock_descriptor 与预期 $descriptor_rel 不符"
    case "$lock_descriptor_sha" in
        *[!0-9a-f]* | "") fail "lock.generatedArtifactDescriptorSha256 不是 64 位小写十六进制" ;;
        ????????????????????????????????????????????????????????????????) : ;;
        *) fail "lock.generatedArtifactDescriptorSha256 长度不是 64" ;;
    esac

    # requiredPaths 数组与 requiredPathSha256 对象 → "path sha" 行（保持 lock 内顺序，已要求有序）。
    lock_map=$(printf '%s\n' "$lock_json" | awk '
        state == 0 && /"requiredPaths"[[:space:]]*:/ { state = 1; next }
        state == 1 && /^[[:space:]]*\]/ { state = 2; next }
        state == 1 {
            line = $0
            while (match(line, /"[^"]+"/)) {
                print "P " substr(line, RSTART + 1, RLENGTH - 2)
                line = substr(line, RSTART + RLENGTH)
            }
        }
        state == 2 && /"requiredPathSha256"[[:space:]]*:/ { state = 3; next }
        state == 3 && /^[[:space:]]*\}/ { state = 4; next }
        state == 3 && match($0, /"[^"]+"[[:space:]]*:[[:space:]]*"[^"]+"/) {
            kv = substr($0, RSTART, RLENGTH)
            split(kv, part, /[[:space:]]*:[[:space:]]*/)
            gsub(/"/, "", part[1]); gsub(/"/, "", part[2])
            print "H " part[1] " " part[2]
        }')
    required_paths=$(printf '%s\n' "$lock_map" | awk '$1 == "P" { print $2 }')
    [ -n "$required_paths" ] || fail "lock.requiredPaths 未解析出任何条目"
    n_paths=$(printf '%s\n' "$required_paths" | wc -l | tr -d ' ')
    printf '%s\n' "$required_paths" | sort -cu || fail "lock.requiredPaths 未排序或含重复（lock 应由 --update-lock 生成）"
    bad_paths=$(printf '%s\n' "$required_paths" | grep -v '^[A-Za-z0-9._/-]*$' || true)
    [ -z "$bad_paths" ] || fail "lock.requiredPaths 含非法路径字符：$bad_paths"
    lock_hashes=$(printf '%s\n' "$lock_map" | awk '$1 == "H" { print $2 " " $3 }')
    n_hashes=$(printf '%s\n' "$lock_hashes" | wc -l | tr -d ' ')
    [ "$n_hashes" -eq "$n_paths" ] ||
        fail "lock.requiredPathSha256 条目数 $n_hashes 与 requiredPaths 数 $n_paths 不一致"
    printf '%s\n' "$lock_hashes" | sort -cu || fail "lock.requiredPathSha256 未排序或含重复"
    # 64 个十六进制字符逐位展开为字符类，兼容不支持的间隔表达式（{64}）的 POSIX awk。
    bad_sha=$(printf '%s\n' "$lock_hashes" | awk '$2 !~ /^[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]$/ { print $1 }')
    [ -z "$bad_sha" ] || fail "lock.requiredPathSha256 存在非 64 位小写十六进制的条目：$bad_sha"
    for p in $required_paths; do
        project "$p" >/dev/null || fail "lock.requiredPaths 含不可投影路径：$p（仅允许 schemas/ ids/ fixtures/ .spec/decisions/ packages/ 前缀）"
    done
    printf '%s\n' "$lock_hashes" | sort >"$scratch/lock-map.txt"
fi

# ── 枚举 required paths（--update-lock 模式：四个源子树全集 + packages/ 的
#    consumers 投影，均从 pin 提交枚举） ─────────────────────────────────────
if [ "$update_lock" -eq 1 ]; then
    if [ -f "$lock_file" ]; then
        echo "--update-lock：将基于 pin 提交 $expected_commit 重新生成 $lock_file（必须单独 PR 提交）"
    else
        echo "--update-lock：首次生成 $lock_file（pin 提交 $expected_commit）"
    fi
    subtree_paths=$(git -C "$src_repo" ls-tree -r --name-only "$expected_commit" -- \
        schemas ids fixtures .spec/decisions)
    packages_paths=$(git -C "$src_repo" -c core.autocrlf=false show \
        "${expected_commit}:packages/index.json" | consumers_projection) ||
        fail "packages/ consumers 投影计算失败（pin 提交缺 packages/index.json、JSON 不可解析或含未实现的目录级消费声明）"
    required_paths=$(printf '%s\n%s\n' "$subtree_paths" "$packages_paths" |
        sed '/^$/d' | LC_ALL=C sort)
    [ -n "$required_paths" ] || fail "pin 提交下四个源子树 + packages/ consumers 投影枚举为空（异常，立即阻塞上报）"
    n_paths=$(printf '%s\n' "$required_paths" | wc -l | tr -d ' ')
fi

# ── 提取 + 逐文件摘要（写入临时镜像） ───────────────────────────────────────
entries="$scratch/entries.txt"
: >"$entries"
missing_src="$scratch/missing-src.txt"
: >"$missing_src.txt"
for p in $required_paths; do
    proj=$(project "$p")
    mkdir -p "$tmp_mirror/$(dirname "$proj")"
    if ! git -C "$src_repo" -c core.autocrlf=false show "${expected_commit}:${p}" >"$tmp_mirror/$proj" 2>/dev/null; then
        printf '%s\n' "$p" >>"$missing_src.txt"
        continue
    fi
    sha=$(sha256sum "$tmp_mirror/$proj" | awk '{ sub(/^\\/, ""); print $1 }')
    printf '%s %s\n' "$p" "$sha" >>"$entries"
done
if [ -s "$missing_src.txt" ]; then
    echo "tools/sync-architecture.sh: FAIL: pin 提交缺少 required path（共 $(wc -l <"$missing_src.txt" | tr -d ' ') 个）：" >&2
    sed 's/^/  缺失: /' "$missing_src.txt" >&2
    exit 1
fi

# ── 默认模式：架构源内容必须与 lock 完全一致，否则失败且不动 lock ────────────
if [ "$update_lock" -eq 0 ]; then
    if ! diff -u "$scratch/lock-map.txt" "$entries" >/dev/null; then
        echo "tools/sync-architecture.sh: FAIL: 架构源内容与 lock 不一致（源漂移或 lock 过期）；lock 未被修改。差异（lock → 源）：" >&2
        diff "$scratch/lock-map.txt" "$entries" | sed 's/^/  /' >&2 || true
        exit 1
    fi
fi

# ── 生成记录（§3.6：compiler/argv/输入输出摘要；确定性内容，不含时间——
#    时间戳只进 build/reports 下的执行记录） ─────────────────────────────────
# 编译器摘要：对 CRLF 归一化后的脚本文本计算（脚本受根 .gitattributes text=auto 影响，
# 不同平台检出行尾不同；归一化保证跨平台一致，与 verify-architecture-lock.sh 同口径）。
compiler_sha=$(tr -d '\r' <"$repo_root/$compiler_rel" | sha256sum | awk '{ print $1 }')
file_count=$(wc -l <"$entries" | tr -d ' ')
stream_hash=$(sha256sum "$entries" | awk '{ sub(/^\\/, ""); print $1 }')
{
    printf '{\n'
    printf '  "schemaVersion": %s,\n' "$expected_schema_version"
    printf '  "kind": "architecture-mirror-generation-record",\n'
    printf '  "repository": "%s",\n' "$expected_repository"
    printf '  "commit": "%s",\n' "$expected_commit"
    printf '  "architectureBaselineId": "%s",\n' "$expected_baseline"
    printf '  "mirrorRoot": "%s",\n' "$mirror_rel"
    printf '  "compilerName": "%s",\n' "$compiler_rel"
    printf '  "compilerVersion": %s,\n' "$compiler_version"
    printf '  "compilerSha256": "%s",\n' "$compiler_sha"
    printf '  "argv": "%s",\n' "$record_argv"
    printf '  "targetPlatform": "%s",\n' "$record_target_platform"
    printf '  "fileCount": %s,\n' "$file_count"
    printf '  "inputHash": "%s",\n' "$stream_hash"
    printf '  "outputHash": "%s",\n' "$stream_hash"
    printf '  "pathSha256": {\n'
    awk -v last="$file_count" \
        'NR < last { printf "    \"%s\": \"%s\",\n", $1, $2 }
         NR == last { printf "    \"%s\": \"%s\"\n", $1, $2 }' "$entries"
    printf '  }\n'
    printf '}\n'
} >"$tmp_mirror/generation-record.json"

# 镜像字节冻结声明：防止任何 core.autocrlf/.gitattributes 组合在检出时改写字节。
printf '* -text\n' >"$tmp_mirror/.gitattributes"

# ── 只读权限 + 断言 ────────────────────────────────────────────────────────
chmod -R a-w "$tmp_mirror"
find "$tmp_mirror" -type f >"$scratch/files.txt"
writable_count=0
while IFS= read -r f; do
    if [ -w "$f" ]; then
        echo "  只读断言失败（仍可写）: $f" >&2
        writable_count=$((writable_count + 1))
    fi
done <"$scratch/files.txt"
[ "$writable_count" -eq 0 ] || fail "chmod a-w 未生效：$writable_count 个镜像文件仍可写"

# ── 原子替换（旧镜像先移走，失败回滚） ──────────────────────────────────────
had_old=0
if [ -e "$mirror_root" ]; then
    had_old=1
    mv "$mirror_root" "$old_mirror"
fi
if ! mv "$tmp_mirror" "$mirror_root"; then
    if [ "$had_old" -eq 1 ]; then
        mv "$old_mirror" "$mirror_root"
        fail "镜像替换失败（已回滚旧镜像）"
    fi
    fail "镜像替换失败"
fi
tmp_mirror="$mirror_parent/.tmp-sync-consumed-$$" # 已消费，防 cleanup 误删新镜像

rollback() {
    rm -rf "$mirror_root" 2>/dev/null || :
    if [ "$had_old" -eq 1 ]; then
        chmod -R u+w "$old_mirror" 2>/dev/null || :
        mv "$old_mirror" "$mirror_root" 2>/dev/null || :
    fi
}

# ── --update-lock：重生成 lock（临时写 + 原子替换；失败回滚） ────────────────
if [ "$update_lock" -eq 1 ]; then
    record_sha=$(sha256sum "$mirror_root/generation-record.json" | awk '{ sub(/^\\/, ""); print $1 }')
    lock_tmp="$scratch/architecture.lock.json.new"
    {
        printf '{\n'
        printf '  "schemaVersion": %s,\n' "$expected_schema_version"
        printf '  "repository": "%s",\n' "$expected_repository"
        printf '  "commit": "%s",\n' "$expected_commit"
        printf '  "architectureBaselineId": "%s",\n' "$expected_baseline"
        printf '  "requiredPaths": [\n'
        awk -v last="$file_count" \
            'NR < last { printf "    \"%s\",\n", $1 }
             NR == last { printf "    \"%s\"\n", $1 }' "$entries"
        printf '  ],\n'
        printf '  "requiredPathSha256": {\n'
        awk -v last="$file_count" \
            'NR < last { printf "    \"%s\": \"%s\",\n", $1, $2 }
             NR == last { printf "    \"%s\": \"%s\"\n", $1, $2 }' "$entries"
        printf '  },\n'
        printf '  "generatedArtifactDescriptorPath": "%s",\n' "$descriptor_rel"
        printf '  "generatedArtifactDescriptorSha256": "%s"\n' "$record_sha"
        printf '}\n'
    } >"$lock_tmp"
    lock_backup="$scratch/architecture.lock.json.bak"
    had_lock=0
    if [ -f "$lock_file" ]; then
        had_lock=1
        cp "$lock_file" "$lock_backup"
    fi
    mv -f "$lock_tmp" "$lock_file"
fi

# ── 替换后全量校验（复用 check-contracts 的校验器） ─────────────────────────
if ! bash "$repo_root/tools/verify-architecture-lock.sh"; then
    echo "tools/sync-architecture.sh: FAIL: 替换后 check-contracts 校验失败，回滚" >&2
    if [ "$update_lock" -eq 1 ] && [ "$had_lock" -eq 1 ]; then
        cp "$lock_backup" "$lock_file"
    fi
    rollback
    exit 1
fi

# ── 收尾：清理旧镜像 + 写执行记录（时间戳只进这里） ─────────────────────────
if [ "$had_old" -eq 1 ]; then
    chmod -R u+w "$old_mirror" 2>/dev/null || :
    rm -rf "$old_mirror"
fi
old_mirror="$mirror_parent/.old-sync-consumed-$$"

mkdir -p "$repo_root/build/reports"
mode_name=sync
[ "$update_lock" -eq 1 ] && mode_name=update-lock
cat >"$repo_root/build/reports/architecture-sync.json" <<EOF
{
  "schemaVersion": 1,
  "kind": "architecture-sync-execution-record",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "mode": "$mode_name",
  "sourceRepo": "$src_repo",
  "repository": "$expected_repository",
  "commit": "$expected_commit",
  "architectureBaselineId": "$expected_baseline",
  "fileCount": $file_count,
  "inputHash": "$stream_hash",
  "outputHash": "$stream_hash",
  "result": "success"
}
EOF

echo "source: $expected_repository @ $expected_commit（经 git show 从对象库提取，未读源仓工作区）"
echo "mirror: $mirror_rel（$file_count 个 required path + generation-record.json + .gitattributes；只读）"
echo "inputHash == outputHash: $stream_hash"
if [ "$update_lock" -eq 1 ]; then
    echo "lock: 已重生成 $lock_file（警告：lock 更新必须以显式 --update-lock 产出并走单独 PR，不得与其他改动混提）"
else
    echo "lock: $lock_file 未修改（只读消费）"
fi
echo "tools/sync-architecture.sh: OK"
