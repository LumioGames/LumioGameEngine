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

        Assert.Equal(280, Marshal.SizeOf<NativeEngineLoader.RootApi>());

        Assert.Equal(0L, Offset(nameof(NativeEngineLoader.RootApi.AbiVersion)));
        Assert.Equal(4L, Offset(nameof(NativeEngineLoader.RootApi.StructSize)));
        Assert.Equal(8L, Offset(nameof(NativeEngineLoader.RootApi.AbiHash)));
        Assert.Equal(40L, Offset(nameof(NativeEngineLoader.RootApi.BuildId)));
        Assert.Equal(56L, Offset(nameof(NativeEngineLoader.RootApi.Ping)));
        Assert.Equal(64L, Offset(nameof(NativeEngineLoader.RootApi.CreateClrHost)));
        Assert.Equal(72L, Offset(nameof(NativeEngineLoader.RootApi.ClrHostCall)));
        Assert.Equal(80L, Offset(nameof(NativeEngineLoader.RootApi.DestroyClrHost)));
        Assert.Equal(88L, Offset(nameof(NativeEngineLoader.RootApi.TimerCreateManager)));
        Assert.Equal(192L, Offset(nameof(NativeEngineLoader.RootApi.TimerDrain)));
        Assert.Equal(200L, Offset(nameof(NativeEngineLoader.RootApi.BlockReadCell)));
        Assert.Equal(208L, Offset(nameof(NativeEngineLoader.RootApi.BlockReadBox)));
        Assert.Equal(216L, Offset(nameof(NativeEngineLoader.RootApi.BlockReadColumn)));
        Assert.Equal(224L, Offset(nameof(NativeEngineLoader.RootApi.BlockWritePrepare)));
        Assert.Equal(232L, Offset(nameof(NativeEngineLoader.RootApi.BlockWriteCommit)));
        Assert.Equal(240L, Offset(nameof(NativeEngineLoader.RootApi.BlockWriteAbort)));
        Assert.Equal(248L, Offset(nameof(NativeEngineLoader.RootApi.SectionRevisionQuery)));
        Assert.Equal(256L, Offset(nameof(NativeEngineLoader.RootApi.ResidencyPinDeclare)));
        Assert.Equal(264L, Offset(nameof(NativeEngineLoader.RootApi.ResidencyPinRelease)));
        Assert.Equal(272L, Offset(nameof(NativeEngineLoader.RootApi.ResidencyPinStatus)));
    }

    private static long Offset(string field)
        => Marshal.OffsetOf<NativeEngineLoader.RootApi>(field).ToInt64();
}
