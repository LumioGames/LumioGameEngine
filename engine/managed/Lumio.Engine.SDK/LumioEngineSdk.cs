using Lumio.Engine.NativeLoader;

namespace Lumio.Engine.SDK;

public static class LumioEngineSdk
{
    public static LumioEngineLease LoadNative(string nativePath)
        => new(NativeEngineLoader.LoadFromBuildInfo(nativePath));
}

public sealed class LumioEngineLease : IDisposable
{
    private readonly NativeEngineLease _inner;

    internal LumioEngineLease(NativeEngineLease inner) => _inner = inner;

    public string NativePath => _inner.NativePath;
    public string BuildId => _inner.BuildId;
    public string AbiHash => _inner.AbiHash;
    public string BinarySha256 => _inner.BinarySha256;

    public void Ping() => _inner.Ping();
    public void Dispose() => _inner.Dispose();
}
