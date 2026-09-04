using Lumio.Engine.NativeLoader;
using Xunit;

namespace Lumio.Engine.NativeLoader.Tests;

public sealed class NativeEngineLoaderTests
{
    [Fact]
    public void NativeTimerAbiExposesTheHostedTimerOperations()
    {
        Assert.NotNull(typeof(INativeTimerAbi).GetMethod(nameof(INativeTimerAbi.CreateManager)));
        Assert.NotNull(typeof(INativeTimerAbi).GetMethod(nameof(INativeTimerAbi.ScheduleRepeating)));
        Assert.NotNull(typeof(INativeTimerAbi).GetMethod(nameof(INativeTimerAbi.Drain)));
        Assert.Equal(16, System.Runtime.InteropServices.Marshal.SizeOf<NativeTimerHandle>());
    }
    [Fact]
    public void MissingNativeFileIsRejectedBeforeLoad()
    {
        var error = Assert.Throws<NativeEngineLoadException>(() =>
            NativeEngineLoader.Load(
                Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString("N"), "missing.dll"),
                "build-id",
                "abi-hash"));

        Assert.Equal(NativeEngineLoadFailure.MissingFile, error.Failure);
    }

    [Fact]
    public void FileHashMismatchIsRejectedBeforeNativeLoad()
    {
        var path = Path.GetTempFileName();
        try
        {
            File.WriteAllText(path, "native-bytes");
            var error = Assert.Throws<NativeEngineLoadException>(() =>
                NativeEngineLoader.Load(path, "build-id", "0000000000000000000000000000000000000000000000000000000000000000"));

            Assert.Equal(NativeEngineLoadFailure.InvalidNativeImage, error.Failure);
        }
        finally
        {
            File.Delete(path);
        }
    }

    [Fact]
    public void BuildInfoRequiresMatchingSourceAndAbiIdentities()
    {
        var info = new NativeBuildInfo("build-id", "abi-hash", "binary-hash");

        Assert.True(info.Matches("build-id", "abi-hash"));
        Assert.False(info.Matches("old-build-id", "abi-hash"));
        Assert.False(info.Matches("build-id", "old-abi-hash"));
    }

    [Fact]
    public void BuildInfoReadsTheDevSidecar()
    {
        var path = Path.GetTempFileName();
        try
        {
            File.WriteAllText(path, "{\"buildId\":\"build-id\",\"abiHash\":\"abi-hash\",\"binarySha256\":\"binary-hash\"}");

            var info = NativeBuildInfo.Read(path);

            Assert.Equal("build-id", info.BuildId);
            Assert.Equal("abi-hash", info.AbiHash);
            Assert.Equal("binary-hash", info.BinarySha256);
        }
        finally
        {
            File.Delete(path);
        }
    }

    [Fact]
    public void BuildInfoSidecarPathIsDerivedFromNativeImage()
    {
        var nativePath = Path.Combine("run", "build-id", "engine.dll");

        Assert.Equal(Path.Combine("run", "build-id", "build-info.json"), NativeBuildInfo.SidecarPath(nativePath));
    }

    [Fact]
    public void NativeTimerAdapterUsesTheLeaseModuleHandleWithoutReflection()
    {
        var source = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "Lumio.Engine.NativeLoader", "NativeLoaderTimerAbi.cs"));
        Assert.DoesNotContain("System.Reflection", source, StringComparison.Ordinal);
        Assert.DoesNotContain("GetField(\"_library\"", source, StringComparison.Ordinal);
    }

    [Fact]
    public void MissingRequiredTimerSlotIsRejected()
    {
        var present = (nint)1;
        var api = new NativeEngineLoader.RootApi
        {
            StructSize = (uint)System.Runtime.InteropServices.Marshal.SizeOf<NativeEngineLoader.RootApi>(),
            TimerCreateManager = present,
            TimerDestroyManager = present,
            TimerRegisterDispatch = present,
            TimerRegisterScope = present,
            TimerTeardownScope = present,
            TimerCreateSlot = present,
            TimerBindSlot = present,
            TimerCloseSlot = present,
            TimerScheduleOneShot = present,
            TimerScheduleRepeating = present,
            TimerCancel = present,
            TimerAdvance = present,
            TimerPump = 0,
            TimerDrain = present,
        };

        var error = Assert.Throws<InvalidOperationException>(() => NativeEngineLoader.ValidateTimerSlots(in api));
        Assert.Contains("timer_* slots", error.Message, StringComparison.Ordinal);
    }
}
