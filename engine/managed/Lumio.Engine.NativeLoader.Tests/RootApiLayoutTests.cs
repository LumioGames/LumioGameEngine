using System.Runtime.InteropServices;
using Xunit;

namespace Lumio.Engine.NativeLoader.Tests;

/// <summary>
/// 根表布局是跨语言契约（engine/abi/native-abi.json）：这里按 x64 指针宽度固定断言
/// 每个字段的偏移与总大小，防止托管侧布局悄悄漂移。偏移按字段名解析——字段缺失或
/// 顺序漂移都会直接失败。
/// </summary>
public sealed class RootApiLayoutTests
{
    [Fact]
    public void RootApiLayoutMatchesTheNativeRootTable()
    {
        if (!Environment.Is64BitProcess)
        {
            return;
        }

        Assert.Equal(88, Marshal.SizeOf<NativeEngineLoader.RootApi>());

        Assert.Equal(0L, Offset(nameof(NativeEngineLoader.RootApi.AbiVersion)));
        Assert.Equal(4L, Offset(nameof(NativeEngineLoader.RootApi.StructSize)));
        Assert.Equal(8L, Offset(nameof(NativeEngineLoader.RootApi.AbiHash)));
        Assert.Equal(40L, Offset(nameof(NativeEngineLoader.RootApi.BuildId)));
        Assert.Equal(56L, Offset(nameof(NativeEngineLoader.RootApi.Ping)));
        Assert.Equal(64L, Offset(nameof(NativeEngineLoader.RootApi.CreateClrHost)));
        Assert.Equal(72L, Offset(nameof(NativeEngineLoader.RootApi.ClrHostCall)));
        Assert.Equal(80L, Offset(nameof(NativeEngineLoader.RootApi.DestroyClrHost)));
    }

    private static long Offset(string field)
        => Marshal.OffsetOf<NativeEngineLoader.RootApi>(field).ToInt64();
}
