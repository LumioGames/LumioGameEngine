#!/usr/bin/env bash
# tools/verify-architecture-lock.sh — architecture lock 与只读镜像完整性校验
# （LCE-P0-002；`just check-contracts` 的唯一后端，规格 §3.4/§3.5、§20.1 门禁）。
# POSIX sh 兼容（bash / dash / Git Bash 均可执行）。
#
# 校验内容（只读本仓；不访问架构源仓，不读取 docs/architecture/——两者都不是输入）：
#   1. architecture.lock.json 结构与钉死常量：8 个固定字段齐全、schemaVersion/repository/
#      commit/architectureBaselineId/generatedArtifactDescriptorPath 与本工具常量一致、
#      requiredPaths 有序唯一且路径合法、requiredPathSha256 与之一一对应且为 64 位小写十六进制。
#   2. 镜像完整性：每个 required path 在镜像中存在（缺失逐条列出）、逐文件 SHA-256 与
#      lock 一致（漂移逐条列出）、镜像中不存在未登记文件或符号链接（逐条列出）。
#   3. 镜像根 .gitattributes 必须精确为 `* -text`（保证任何检出配置不改字节）。
#   4. 生成记录 generation-record.json：存在、SHA-256 与 lock.generatedArtifactDescriptorSha256
#      一致、字段常量一致、pathSha256 与 lock 完全相等、fileCount 一致、Input/Output Hash
#      从镜像字节重算一致、compilerSha256 与 tools/sync-architecture.sh 一致
#      （对 CRLF 归一化后计算，兼容跨平台检出；脚本变更后必须显式 --update-lock）。
#   5. 只读权限为通告项（git 不传输权限位，重新检出后以本工具 sync-contracts 恢复只读）。
#
# Hash 口径与 tools/sync-architecture.sh 相同：逐文件 SHA-256 = 字节摘要；
# Input/Output Hash = 按 source path 字典序拼接 `<path> <sha256>\n` 字节流的整体 SHA-256。
#
# 退出码：0 全部通过；1 校验失败（缺失/漂移/未登记文件/记录不一致等，均逐条列出）。

set -eu
export LC_ALL=C

repo_root=$(cd "$(dirname "$0")/.." && pwd)
lock_file="$repo_root/architecture.lock.json"
mirror_root="$repo_root/generated/architecture/LGE-V1.2-2026-08-27"
mirror_rel="generated/architecture/LGE-V1.2-2026-08-27"
descriptor_rel="$mirror_rel/generation-record.json"
expected_schema_version=1
expected_repository="https://github.com/LumioGames/LumioGameEngineArchitecture"
expected_commit="2d7980d95b163404e33cc6212db13ac948d30d40"
expected_baseline="LGE-V1.2-2026-08-27"
expected_record_kind="architecture-mirror-generation-record"
compiler_rel="tools/sync-architecture.sh"
expected_compiler_version=1
expected_record_argv="bash tools/sync-architecture.sh"
expected_record_target_platform="host-independent"

fail=0
err() {
    echo "tools/verify-architecture-lock.sh: FAIL: $1" >&2
    fail=1
}

# ── 1. 解析并校验 lock（结构性错误立即退出：后续检查无从谈起） ───────────────
[ -f "$lock_file" ] || {
    echo "tools/verify-architecture-lock.sh: FAIL: 缺少 $lock_file" >&2
    exit 1
}
lock_json=$(tr -d '\r' <"$lock_file")

lock_str_field() {
    printf '%s\n' "$lock_json" |
        sed -n "s/^[[:space:]]*\"$1\":[[:space:]]*\"\([^\"]*\)\".*$/\1/p" |
        head -n 1
}
is_sha256() {
    case "$1" in
        *[!0-9a-f]* | "") return 1 ;;
        ????????????????????????????????????????????????????????????????) return 0 ;;
        *) return 1 ;;
    esac
}

lock_schema_version=$(printf '%s\n' "$lock_json" |
    sed -n 's/^[[:space:]]*"schemaVersion":[[:space:]]*\([0-9][0-9]*\).*$/\1/p' | head -n 1)
lock_repository=$(lock_str_field repository)
lock_commit=$(lock_str_field commit)
lock_baseline=$(lock_str_field architectureBaselineId)
lock_descriptor=$(lock_str_field generatedArtifactDescriptorPath)
lock_descriptor_sha=$(lock_str_field generatedArtifactDescriptorSha256)

[ "$lock_schema_version" = "$expected_schema_version" ] ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.schemaVersion 非 $expected_schema_version" >&2; exit 1; }
[ "$lock_repository" = "$expected_repository" ] ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.repository=$lock_repository 与钉死值 $expected_repository 不符" >&2; exit 1; }
[ "$lock_commit" = "$expected_commit" ] ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.commit=$lock_commit 与钉死值 $expected_commit 不符（基线漂移）" >&2; exit 1; }
[ "$lock_baseline" = "$expected_baseline" ] ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.architectureBaselineId=$lock_baseline 与钉死值 $expected_baseline 不符" >&2; exit 1; }
[ "$lock_descriptor" = "$descriptor_rel" ] ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.generatedArtifactDescriptorPath=$lock_descriptor 与预期 $descriptor_rel 不符" >&2; exit 1; }
is_sha256 "$lock_descriptor_sha" ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.generatedArtifactDescriptorSha256 不是 64 位小写十六进制" >&2; exit 1; }

lock_pairs=$(printf '%s\n' "$lock_json" | awk '
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
required_paths=$(printf '%s\n' "$lock_pairs" | awk '$1 == "P" { print $2 }')
lock_hashes=$(printf '%s\n' "$lock_pairs" | awk '$1 == "H" { print $2 " " $3 }')
[ -n "$required_paths" ] ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.requiredPaths 未解析出任何条目" >&2; exit 1; }
n_paths=$(printf '%s\n' "$required_paths" | wc -l | tr -d ' ')
n_hashes=$(printf '%s\n' "$lock_hashes" | wc -l | tr -d ' ')
printf '%s\n' "$required_paths" | sort -cu 2>/dev/null ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.requiredPaths 未按字典序排列或含重复" >&2; exit 1; }
[ "$n_hashes" -eq "$n_paths" ] ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.requiredPathSha256 条目数 $n_hashes ≠ requiredPaths 数 $n_paths" >&2; exit 1; }
printf '%s\n' "$lock_hashes" | sort -cu 2>/dev/null ||
    { echo "tools/verify-architecture-lock.sh: FAIL: lock.requiredPathSha256 未按字典序排列或含重复" >&2; exit 1; }
bad_paths=$(printf '%s\n' "$required_paths" | grep -v '^[A-Za-z0-9._/-]*$' || true)
[ -z "$bad_paths" ] || {
    echo "tools/verify-architecture-lock.sh: FAIL: lock.requiredPaths 含非法路径字符：" >&2
    printf '%s\n' "$bad_paths" | sed 's/^/  /' >&2
    exit 1
}

project() {
    case "$1" in
        .spec/decisions/*) printf 'decisions/%s\n' "${1#.spec/decisions/}" ;;
        schemas/* | ids/* | fixtures/* | tools/*) printf '%s\n' "$1" ;;
        *) return 1 ;;
    esac
}
unprojectable=""
for p in $required_paths; do
    project "$p" >/dev/null || unprojectable="$unprojectable $p"
done
[ -z "$unprojectable" ] || {
    echo "tools/verify-architecture-lock.sh: FAIL: lock.requiredPaths 含不可投影路径：$unprojectable" >&2
    exit 1
}

# ── 2. 镜像存在性 / 逐文件摘要 / 未登记文件 ────────────────────────────────
if [ ! -d "$mirror_root" ]; then
    echo "tools/verify-architecture-lock.sh: FAIL: 镜像目录不存在：$mirror_rel（先运行 just sync-contracts）" >&2
    exit 1
fi

missing=""
drifted=""
mirror_actual=$(cd "$mirror_root" && find . \( -type f -o -type l \) | sed 's#^\./##' | LC_ALL=C sort)
stream="$repo_root/build/.tmp/verify-architecture-lock-stream-$$"
mkdir -p "$(dirname "$stream")"
: >"$stream"
for p in $required_paths; do
    proj=$(project "$p")
    if [ ! -f "$mirror_root/$proj" ] || [ -L "$mirror_root/$proj" ]; then
        missing="$missing$proj
"
        continue
    fi
    actual=$(sha256sum "$mirror_root/$proj" | awk '{ sub(/^\\/, ""); print $1 }')
    printf '%s %s\n' "$p" "$actual" >>"$stream"
    expected=$(printf '%s\n' "$lock_hashes" | awk -v key="$p" '$1 == key { print $2 }')
    if [ "$actual" != "$expected" ]; then
        drifted="$drifted$proj（期望 $expected，实际 $actual）
"
    fi
done

expected_files=$(printf '%s\n' "$required_paths" | while IFS= read -r p; do project "$p"; done | LC_ALL=C sort)
printf '%s\n' "$expected_files" >"$stream.expected"
printf '%s\n.gitattributes\ngeneration-record.json\n' "$expected_files" | LC_ALL=C sort >"$stream.expected-all"
printf '%s\n' "$mirror_actual" >"$stream.actual"
extra=$(comm -13 "$stream.expected-all" "$stream.actual")

[ -z "$missing" ] || {
    echo "tools/verify-architecture-lock.sh: FAIL: 缺失 required path（$(printf '%s' "$missing" | wc -l | tr -d ' ') 个）：" >&2
    printf '%s\n' "$missing" | sed '/^$/d; s#^#  缺失: '"$mirror_rel"'/#' >&2
    fail=1
}
[ -z "$drifted" ] || {
    echo "tools/verify-architecture-lock.sh: FAIL: 镜像内容与 lock 漂移（$(printf '%s\n' "$drifted" | sed '/^$/d' | wc -l | tr -d ' ') 个）：" >&2
    printf '%s\n' "$drifted" | sed '/^$/d; s/^/  漂移: /' >&2
    fail=1
}
[ -z "$extra" ] || {
    echo "tools/verify-architecture-lock.sh: FAIL: 镜像中存在未登记文件/符号链接（$(printf '%s\n' "$extra" | wc -l | tr -d ' ') 个）：" >&2
    printf '%s\n' "$extra" | sed '/^$/d; s#^#  未登记: '"$mirror_rel"'/#' >&2
    fail=1
}

# ── 3. 镜像根 .gitattributes 字节冻结声明 ──────────────────────────────────
gitattr_content=$(cat "$mirror_root/.gitattributes" 2>/dev/null || true)
[ "$gitattr_content" = "* -text" ] ||
    err "镜像根 .gitattributes 必须精确为「* -text」（实际：$(printf '%s' "$gitattr_content" | head -c 80)）"

# ── 4. 生成记录 ────────────────────────────────────────────────────────────
record_path="$mirror_root/generation-record.json"
if [ ! -f "$record_path" ] || [ -L "$record_path" ]; then
    err "缺少生成记录 $descriptor_rel"
else
    record_sha=$(sha256sum "$record_path" | awk '{ sub(/^\\/, ""); print $1 }')
    [ "$record_sha" = "$lock_descriptor_sha" ] ||
        err "生成记录 SHA-256 与 lock 不一致（期望 $lock_descriptor_sha，实际 $record_sha）——镜像须由锁定工具重建"

    record_json=$(tr -d '\r' <"$record_path")
    record_str_field() {
        printf '%s\n' "$record_json" |
            sed -n "s/^[[:space:]]*\"$1\":[[:space:]]*\"\([^\"]*\)\".*$/\1/p" |
            head -n 1
    }
    record_schema_version=$(printf '%s\n' "$record_json" |
        sed -n 's/^[[:space:]]*"schemaVersion":[[:space:]]*\([0-9][0-9]*\).*$/\1/p' | head -n 1)
    record_kind=$(record_str_field kind)
    record_repository=$(record_str_field repository)
    record_commit=$(record_str_field commit)
    record_baseline=$(record_str_field architectureBaselineId)
    record_mirror_root=$(record_str_field mirrorRoot)
    record_compiler_name=$(record_str_field compilerName)
    record_compiler_version=$(printf '%s\n' "$record_json" |
        sed -n 's/^[[:space:]]*"compilerVersion":[[:space:]]*\([0-9][0-9]*\).*$/\1/p' | head -n 1)
    record_compiler_sha=$(record_str_field compilerSha256)
    record_argv=$(record_str_field argv)
    record_target=$(record_str_field targetPlatform)
    record_file_count=$(printf '%s\n' "$record_json" |
        sed -n 's/^[[:space:]]*"fileCount":[[:space:]]*\([0-9][0-9]*\).*$/\1/p' | head -n 1)
    record_input_hash=$(record_str_field inputHash)
    record_output_hash=$(record_str_field outputHash)

    [ "$record_schema_version" = "$expected_schema_version" ] || err "生成记录 schemaVersion 非 $expected_schema_version"
    [ "$record_kind" = "$expected_record_kind" ] || err "生成记录 kind 非 $expected_record_kind"
    [ "$record_repository" = "$expected_repository" ] || err "生成记录 repository 与 lock 不一致"
    [ "$record_commit" = "$expected_commit" ] || err "生成记录 commit 与 lock 不一致"
    [ "$record_baseline" = "$expected_baseline" ] || err "生成记录 architectureBaselineId 与 lock 不一致"
    [ "$record_mirror_root" = "$mirror_rel" ] || err "生成记录 mirrorRoot 非 $mirror_rel"
    [ "$record_compiler_name" = "$compiler_rel" ] || err "生成记录 compilerName 非 $compiler_rel"
    [ "$record_compiler_version" = "$expected_compiler_version" ] || err "生成记录 compilerVersion 非 $expected_compiler_version"
    [ "$record_argv" = "$expected_record_argv" ] || err "生成记录 argv 非 $expected_record_argv"
    [ "$record_target" = "$expected_record_target_platform" ] || err "生成记录 targetPlatform 非 $expected_record_target_platform"
    [ "$record_file_count" = "$n_paths" ] || err "生成记录 fileCount=$record_file_count 与 lock requiredPaths 数 $n_paths 不一致"
    is_sha256 "$record_input_hash" || err "生成记录 inputHash 不是 64 位小写十六进制"
    is_sha256 "$record_output_hash" || err "生成记录 outputHash 不是 64 位小写十六进制"

    # 编译器摘要：对 CRLF 归一化后的脚本文本计算（脚本受根 .gitattributes text=auto 影响，
    # 不同平台检出行尾不同；归一化保证跨平台一致）。
    compiler_sha=$(tr -d '\r' <"$repo_root/$compiler_rel" | sha256sum | awk '{ print $1 }')
    [ "$record_compiler_sha" = "$compiler_sha" ] ||
        err "compilerSha256 与 $compiler_rel 当前内容不一致（工具变更后必须显式 --update-lock 重生成 lock，单独 PR）"

    # pathSha256 与 lock 完全相等。
    record_hashes=$(printf '%s\n' "$record_json" | awk '
        state == 0 && /"pathSha256"[[:space:]]*:/ { state = 1; next }
        state == 1 && /^[[:space:]]*\}/ { state = 2; next }
        state == 1 && match($0, /"[^"]+"[[:space:]]*:[[:space:]]*"[^"]+"/) {
            kv = substr($0, RSTART, RLENGTH)
            split(kv, part, /[[:space:]]*:[[:space:]]*/)
            gsub(/"/, "", part[1]); gsub(/"/, "", part[2])
            print part[1] " " part[2]
        }' | LC_ALL=C sort)
    lock_hashes_sorted=$(printf '%s\n' "$lock_hashes" | LC_ALL=C sort)
    [ "$record_hashes" = "$lock_hashes_sorted" ] ||
        err "生成记录 pathSha256 与 lock.requiredPathSha256 不一致"

    # Input/Output Hash 从镜像字节重算（镜像为原样复制，两者应相等且与记录一致）。
    if [ -z "$missing" ]; then
        recomputed=$(LC_ALL=C sort "$stream" | sha256sum | awk '{ print $1 }')
        [ "$recomputed" = "$record_input_hash" ] || err "Input Hash 重算不一致（期望 $record_input_hash，实际 $recomputed）"
        [ "$recomputed" = "$record_output_hash" ] || err "Output Hash 重算不一致（期望 $record_output_hash，实际 $recomputed）"
    fi
fi

rm -f "$stream" "$stream.expected" "$stream.expected-all" "$stream.actual" 2>/dev/null || :

# ── 5. 只读权限（通告项：git 不传输权限位；重新检出后运行 sync-contracts 恢复） ──
writable_count=0
file_total=0
while IFS= read -r f; do
    [ -n "$f" ] || continue
    file_total=$((file_total + 1))
    if [ -w "$mirror_root/$f" ]; then
        writable_count=$((writable_count + 1))
    fi
done <<EOF
$mirror_actual
EOF

# ── 汇总 ───────────────────────────────────────────────────────────────────
if [ "$fail" -ne 0 ]; then
    echo "tools/verify-architecture-lock.sh: FAIL（详见上方逐条清单；输入仅为 architecture.lock.json 与 $mirror_rel，未访问架构源仓与 docs/architecture/）" >&2
    exit 1
fi
echo "lock: $expected_repository @ $expected_commit（$expected_baseline；required paths $n_paths，逐文件 SHA-256 全部一致）"
echo "mirror: $mirror_rel（文件 $file_total 个 = required $n_paths + generation-record.json + .gitattributes；无缺失/漂移/未登记）"
echo "record: $descriptor_rel SHA-256 与 lock 一致；Input Hash == Output Hash == $record_input_hash；compilerSha256 与 $compiler_rel 一致"
echo "input-scope: 仅 architecture.lock.json 与只读镜像；未使用 docs/architecture/ 或架构源仓作为输入"
echo "read-only: $writable_count/$file_total 可写（git 不传输权限位；非 0 时运行 just sync-contracts 恢复只读）"
echo "tools/verify-architecture-lock.sh: OK"
