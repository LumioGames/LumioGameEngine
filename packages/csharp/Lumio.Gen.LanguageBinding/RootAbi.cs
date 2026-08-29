// Generated Root ABI binding. Do not hand-edit.
// Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27. ADR-040.
// Pure managed layout description; the consumer binds the entry symbol itself.
using System;
using System.Runtime.InteropServices;

namespace Lumio.Gen.LanguageBinding
{
    public readonly struct StructSize
    {
        public StructSize(string name, int size)
        {
            Name = name; Size = size;
        }
        public string Name { get; }
        public int Size { get; }
    }

    public static class RootAbi
    {
        public const uint AbiVersion = 1;
        public const string EntrySymbol = "lumio_core_get_api_v1";
        public const string SymbolPrefix = "lumio_";
        public const string CallingConvention = "C";
        public const ulong CapabilityBits = 7;
        public const string TargetProfileId = "linux-x86_64-glibc";
        public const int PointerBytes = 8;
        public const int MaxAlignment = 8;
        public const int RootHeaderBytes = 16;
        public const int TableHeaderBytes = 16;
    }

    public enum LumioStatus : int { Ok = 0 }

    [StructLayout(LayoutKind.Sequential)]
    public struct LumioHandle
    {
        public uint Index;
        public uint Generation;
        public ulong Context;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LumioBuffer
    {
        public IntPtr Ptr;
        public ulong Len;
        public ulong Capacity;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LumioCoreApi
    {
        public uint Version;
        public uint StructSize;
        public ulong Reserved0;
        // LumioStatus lumio_core_init(IntPtr config, LumioHandle out_context)
        public IntPtr LumioCoreInit;
        // LumioStatus lumio_core_shutdown(LumioHandle context)
        public IntPtr LumioCoreShutdown;
        // LumioStatus lumio_core_last_error_detail(LumioHandle context, LumioBuffer out_detail)
        public IntPtr LumioCoreLastErrorDetail;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 1)]
        public IntPtr[] Reserved;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LumioVoxelApi
    {
        public uint Version;
        public uint StructSize;
        public ulong Reserved0;
        // LumioStatus lumio_voxel_world_create(LumioHandle context, IntPtr desc, LumioHandle out_world)
        public IntPtr LumioVoxelWorldCreate;
        // LumioStatus lumio_voxel_world_destroy(LumioHandle world)
        public IntPtr LumioVoxelWorldDestroy;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LumioRootApi
    {
        public uint AbiVersion;
        public uint StructSize;
        public ulong CapabilityBits;
        public IntPtr LumioCoreApi;
        public IntPtr LumioVoxelApi;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 32)]
        public byte[] ReservedTail;
    }

    public readonly struct SlotOffset
    {
        public SlotOffset(string table, string slot, int offset)
        {
            Table = table; Slot = slot; Offset = offset;
        }
        public string Table { get; }
        public string Slot { get; }
        public int Offset { get; }
    }
    public static class RootAbiLayout
    {
        public static readonly SlotOffset[] SlotOffsets =
        {
            new SlotOffset("lumio_core_api", "lumio_core_init", 16),
            new SlotOffset("lumio_core_api", "lumio_core_shutdown", 24),
            new SlotOffset("lumio_core_api", "lumio_core_last_error_detail", 32),
            new SlotOffset("lumio_voxel_api", "lumio_voxel_world_create", 16),
            new SlotOffset("lumio_voxel_api", "lumio_voxel_world_destroy", 24),
        };

        public static readonly StructSize[] StructSizes =
        {
            new StructSize("lumio_handle_t", 16),
            new StructSize("lumio_buffer_t", 24),
            new StructSize("lumio_core_api", 48),
            new StructSize("lumio_voxel_api", 32),
            new StructSize("lumio_root_api", 64),
        };
    }

    // ADR-040 section 7 (D-015): capability keys projected from the ID Registry.
    // ids/index.json is the authority for these numerics; this is its only published
    // projection. Enumeration keys, not bit positions.
    public readonly struct CapabilityKey
    {
        public CapabilityKey(string name, uint numeric, string status)
        {
            Name = name; Numeric = numeric; Status = status;
        }
        public string Name { get; }
        public uint Numeric { get; }
        public string Status { get; }
    }

    public static class CapabilityKeys
    {
        public static readonly CapabilityKey[] All =
        {
            new CapabilityKey("Native", 1u, "Active"),
            new CapabilityKey("HybridCLR", 2u, "Reserved"),
            new CapabilityKey("ReferenceVoxel", 3u, "Active"),
            new CapabilityKey("VoxelSnapshot", 4u, "Active"),
            new CapabilityKey("VoxelStreaming", 5u, "Active"),
            new CapabilityKey("VoxelSpatial", 6u, "Active"),
            new CapabilityKey("VoxelMeshCollision", 7u, "Active"),
            new CapabilityKey("VoxelAllResident", 8u, "Active"),
            new CapabilityKey("VoxelVolatileChunks", 9u, "Active"),
        };

        public const uint Native = 1u;
        public const uint HybridCLR = 2u;
        public const uint ReferenceVoxel = 3u;
        public const uint VoxelSnapshot = 4u;
        public const uint VoxelStreaming = 5u;
        public const uint VoxelSpatial = 6u;
        public const uint VoxelMeshCollision = 7u;
        public const uint VoxelAllResident = 8u;
        public const uint VoxelVolatileChunks = 9u;
    }
}
