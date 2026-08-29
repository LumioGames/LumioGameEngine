/* Generated Root ABI header. Do not hand-edit. */
/* Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27. */
/* Compiler: lumio-abi-compiler 1.0.0. ADR-040. */
/* Layout profile: linux-x86_64-glibc (pointer 8 bytes, max align 8). */

#ifndef LUMIO_CORE_H
#define LUMIO_CORE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LUMIO_ABI_VERSION 1
#define LUMIO_ENTRY_SYMBOL "lumio_core_get_api_v1"
#define LUMIO_SYMBOL_PREFIX "lumio_"
#define LUMIO_CAPABILITY_BITS 7u

typedef int32_t lumio_status_t;

typedef struct lumio_handle_t {
    uint32_t index;
    uint32_t generation;
    uint64_t context;
} lumio_handle_t;

typedef struct lumio_buffer_t {
    void* ptr;
    uint64_t len;
    uint64_t capacity;
} lumio_buffer_t;

/* Opaque caller-owned payloads; bodies are guarded by their own struct_size. */
struct lumio_core_config_v1;
struct lumio_voxel_world_desc_v1;

typedef struct lumio_core_api {
    uint32_t version;
    uint32_t struct_size;
    uint64_t reserved0;
    lumio_status_t (*lumio_core_init)(const struct lumio_core_config_v1* config, lumio_handle_t out_context);
    lumio_status_t (*lumio_core_shutdown)(lumio_handle_t context);
    lumio_status_t (*lumio_core_last_error_detail)(lumio_handle_t context, lumio_buffer_t out_detail);
    void* reserved[1];
} lumio_core_api;

typedef struct lumio_voxel_api {
    uint32_t version;
    uint32_t struct_size;
    uint64_t reserved0;
    lumio_status_t (*lumio_voxel_world_create)(lumio_handle_t context, const struct lumio_voxel_world_desc_v1* desc, lumio_handle_t out_world);
    lumio_status_t (*lumio_voxel_world_destroy)(lumio_handle_t world);
} lumio_voxel_api;

typedef struct lumio_root_api {
    uint32_t abi_version;
    uint32_t struct_size;
    uint64_t capability_bits;
    const lumio_core_api* lumio_core_api;
    const lumio_voxel_api* lumio_voxel_api;
    unsigned char reserved_tail[32];
} lumio_root_api;

lumio_status_t lumio_core_get_api_v1(uint32_t requested_version, const lumio_root_api** out_table);

/* Layout Golden assertions: a mismatch is a build failure, never a runtime discovery. */
#define LUMIO_STATIC_ASSERT(cond, tag) typedef char lumio_assert_##tag[(cond) ? 1 : -1]
LUMIO_STATIC_ASSERT(sizeof(lumio_handle_t) == 16, handle_size);
LUMIO_STATIC_ASSERT(sizeof(lumio_buffer_t) == 24, buffer_size);
LUMIO_STATIC_ASSERT(sizeof(lumio_status_t) == 4, status_size);
LUMIO_STATIC_ASSERT(sizeof(void*) == 8, pointer_size);
LUMIO_STATIC_ASSERT(sizeof(lumio_core_api) == 48, lumio_core_api_size);
LUMIO_STATIC_ASSERT(offsetof(lumio_core_api, lumio_core_init) == 16, lumio_core_init_offset);
LUMIO_STATIC_ASSERT(offsetof(lumio_core_api, lumio_core_shutdown) == 24, lumio_core_shutdown_offset);
LUMIO_STATIC_ASSERT(offsetof(lumio_core_api, lumio_core_last_error_detail) == 32, lumio_core_last_error_detail_offset);
LUMIO_STATIC_ASSERT(sizeof(lumio_voxel_api) == 32, lumio_voxel_api_size);
LUMIO_STATIC_ASSERT(offsetof(lumio_voxel_api, lumio_voxel_world_create) == 16, lumio_voxel_world_create_offset);
LUMIO_STATIC_ASSERT(offsetof(lumio_voxel_api, lumio_voxel_world_destroy) == 24, lumio_voxel_world_destroy_offset);
LUMIO_STATIC_ASSERT(sizeof(lumio_root_api) == 64, root_size);
LUMIO_STATIC_ASSERT(offsetof(lumio_root_api, lumio_core_api) == 16, root_lumio_core_api_offset);
LUMIO_STATIC_ASSERT(offsetof(lumio_root_api, lumio_voxel_api) == 24, root_lumio_voxel_api_offset);

#ifdef __cplusplus
}
#endif

#endif /* LUMIO_CORE_H */
