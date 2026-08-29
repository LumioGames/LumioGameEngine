/*
 * C 侧 ABI 布局探针（规格 §8.2「生成物/跨语言测试」）。
 *
 * 真值来源只有一个：架构源生成的 include/lumio_core.h。本文件**不复制 Golden 数值**
 * ——header 自带的 LUMIO_STATIC_ASSERT 行就是 C 侧的 Golden 判据，布局不符则编译失败
 * （ADR-040：a mismatch is a build failure, never a runtime discovery）。抄第二份
 * 只会制造第二处真值。
 *
 * 本文件只做两件 header 没做的事：
 *   1) 补 header 未覆盖的编译期断言——_Alignof 与 Root header 三字段偏移；
 *   2) 把测量值按规范格式打印，供与 Rust / C# 两侧的同格式输出逐行比对。
 *
 * 构建运行：
 *   cc -std=c11 -Wall -Wextra -Werror \
 *      -I modules/root-abi/generated/LGE-V1.4-2026-08-27/include \
 *      -o <out> modules/root-abi/tests/c/header_layout.c && <out>
 */

#include <stdio.h>
#include <string.h>

#include <lumio_core.h>

/*
 * header 只断言 size 与 slot offset。以下两类同属 Golden 但 header 未覆盖，在这里补。
 * 对齐基准 8 取自锁定 layout profile linux-x86_64-glibc 的 maxAlignment
 * （ADR-040 §4；同值发布在 reports/layout-report.json 与 C#/Rust 绑定的
 * MaxAlignment / MAX_ALIGNMENT 常量，唯独 C header 没有对应宏）。
 */
LUMIO_STATIC_ASSERT(_Alignof(lumio_handle_t) == 8, handle_align);
LUMIO_STATIC_ASSERT(_Alignof(lumio_buffer_t) == 8, buffer_align);
LUMIO_STATIC_ASSERT(_Alignof(lumio_core_api) == 8, core_api_align);
LUMIO_STATIC_ASSERT(_Alignof(lumio_voxel_api) == 8, voxel_api_align);
LUMIO_STATIC_ASSERT(_Alignof(lumio_root_api) == 8, root_api_align);

/* ADR-040 §4 冻结的 Root header：abi_version @0、struct_size @4、capability_bits @8。 */
LUMIO_STATIC_ASSERT(offsetof(lumio_root_api, abi_version) == 0, root_abi_version_offset);
LUMIO_STATIC_ASSERT(offsetof(lumio_root_api, struct_size) == 4, root_struct_size_offset);
LUMIO_STATIC_ASSERT(offsetof(lumio_root_api, capability_bits) == 8, root_capability_bits_offset);

/* API table header 在两张表上必须同构（ADR-040 §4）。 */
LUMIO_STATIC_ASSERT(offsetof(lumio_core_api, version) == offsetof(lumio_voxel_api, version),
                    table_version_offset);
LUMIO_STATIC_ASSERT(offsetof(lumio_core_api, struct_size) == offsetof(lumio_voxel_api, struct_size),
                    table_struct_size_offset);

static void report_size(const char *name, size_t measured)
{
    printf("size.%s=%zu\n", name, measured);
}

static void report_align(const char *name, size_t measured)
{
    printf("align.%s=%zu\n", name, measured);
}

static void report_offset(const char *table, const char *slot, size_t measured)
{
    printf("offset.%s.%s=%zu\n", table, slot, measured);
}

int main(void)
{
    printf("abi_version=%u\n", (unsigned)LUMIO_ABI_VERSION);
    printf("capability_bits=%llu\n", (unsigned long long)LUMIO_CAPABILITY_BITS);
    printf("entry_symbol=%s\n", LUMIO_ENTRY_SYMBOL);
    printf("symbol_prefix=%s\n", LUMIO_SYMBOL_PREFIX);
    printf("pointer_bytes=%zu\n", sizeof(void *));

    report_size("lumio_handle_t", sizeof(lumio_handle_t));
    report_size("lumio_buffer_t", sizeof(lumio_buffer_t));
    report_size("lumio_core_api", sizeof(lumio_core_api));
    report_size("lumio_voxel_api", sizeof(lumio_voxel_api));
    report_size("lumio_root_api", sizeof(lumio_root_api));

    report_align("lumio_handle_t", _Alignof(lumio_handle_t));
    report_align("lumio_buffer_t", _Alignof(lumio_buffer_t));
    report_align("lumio_core_api", _Alignof(lumio_core_api));
    report_align("lumio_voxel_api", _Alignof(lumio_voxel_api));
    report_align("lumio_root_api", _Alignof(lumio_root_api));

    report_offset("lumio_core_api", "lumio_core_init", offsetof(lumio_core_api, lumio_core_init));
    report_offset("lumio_core_api", "lumio_core_shutdown",
                  offsetof(lumio_core_api, lumio_core_shutdown));
    report_offset("lumio_core_api", "lumio_core_last_error_detail",
                  offsetof(lumio_core_api, lumio_core_last_error_detail));
    report_offset("lumio_voxel_api", "lumio_voxel_world_create",
                  offsetof(lumio_voxel_api, lumio_voxel_world_create));
    report_offset("lumio_voxel_api", "lumio_voxel_world_destroy",
                  offsetof(lumio_voxel_api, lumio_voxel_world_destroy));
    report_offset("lumio_root_api", "lumio_core_api", offsetof(lumio_root_api, lumio_core_api));
    report_offset("lumio_root_api", "lumio_voxel_api", offsetof(lumio_root_api, lumio_voxel_api));

    /* entry symbol 是 header 发布的字符串宏，不复制第二份字面量做比较。 */
    if (strlen(LUMIO_ENTRY_SYMBOL) == 0) {
        fprintf(stderr, "FAIL: header 发布的 entry symbol 为空\n");
        return 1;
    }
    return 0;
}
