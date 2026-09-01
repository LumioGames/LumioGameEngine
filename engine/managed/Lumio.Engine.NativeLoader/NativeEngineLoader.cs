using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text.Json;

namespace Lumio.Engine.NativeLoader;

public enum NativeEngineLoadFailure
{
    MissingFile,
    InvalidNativeImage,
    UnsupportedVersion,
    ApiTableNull,
    AbiMismatch,
    BuildIdMismatch,
}

public sealed class NativeEngineLoadException : Exception
{
    public NativeEngineLoadException(NativeEngineLoadFailure failure, string message, Exception? inner = null)
        : base(message, inner)
    {
        Failure = failure;
    }

    public NativeEngineLoadFailure Failure { get; }
}

public sealed record NativeBuildInfo(string BuildId, string AbiHash, string BinarySha256)
{
    public static string SidecarPath(string nativePath)
        => Path.Combine(Path.GetDirectoryName(nativePath) ?? string.Empty, "build-info.json");

    public static NativeBuildInfo Read(string path)
    {
        using var document = JsonDocument.Parse(File.ReadAllText(path));
        var root = document.RootElement;
        return new NativeBuildInfo(
            Required(root, "buildId"),
            Required(root, "abiHash"),
            Required(root, "binarySha256"));
    }

    public bool Matches(string expectedBuildId, string expectedAbiHash)
        => string.Equals(BuildId, expectedBuildId, StringComparison.OrdinalIgnoreCase)
           && string.Equals(AbiHash, expectedAbiHash, StringComparison.OrdinalIgnoreCase);

    private static string Required(JsonElement root, string name)
    {
        if (!root.TryGetProperty(name, out var value) || value.ValueKind != JsonValueKind.String)
        {
            throw new FormatException($"build-info.json is missing string property '{name}'.");
        }

        return value.GetString()!;
    }
}

public sealed class NativeEngineLease : IDisposable
{
    private readonly nint _library;
    private readonly nint _ping;
    private bool _disposed;

    internal NativeEngineLease(
        nint library,
        nint _,
        string nativePath,
        string buildId,
        string abiHash,
        string binarySha256,
        nint ping)
    {
        _library = library;
        _ping = ping;
        NativePath = nativePath;
        BuildId = buildId;
        AbiHash = abiHash;
        BinarySha256 = binarySha256;
    }

    public string NativePath { get; }
    public string BuildId { get; }
    public string AbiHash { get; }
    public string BinarySha256 { get; }

    public void Ping()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        var ping = Marshal.GetDelegateForFunctionPointer<PingDelegate>(_ping);
        var marker = Marshal.AllocHGlobal(sizeof(uint));
        try
        {
            Marshal.WriteInt32(marker, 0);
            var status = ping(marker);
            if (status != 0 || Marshal.ReadInt32(marker) != 1)
            {
                throw new NativeEngineLoadException(
                    NativeEngineLoadFailure.InvalidNativeImage,
                    $"Native ping failed with status {status}.");
            }
        }
        finally
        {
            Marshal.FreeHGlobal(marker);
        }
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        _disposed = true;
        NativeLibrary.Free(_library);
    }

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate int PingDelegate(nint marker);
}

public static class NativeEngineLoader
{
    private static string EntrySymbol => AbiConstants.EntrySymbol;
    private static uint AbiVersion => AbiConstants.AbiVersion;

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate int GetApiDelegate(uint requestedVersion, out nint api);

    /// <summary>
    /// 根 API 表的托管镜像（engine/abi/native-abi.json 的 root.fields 是唯一真值）。
    /// 只追加不插入；struct_size 按「≥ 托管布局」协商，字段偏移由
    /// RootApiLayoutTests 按名字锁定。
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    internal struct RootApi
    {
        public uint AbiVersion;
        public uint StructSize;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 32)]
        public byte[]? AbiHash;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 16)]
        public byte[]? BuildId;
        public nint Ping;
        public nint CreateClrHost;
        public nint ClrHostCall;
        public nint DestroyClrHost;
    }

    public static NativeEngineLease Load(string nativePath, string expectedBuildId, string expectedAbiHash)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(nativePath);
        ArgumentException.ThrowIfNullOrWhiteSpace(expectedBuildId);
        ArgumentException.ThrowIfNullOrWhiteSpace(expectedAbiHash);

        if (!File.Exists(nativePath))
        {
            throw new NativeEngineLoadException(
                NativeEngineLoadFailure.MissingFile,
                $"Native engine file does not exist: {nativePath}");
        }

        var binarySha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(nativePath))).ToLowerInvariant();
        nint library = 0;
        try
        {
            library = NativeLibrary.Load(nativePath);
            var entryAddress = NativeLibrary.GetExport(library, EntrySymbol);
            var entry = Marshal.GetDelegateForFunctionPointer<GetApiDelegate>(entryAddress);
            var status = entry(AbiVersion, out var apiAddress);
            if (status != 0)
            {
                throw new NativeEngineLoadException(
                    NativeEngineLoadFailure.UnsupportedVersion,
                    $"Native engine entry rejected ABI version {AbiVersion} with status {status}.");
            }

            if (apiAddress == 0)
            {
                throw new NativeEngineLoadException(
                    NativeEngineLoadFailure.ApiTableNull,
                    "Native engine entry returned a null API table.");
            }

            var api = Marshal.PtrToStructure<RootApi>(apiAddress);
            if (api.AbiVersion != AbiVersion
                || api.StructSize < Marshal.SizeOf<RootApi>()
                || api.Ping == 0
                || api.CreateClrHost == 0
                || api.ClrHostCall == 0
                || api.DestroyClrHost == 0)
            {
                throw new NativeEngineLoadException(
                    NativeEngineLoadFailure.InvalidNativeImage,
                    "Native engine API table has an invalid version, size, ping, or CLR host slot.");
            }

            var abiHash = Convert.ToHexString(api.AbiHash ?? Array.Empty<byte>()).ToLowerInvariant();
            var buildId = Convert.ToHexString(api.BuildId ?? Array.Empty<byte>()).ToLowerInvariant();
            if (!string.Equals(abiHash, expectedAbiHash, StringComparison.OrdinalIgnoreCase))
            {
                throw new NativeEngineLoadException(
                    NativeEngineLoadFailure.AbiMismatch,
                    $"Native engine ABI hash {abiHash} does not match {expectedAbiHash}.");
            }

            if (!string.Equals(buildId, expectedBuildId, StringComparison.OrdinalIgnoreCase))
            {
                throw new NativeEngineLoadException(
                    NativeEngineLoadFailure.BuildIdMismatch,
                    $"Native engine BuildId {buildId} does not match {expectedBuildId}.");
            }

            return new NativeEngineLease(library, apiAddress, nativePath, buildId, abiHash, binarySha256, api.Ping);
        }
        catch (NativeEngineLoadException)
        {
            if (library != 0)
            {
                NativeLibrary.Free(library);
            }

            throw;
        }
        catch (Exception ex) when (ex is DllNotFoundException or BadImageFormatException or EntryPointNotFoundException)
        {
            if (library != 0)
            {
                NativeLibrary.Free(library);
            }

            throw new NativeEngineLoadException(
                NativeEngineLoadFailure.InvalidNativeImage,
                $"Could not load native engine image {nativePath}.",
                ex);
        }
    }

    public static NativeEngineLease LoadFromBuildInfo(string nativePath)
    {
        var sidecar = NativeBuildInfo.SidecarPath(nativePath);
        if (!File.Exists(sidecar))
        {
            throw new NativeEngineLoadException(
                NativeEngineLoadFailure.MissingFile,
                $"Native engine build-info sidecar does not exist: {sidecar}");
        }

        var info = NativeBuildInfo.Read(sidecar);
        return Load(nativePath, info.BuildId, info.AbiHash);
    }
}
