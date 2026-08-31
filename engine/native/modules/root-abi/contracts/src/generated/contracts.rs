//! ContractTypes 消费面（packages/abi）：Root ABI bundle 与 C Header 的
//! 逐字节嵌入 + bundle 发布值的标量视图。Rust 侧结构体绑定（repr(C) 类型）
//! 属上游 LanguageBinding 生成包，其 consumers 不含本仓，故本文件不含任何
//! repr(C)/extern "C" 定义（seam，见 crate 文档）。
// 本文件由锁定生成器从只读架构镜像派生——禁止手改（rules/system.md 生成物纪律）。
// 重生成：LUMIO_CONTRACTS_REGENERATE=1 cargo test -p lumio-core-contracts --locked --test generated_integrity
// 派生输入与逐文件摘要见 modules/root-abi/contracts/generated-contract-artifact.json。

#[rustfmt::skip]
pub const BUNDLE_ID: &str = "root-abi-v1";
#[rustfmt::skip]
pub const ENTRY_SYMBOL: &str = "lumio_core_get_api_v1";
#[rustfmt::skip]
pub const SYMBOL_PREFIX: &str = "lumio_";
#[rustfmt::skip]
pub const CALLING_CONVENTION: &str = "C";
#[rustfmt::skip]
pub const ENDIANNESS: &str = "Little";
#[rustfmt::skip]
pub const LAYOUT_PROFILE_ID: &str = "linux-x86_64-glibc";
#[rustfmt::skip]
pub const LAYOUT_OS: &str = "LinuxServer";
#[rustfmt::skip]
pub const LAYOUT_ARCH: &str = "x86_64";
#[rustfmt::skip]
pub const LAYOUT_ABI_RUNTIME: &str = "glibc";
#[rustfmt::skip]
pub const COMPILER_NAME: &str = "lumio-abi-compiler";
#[rustfmt::skip]
pub const COMPILER_VERSION: &str = "1.0.0";
#[rustfmt::skip]
pub const COMPILER_SHA256_HEX: &str = "217437fd4755e1a339e2029838cc4a2d2fb305fa05520c8cfd10ea98cc2ff290";
#[rustfmt::skip]
pub const UPSTREAM_INPUT_HASH_HEX: &str = "696a58d0525b897b549dd1e432166ae1020835902a5984221a8e60d5d8285bb3";
#[rustfmt::skip]
pub const ABI_VERSION: u32 = 1;
#[rustfmt::skip]
pub const POINTER_WIDTH_BITS: u32 = 64;
#[rustfmt::skip]
pub const POINTER_BYTES: u32 = 8;
#[rustfmt::skip]
pub const MAX_ALIGNMENT: u32 = 8;
#[rustfmt::skip]
pub const ROOT_HEADER_BYTES: u32 = 16;
#[rustfmt::skip]
pub const TABLE_HEADER_BYTES: u32 = 16;
#[rustfmt::skip]
pub const ROOT_DECLARED_STRUCT_SIZE: u32 = 64;
#[rustfmt::skip]
pub const ROOT_MINIMUM_STRUCT_SIZE: u32 = 32;
#[rustfmt::skip]
pub const STATUS_SIZE_BYTES: u32 = 4;
#[rustfmt::skip]
pub const HANDLE_SIZE_BYTES: u32 = 16;
#[rustfmt::skip]
pub const HANDLE_ALIGN_BYTES: u32 = 8;
#[rustfmt::skip]
pub const BUFFER_SIZE_BYTES: u32 = 24;
#[rustfmt::skip]
pub const BUFFER_ALIGN_BYTES: u32 = 8;
#[rustfmt::skip]
pub const CAPABILITY_BITS: u64 = 7;

#[rustfmt::skip]
pub const ROOT_ABI_BUNDLE_SHA256_HEX: &str = "03ca75361fed3ca95f8efd55af2e311ea8300b2635b590ae6d46394d58bc6a39";
#[rustfmt::skip]
pub const ROOT_ABI_BUNDLE_JSON: &[u8] = b"{\"abi\":{\"abiVersion\":1,\"callingConvention\":\"C\",\"capabilityBits\":7,\"endianness\":\"Little\",\"entrySymbol\":\"lumio_core_get_api_v1\",\"pointerWidth\":64,\"symbolPrefix\":\"lumio_\"},\"baselineId\":\"LGE-V1.4-2026-08-27\",\"bundleId\":\"root-abi-v1\",\"compiler\":{\"digest\":\"217437fd4755e1a339e2029838cc4a2d2fb305fa05520c8cfd10ea98cc2ff290\",\"name\":\"lumio-abi-compiler\",\"version\":\"1.0.0\"},\"inputHash\":\"696a58d0525b897b549dd1e432166ae1020835902a5984221a8e60d5d8285bb3\",\"inputSet\":[\"schemas/native-managed-abi.schema.json\",\"fixtures/valid/native-managed-abi.json\"],\"layoutProfile\":{\"abiRuntime\":\"glibc\",\"arch\":\"x86_64\",\"maxAlignment\":8,\"os\":\"LinuxServer\",\"pointerBytes\":8,\"rootHeaderBytes\":16,\"tableHeaderBytes\":16,\"targetProfileId\":\"linux-x86_64-glibc\"},\"outputFiles\":[{\"digest\":\"040451bbde5a4dec3726be5f5a7be4bb934c3f68a1ca87f9c55559cae738efc7\",\"path\":\"abi/lumio_core.h\",\"role\":\"CHeader\"},{\"digest\":\"5e81bdfb6e879d849e2cb77a847a07167e5a459f2f23fd43f07609e726043bec\",\"path\":\"rust/lumio-gen-language-binding/src/root_abi.rs\",\"role\":\"RustBinding\"},{\"digest\":\"d89ff35434438773055ce4108b9f04ef6ff2b42335101249163f65c734975cd1\",\"path\":\"csharp/Lumio.Gen.LanguageBinding/RootAbi.cs\",\"role\":\"CSharpBinding\"}],\"root\":{\"declaredStructSize\":64,\"fields\":[{\"c\":\"uint32_t\",\"csharp\":\"uint\",\"name\":\"abi_version\",\"offset\":0,\"rust\":\"u32\",\"size\":4},{\"c\":\"uint32_t\",\"csharp\":\"uint\",\"name\":\"struct_size\",\"offset\":4,\"rust\":\"u32\",\"size\":4},{\"c\":\"uint64_t\",\"csharp\":\"ulong\",\"name\":\"capability_bits\",\"offset\":8,\"rust\":\"u64\",\"size\":8}],\"minimumStructSize\":32,\"tables\":[{\"name\":\"lumio_core_api\",\"offset\":16},{\"name\":\"lumio_voxel_api\",\"offset\":24}]},\"schemaEpoch\":1,\"tables\":[{\"declaredStructSize\":48,\"fields\":[{\"c\":\"uint32_t\",\"csharp\":\"uint\",\"name\":\"version\",\"offset\":0,\"rust\":\"u32\",\"size\":4},{\"c\":\"uint32_t\",\"csharp\":\"uint\",\"name\":\"struct_size\",\"offset\":4,\"rust\":\"u32\",\"size\":4},{\"c\":\"uint64_t\",\"csharp\":\"ulong\",\"name\":\"reserved0\",\"offset\":8,\"rust\":\"u64\",\"size\":8}],\"functionCount\":3,\"minimumStructSize\":48,\"name\":\"lumio_core_api\",\"reservedSlots\":1,\"slots\":[{\"cSignature\":\"lumio_status_t (*lumio_core_init)(const struct lumio_core_config_v1* config, lumio_handle_t out_context)\",\"csharpSignature\":\"LumioStatus lumio_core_init(IntPtr config, LumioHandle out_context)\",\"name\":\"lumio_core_init\",\"offset\":16,\"params\":[{\"name\":\"config\",\"typeRef\":\"struct:core_config:v1\"},{\"name\":\"out_context\",\"typeRef\":\"handle:core_context\"}],\"returns\":\"status\",\"rustSignature\":\"extern \\\"C\\\" fn(config: *const LumioCoreConfigV1, out_context: LumioHandle) -> LumioStatus\",\"slotIndex\":0},{\"cSignature\":\"lumio_status_t (*lumio_core_shutdown)(lumio_handle_t context)\",\"csharpSignature\":\"LumioStatus lumio_core_shutdown(LumioHandle context)\",\"name\":\"lumio_core_shutdown\",\"offset\":24,\"params\":[{\"name\":\"context\",\"typeRef\":\"handle:core_context\"}],\"returns\":\"status\",\"rustSignature\":\"extern \\\"C\\\" fn(context: LumioHandle) -> LumioStatus\",\"slotIndex\":1},{\"cSignature\":\"lumio_status_t (*lumio_core_last_error_detail)(lumio_handle_t context, lumio_buffer_t out_detail)\",\"csharpSignature\":\"LumioStatus lumio_core_last_error_detail(LumioHandle context, LumioBuffer out_detail)\",\"name\":\"lumio_core_last_error_detail\",\"offset\":32,\"params\":[{\"name\":\"context\",\"typeRef\":\"handle:core_context\"},{\"name\":\"out_detail\",\"typeRef\":\"buffer:out\"}],\"returns\":\"status\",\"rustSignature\":\"extern \\\"C\\\" fn(context: LumioHandle, out_detail: LumioBuffer) -> LumioStatus\",\"slotIndex\":2}],\"version\":1},{\"declaredStructSize\":32,\"fields\":[{\"c\":\"uint32_t\",\"csharp\":\"uint\",\"name\":\"version\",\"offset\":0,\"rust\":\"u32\",\"size\":4},{\"c\":\"uint32_t\",\"csharp\":\"uint\",\"name\":\"struct_size\",\"offset\":4,\"rust\":\"u32\",\"size\":4},{\"c\":\"uint64_t\",\"csharp\":\"ulong\",\"name\":\"reserved0\",\"offset\":8,\"rust\":\"u64\",\"size\":8}],\"functionCount\":2,\"minimumStructSize\":32,\"name\":\"lumio_voxel_api\",\"reservedSlots\":0,\"slots\":[{\"cSignature\":\"lumio_status_t (*lumio_voxel_world_create)(lumio_handle_t context, const struct lumio_voxel_world_desc_v1* desc, lumio_handle_t out_world)\",\"csharpSignature\":\"LumioStatus lumio_voxel_world_create(LumioHandle context, IntPtr desc, LumioHandle out_world)\",\"name\":\"lumio_voxel_world_create\",\"offset\":16,\"params\":[{\"name\":\"context\",\"typeRef\":\"handle:core_context\"},{\"name\":\"desc\",\"typeRef\":\"struct:voxel_world_desc:v1\"},{\"name\":\"out_world\",\"typeRef\":\"handle:voxel_world\"}],\"returns\":\"status\",\"rustSignature\":\"extern \\\"C\\\" fn(context: LumioHandle, desc: *const LumioVoxelWorldDescV1, out_world: LumioHandle) -> LumioStatus\",\"slotIndex\":0},{\"cSignature\":\"lumio_status_t (*lumio_voxel_world_destroy)(lumio_handle_t world)\",\"csharpSignature\":\"LumioStatus lumio_voxel_world_destroy(LumioHandle world)\",\"name\":\"lumio_voxel_world_destroy\",\"offset\":24,\"params\":[{\"name\":\"world\",\"typeRef\":\"handle:voxel_world\"}],\"returns\":\"status\",\"rustSignature\":\"extern \\\"C\\\" fn(world: LumioHandle) -> LumioStatus\",\"slotIndex\":1}],\"version\":1}],\"typeMapping\":[{\"align\":1,\"c\":\"uint8_t\",\"csharp\":\"byte\",\"rust\":\"u8\",\"size\":1,\"typeRef\":\"u8\"},{\"align\":2,\"c\":\"uint16_t\",\"csharp\":\"ushort\",\"rust\":\"u16\",\"size\":2,\"typeRef\":\"u16\"},{\"align\":4,\"c\":\"uint32_t\",\"csharp\":\"uint\",\"rust\":\"u32\",\"size\":4,\"typeRef\":\"u32\"},{\"align\":8,\"c\":\"uint64_t\",\"csharp\":\"ulong\",\"rust\":\"u64\",\"size\":8,\"typeRef\":\"u64\"},{\"align\":1,\"c\":\"int8_t\",\"csharp\":\"sbyte\",\"rust\":\"i8\",\"size\":1,\"typeRef\":\"i8\"},{\"align\":2,\"c\":\"int16_t\",\"csharp\":\"short\",\"rust\":\"i16\",\"size\":2,\"typeRef\":\"i16\"},{\"align\":4,\"c\":\"int32_t\",\"csharp\":\"int\",\"rust\":\"i32\",\"size\":4,\"typeRef\":\"i32\"},{\"align\":8,\"c\":\"int64_t\",\"csharp\":\"long\",\"rust\":\"i64\",\"size\":8,\"typeRef\":\"i64\"},{\"align\":4,\"c\":\"float\",\"csharp\":\"float\",\"rust\":\"f32\",\"size\":4,\"typeRef\":\"f32\"},{\"align\":8,\"c\":\"double\",\"csharp\":\"double\",\"rust\":\"f64\",\"size\":8,\"typeRef\":\"f64\"},{\"align\":4,\"c\":\"uint32_t\",\"csharp\":\"uint\",\"rust\":\"u32\",\"size\":4,\"typeRef\":\"bool32\"},{\"align\":4,\"c\":\"lumio_status_t\",\"csharp\":\"LumioStatus\",\"rust\":\"LumioStatus\",\"size\":4,\"typeRef\":\"status\"},{\"align\":8,\"c\":\"lumio_handle_t\",\"csharp\":\"LumioHandle\",\"rust\":\"LumioHandle\",\"size\":16,\"typeRef\":\"handle:<kind>\"},{\"align\":8,\"c\":\"lumio_buffer_t\",\"csharp\":\"LumioBuffer\",\"rust\":\"LumioBuffer\",\"size\":24,\"typeRef\":\"buffer:in\"},{\"align\":8,\"c\":\"lumio_buffer_t\",\"csharp\":\"LumioBuffer\",\"rust\":\"LumioBuffer\",\"size\":24,\"typeRef\":\"buffer:out\"},{\"align\":8,\"c\":\"lumio_buffer_t\",\"csharp\":\"LumioBuffer\",\"rust\":\"LumioBuffer\",\"size\":24,\"typeRef\":\"buffer:inout\"},{\"align\":8,\"c\":\"const lumio_<name>_v<N>*\",\"csharp\":\"IntPtr\",\"rust\":\"*const Lumio<Name>V<N>\",\"size\":8,\"typeRef\":\"struct:<name>:v<N>\"},{\"align\":8,\"c\":\"const lumio_<name>*\",\"csharp\":\"IntPtr\",\"rust\":\"*const Lumio<Name>\",\"size\":8,\"typeRef\":\"ptr:const:<name>\"},{\"align\":8,\"c\":\"lumio_<name>*\",\"csharp\":\"IntPtr\",\"rust\":\"*mut Lumio<Name>\",\"size\":8,\"typeRef\":\"ptr:mut:<name>\"}]}\n";
#[rustfmt::skip]
pub const LUMIO_CORE_H_SHA256_HEX: &str = "040451bbde5a4dec3726be5f5a7be4bb934c3f68a1ca87f9c55559cae738efc7";
#[rustfmt::skip]
pub const LUMIO_CORE_H: &[u8] = b"/* Generated Root ABI header. Do not hand-edit. */\n/* Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27. */\n/* Compiler: lumio-abi-compiler 1.0.0. ADR-040. */\n/* Layout profile: linux-x86_64-glibc (pointer 8 bytes, max align 8). */\n\n#ifndef LUMIO_CORE_H\n#define LUMIO_CORE_H\n\n#include <stdint.h>\n#include <stddef.h>\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n#define LUMIO_ABI_VERSION 1\n#define LUMIO_ENTRY_SYMBOL \"lumio_core_get_api_v1\"\n#define LUMIO_SYMBOL_PREFIX \"lumio_\"\n#define LUMIO_CAPABILITY_BITS 7u\n\ntypedef int32_t lumio_status_t;\n\ntypedef struct lumio_handle_t {\n    uint32_t index;\n    uint32_t generation;\n    uint64_t context;\n} lumio_handle_t;\n\ntypedef struct lumio_buffer_t {\n    void* ptr;\n    uint64_t len;\n    uint64_t capacity;\n} lumio_buffer_t;\n\n/* Opaque caller-owned payloads; bodies are guarded by their own struct_size. */\nstruct lumio_core_config_v1;\nstruct lumio_voxel_world_desc_v1;\n\ntypedef struct lumio_core_api {\n    uint32_t version;\n    uint32_t struct_size;\n    uint64_t reserved0;\n    lumio_status_t (*lumio_core_init)(const struct lumio_core_config_v1* config, lumio_handle_t out_context);\n    lumio_status_t (*lumio_core_shutdown)(lumio_handle_t context);\n    lumio_status_t (*lumio_core_last_error_detail)(lumio_handle_t context, lumio_buffer_t out_detail);\n    void* reserved[1];\n} lumio_core_api;\n\ntypedef struct lumio_voxel_api {\n    uint32_t version;\n    uint32_t struct_size;\n    uint64_t reserved0;\n    lumio_status_t (*lumio_voxel_world_create)(lumio_handle_t context, const struct lumio_voxel_world_desc_v1* desc, lumio_handle_t out_world);\n    lumio_status_t (*lumio_voxel_world_destroy)(lumio_handle_t world);\n} lumio_voxel_api;\n\ntypedef struct lumio_root_api {\n    uint32_t abi_version;\n    uint32_t struct_size;\n    uint64_t capability_bits;\n    const lumio_core_api* lumio_core_api;\n    const lumio_voxel_api* lumio_voxel_api;\n    unsigned char reserved_tail[32];\n} lumio_root_api;\n\nlumio_status_t lumio_core_get_api_v1(uint32_t requested_version, const lumio_root_api** out_table);\n\n/* Layout Golden assertions: a mismatch is a build failure, never a runtime discovery. */\n#define LUMIO_STATIC_ASSERT(cond, tag) typedef char lumio_assert_##tag[(cond) ? 1 : -1]\nLUMIO_STATIC_ASSERT(sizeof(lumio_handle_t) == 16, handle_size);\nLUMIO_STATIC_ASSERT(sizeof(lumio_buffer_t) == 24, buffer_size);\nLUMIO_STATIC_ASSERT(sizeof(lumio_status_t) == 4, status_size);\nLUMIO_STATIC_ASSERT(sizeof(void*) == 8, pointer_size);\nLUMIO_STATIC_ASSERT(sizeof(lumio_core_api) == 48, lumio_core_api_size);\nLUMIO_STATIC_ASSERT(offsetof(lumio_core_api, lumio_core_init) == 16, lumio_core_init_offset);\nLUMIO_STATIC_ASSERT(offsetof(lumio_core_api, lumio_core_shutdown) == 24, lumio_core_shutdown_offset);\nLUMIO_STATIC_ASSERT(offsetof(lumio_core_api, lumio_core_last_error_detail) == 32, lumio_core_last_error_detail_offset);\nLUMIO_STATIC_ASSERT(sizeof(lumio_voxel_api) == 32, lumio_voxel_api_size);\nLUMIO_STATIC_ASSERT(offsetof(lumio_voxel_api, lumio_voxel_world_create) == 16, lumio_voxel_world_create_offset);\nLUMIO_STATIC_ASSERT(offsetof(lumio_voxel_api, lumio_voxel_world_destroy) == 24, lumio_voxel_world_destroy_offset);\nLUMIO_STATIC_ASSERT(sizeof(lumio_root_api) == 64, root_size);\nLUMIO_STATIC_ASSERT(offsetof(lumio_root_api, lumio_core_api) == 16, root_lumio_core_api_offset);\nLUMIO_STATIC_ASSERT(offsetof(lumio_root_api, lumio_voxel_api) == 24, root_lumio_voxel_api_offset);\n\n#ifdef __cplusplus\n}\n#endif\n\n#endif /* LUMIO_CORE_H */\n";
