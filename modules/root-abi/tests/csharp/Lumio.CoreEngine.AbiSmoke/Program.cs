// C# 侧 ABI 布局探针（规格 §8.2「生成物/跨语言测试」）。
//
// C 有 header 的 static assert、Rust 有生成绑定的 const assert，二者都在编译期把
// 布局钉死；C# 没有等价的编译期设施，因此这里在运行期把 Marshal 测量值与**生成物
// 自带的 Golden**（RootAbiLayout.StructSizes / SlotOffsets、RootAbi.* 常量）逐项比对。
// 本文件不写任何布局数值——Golden 一律取自生成绑定。
//
// 输出格式与 C / Rust 两侧完全一致，便于三份逐行 diff。

using System.Runtime.InteropServices;

using Lumio.Gen.LanguageBinding;

internal static class Program
{
    // 对齐探针：Sequential 布局下 `V` 的偏移即该类型的非托管对齐。
    // 泛型结构体不能交给 Marshal.OffsetOf，故每个受测类型各有一个。
    [StructLayout(LayoutKind.Sequential)]
    private struct AlignOfHandle { public byte B; public LumioHandle V; }

    [StructLayout(LayoutKind.Sequential)]
    private struct AlignOfBuffer { public byte B; public LumioBuffer V; }

    [StructLayout(LayoutKind.Sequential)]
    private struct AlignOfCoreApi { public byte B; public LumioCoreApi V; }

    [StructLayout(LayoutKind.Sequential)]
    private struct AlignOfVoxelApi { public byte B; public LumioVoxelApi V; }

    [StructLayout(LayoutKind.Sequential)]
    private struct AlignOfRootApi { public byte B; public LumioRootApi V; }

    private static int _failures;

    private static void Check(string key, long measured, long golden)
    {
        Console.WriteLine($"{key}={measured}");
        if (measured != golden)
        {
            Console.Error.WriteLine($"FAIL {key}: measured {measured}, golden {golden}");
            _failures++;
        }
    }

    private static int SizeOfByGoldenName(string name) => name switch
    {
        "lumio_handle_t" => Marshal.SizeOf<LumioHandle>(),
        "lumio_buffer_t" => Marshal.SizeOf<LumioBuffer>(),
        "lumio_core_api" => Marshal.SizeOf<LumioCoreApi>(),
        "lumio_voxel_api" => Marshal.SizeOf<LumioVoxelApi>(),
        "lumio_root_api" => Marshal.SizeOf<LumioRootApi>(),
        _ => throw new InvalidOperationException($"Golden 出现未知结构 {name}"),
    };

    private static int AlignOfByGoldenName(string name) => name switch
    {
        "lumio_handle_t" => (int)Marshal.OffsetOf<AlignOfHandle>(nameof(AlignOfHandle.V)),
        "lumio_buffer_t" => (int)Marshal.OffsetOf<AlignOfBuffer>(nameof(AlignOfBuffer.V)),
        "lumio_core_api" => (int)Marshal.OffsetOf<AlignOfCoreApi>(nameof(AlignOfCoreApi.V)),
        "lumio_voxel_api" => (int)Marshal.OffsetOf<AlignOfVoxelApi>(nameof(AlignOfVoxelApi.V)),
        "lumio_root_api" => (int)Marshal.OffsetOf<AlignOfRootApi>(nameof(AlignOfRootApi.V)),
        _ => throw new InvalidOperationException($"Golden 出现未知结构 {name}"),
    };

    private static int SlotOffsetOf(string table, string slot) => (table, slot) switch
    {
        ("lumio_core_api", "lumio_core_init") =>
            (int)Marshal.OffsetOf<LumioCoreApi>(nameof(LumioCoreApi.LumioCoreInit)),
        ("lumio_core_api", "lumio_core_shutdown") =>
            (int)Marshal.OffsetOf<LumioCoreApi>(nameof(LumioCoreApi.LumioCoreShutdown)),
        ("lumio_core_api", "lumio_core_last_error_detail") =>
            (int)Marshal.OffsetOf<LumioCoreApi>(nameof(LumioCoreApi.LumioCoreLastErrorDetail)),
        ("lumio_voxel_api", "lumio_voxel_world_create") =>
            (int)Marshal.OffsetOf<LumioVoxelApi>(nameof(LumioVoxelApi.LumioVoxelWorldCreate)),
        ("lumio_voxel_api", "lumio_voxel_world_destroy") =>
            (int)Marshal.OffsetOf<LumioVoxelApi>(nameof(LumioVoxelApi.LumioVoxelWorldDestroy)),
        _ => throw new InvalidOperationException($"Golden 出现未知 slot {table}.{slot}"),
    };

    private static int Main()
    {
        Console.WriteLine($"abi_version={RootAbi.AbiVersion}");
        Console.WriteLine($"capability_bits={RootAbi.CapabilityBits}");
        Console.WriteLine($"entry_symbol={RootAbi.EntrySymbol}");
        Console.WriteLine($"symbol_prefix={RootAbi.SymbolPrefix}");
        Console.WriteLine($"pointer_bytes={IntPtr.Size}");

        if (IntPtr.Size != RootAbi.PointerBytes)
        {
            Console.Error.WriteLine(
                $"FAIL pointer_bytes: 宿主 {IntPtr.Size}, golden {RootAbi.PointerBytes}");
            _failures++;
        }

        foreach ((string name, int size) in RootAbiLayout.StructSizes)
        {
            Check($"size.{name}", SizeOfByGoldenName(name), size);
        }

        // 对齐 Golden 是 layout profile 的 maxAlignment（生成绑定发布为 MaxAlignment）：
        // 本 profile 下每个受测结构的对齐都等于它。
        foreach ((string name, int _) in RootAbiLayout.StructSizes)
        {
            Check($"align.{name}", AlignOfByGoldenName(name), RootAbi.MaxAlignment);
        }

        foreach (SlotOffset slot in RootAbiLayout.SlotOffsets)
        {
            Check($"offset.{slot.Table}.{slot.Slot}", SlotOffsetOf(slot.Table, slot.Slot), slot.Offset);
        }

        // Root Table 的 table 指针槽位：ADR-040 §4 的 `16 + i * pointerBytes`，
        // 两个基数都来自生成绑定的已发布常量。
        Check("offset.lumio_root_api.lumio_core_api",
              (int)Marshal.OffsetOf<LumioRootApi>(nameof(LumioRootApi.LumioCoreApi)),
              RootAbi.RootHeaderBytes);
        Check("offset.lumio_root_api.lumio_voxel_api",
              (int)Marshal.OffsetOf<LumioRootApi>(nameof(LumioRootApi.LumioVoxelApi)),
              RootAbi.RootHeaderBytes + RootAbi.PointerBytes);

        if (_failures != 0)
        {
            Console.Error.WriteLine($"C# layout probe: {_failures} 项与上游 Golden 不一致");
            return 1;
        }

        return 0;
    }
}
