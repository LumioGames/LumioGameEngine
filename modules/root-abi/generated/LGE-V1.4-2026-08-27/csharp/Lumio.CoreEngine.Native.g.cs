// Generated Root ABI binding. Do not hand-edit.
// Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27. ADR-040.
// Pure managed layout description; the consumer binds the entry symbol itself.
using System;
using System.Runtime.InteropServices;

namespace Lumio.Gen.LanguageBinding;

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

public readonly record struct SlotOffset(string Table, string Slot, int Offset);
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

    public static readonly (string Name, int Size)[] StructSizes =
    {
        ("lumio_handle_t", 16),
        ("lumio_buffer_t", 24),
        ("lumio_core_api", 48),
        ("lumio_voxel_api", 32),
        ("lumio_root_api", 64),
    };
}
