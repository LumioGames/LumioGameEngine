using Lumio.Engine.SDK;
using Xunit;

namespace Lumio.Engine.NativeLoader.Tests;

public sealed class SdkFacadeTests
{
    [Fact]
    public void SDKFacadeExposesTheNativeLoadEntryPoint()
    {
        var missing = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString("N"), "engine.dll");

        var error = Assert.Throws<NativeEngineLoadException>(() => LumioEngineSdk.LoadNative(missing));

        Assert.Equal(NativeEngineLoadFailure.MissingFile, error.Failure);
    }
}
