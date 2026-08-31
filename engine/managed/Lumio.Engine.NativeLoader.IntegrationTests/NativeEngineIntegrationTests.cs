using Lumio.Engine.NativeLoader;
using Xunit;

namespace Lumio.Engine.NativeLoader.IntegrationTests;

public sealed class NativeEngineIntegrationTests
{
    [Fact]
    public void LoadsTheStagedNativeImageAndExecutesTheRootApi()
    {
        var nativePath = RequiredEnvironment("LUMIO_NATIVE_TEST_PATH");
        var buildId = RequiredEnvironment("LUMIO_NATIVE_TEST_BUILD_ID");
        var abiHash = RequiredEnvironment("LUMIO_NATIVE_TEST_ABI_HASH");

        using var lease = NativeEngineLoader.Load(nativePath, buildId, abiHash);

        lease.Ping();
        Assert.Equal(buildId, lease.BuildId);
        Assert.Equal(abiHash, lease.AbiHash);
        Assert.Equal(nativePath, lease.NativePath);
    }

    private static string RequiredEnvironment(string name)
        => Environment.GetEnvironmentVariable(name)
           ?? throw new InvalidOperationException($"Set {name} before running the native integration test.");
}
