using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
using Native = Lumio.Engine.NativeLoader;

namespace Lumio.Engine.SDK;

public enum VoxelPresence
{
    Ready = 0,
    Unchanged = 1,
    Pending = 2,
    Unavailable = 3,
}

public readonly record struct VoxelSectionKey(int X, byte Y, int Z);

public readonly record struct VoxelWorldCoordinate(int X, byte Y, int Z);

public readonly record struct VoxelBoxRequest(VoxelWorldCoordinate Min, VoxelWorldCoordinate Max);

public readonly record struct VoxelColumnRequest(int X, int Z, byte MinY, byte MaxY);

public readonly record struct VoxelBlockReadResult(
    VoxelPresence Presence,
    bool HasBlockId,
    uint BlockId,
    ulong SectionRevision);

public readonly record struct VoxelSectionSegment(
    VoxelSectionKey SectionKey,
    VoxelPresence Presence,
    ulong SectionRevision,
    uint FirstResult,
    uint ResultCount);

public readonly record struct VoxelBatchReadResult(
    uint ResultCount,
    uint SegmentCount,
    bool Truncated);

public readonly record struct VoxelSectionRevisionResult(
    VoxelPresence Presence,
    ulong SectionRevision);

public readonly record struct VoxelBlockWriteEntry(
    VoxelSectionKey SectionKey,
    ushort CellOffset,
    uint BlockId,
    ulong ExpectedSectionRevision);

public readonly record struct VoxelWriteReceipt(
    VoxelSectionKey SectionKey,
    ulong UpToSectionRevision,
    ulong WorldRevision);

public enum VoxelErrorCode
{
    Unknown = -1,
    UnknownSectionKey = Native.VoxelErrorCodes.UnknownSectionKey,
    UnknownChunkKey = Native.VoxelErrorCodes.UnknownChunkKey,
    SectionYOutOfRange = Native.VoxelErrorCodes.SectionYOutOfRange,
    CoordinateOutOfBounds = Native.VoxelErrorCodes.CoordinateOutOfBounds,
    SectionUnavailable = Native.VoxelErrorCodes.SectionUnavailable,
    StaleSectionRevision = Native.VoxelErrorCodes.StaleSectionRevision,
    PaletteOverflow = Native.VoxelErrorCodes.PaletteOverflow,
    SectionEncodingMismatch = Native.VoxelErrorCodes.SectionEncodingMismatch,
    SectionDigestMismatch = Native.VoxelErrorCodes.SectionDigestMismatch,
    DirtySectionNotDurable = Native.VoxelErrorCodes.DirtySectionNotDurable,
    LightingInPayload = Native.VoxelErrorCodes.LightingInPayload,
    ChunkCarriesData = Native.VoxelErrorCodes.ChunkCarriesData,
    UnknownMaterialClass = Native.VoxelErrorCodes.UnknownMaterialClass,
    MaterialClassNotACellLane = Native.VoxelErrorCodes.MaterialClassNotACellLane,
    LiquidAutoPropagationUnsupported = Native.VoxelErrorCodes.LiquidAutoPropagationUnsupported,
    CrossMaterialFaceMerge = Native.VoxelErrorCodes.CrossMaterialFaceMerge,
    EntityBindingMissing = Native.VoxelErrorCodes.EntityBindingMissing,
    EntityBindingOrphan = Native.VoxelErrorCodes.EntityBindingOrphan,
    EntityBindingTypeMismatch = Native.VoxelErrorCodes.EntityBindingTypeMismatch,
    EntityBindingNotSparse = Native.VoxelErrorCodes.EntityBindingNotSparse,
    BusinessDataInPayload = Native.VoxelErrorCodes.BusinessDataInPayload,
    BindingCommitSplit = Native.VoxelErrorCodes.BindingCommitSplit,
    BlockTypeScopeViolation = Native.VoxelErrorCodes.BlockTypeScopeViolation,
    SystemReservedTypeMisuse = Native.VoxelErrorCodes.SystemReservedTypeMisuse,
    RoomLocalTypeWithoutMapping = Native.VoxelErrorCodes.RoomLocalTypeWithoutMapping,
    PlayerTypeDeclaresBehavior = Native.VoxelErrorCodes.PlayerTypeDeclaresBehavior,
    PaletteReclaimBeforeEscalation = Native.VoxelErrorCodes.PaletteReclaimBeforeEscalation,
    DeadPaletteEntryInPayload = Native.VoxelErrorCodes.DeadPaletteEntryInPayload,
    DeltaBaseRevisionMismatch = Native.VoxelErrorCodes.DeltaBaseRevisionMismatch,
    DeltaUsedForFirstDelivery = Native.VoxelErrorCodes.DeltaUsedForFirstDelivery,
    UnresolvedHitTreatedAsAir = Native.VoxelErrorCodes.UnresolvedHitTreatedAsAir,
    UnresolvedHitTreatedAsSolid = Native.VoxelErrorCodes.UnresolvedHitTreatedAsSolid,
    QueryBufferOverflow = Native.VoxelErrorCodes.QueryBufferOverflow,
    QueryResultDivergence = Native.VoxelErrorCodes.QueryResultDivergence,
    CollisionBehaviorNotFromMaterialTable = Native.VoxelErrorCodes.CollisionBehaviorNotFromMaterialTable,
    QueryMutatesWorld = Native.VoxelErrorCodes.QueryMutatesWorld,
    WorldYOutOfRange = Native.VoxelErrorCodes.WorldYOutOfRange,
    BlockCatalogNotDense = Native.VoxelErrorCodes.BlockCatalogNotDense,
    BlockCatalogNameReused = Native.VoxelErrorCodes.BlockCatalogNameReused,
    BlockCatalogRowIncomplete = Native.VoxelErrorCodes.BlockCatalogRowIncomplete,
    ReadBudgetExceeded = Native.VoxelErrorCodes.ReadBudgetExceeded,
    ReadResultMissingRevision = Native.VoxelErrorCodes.ReadResultMissingRevision,
    WriteBatchTooLarge = Native.VoxelErrorCodes.WriteBatchTooLarge,
    UnstructuredMutationEntry = Native.VoxelErrorCodes.UnstructuredMutationEntry,
    CellOffsetOutOfRange = Native.VoxelErrorCodes.CellOffsetOutOfRange,
    ResidencyPinExceedsBudget = Native.VoxelErrorCodes.ResidencyPinExceedsBudget,
    PinRegionNotReady = Native.VoxelErrorCodes.PinRegionNotReady,
    PinnedSectionEvicted = Native.VoxelErrorCodes.PinnedSectionEvicted,
    PinnedReadReturnedPending = Native.VoxelErrorCodes.PinnedReadReturnedPending,
    UnknownBehaviorTemplate = Native.VoxelErrorCodes.UnknownBehaviorTemplate,
    CellReadMissingPresence = Native.VoxelErrorCodes.CellReadMissingPresence,
    UnregisteredBlockType = Native.VoxelErrorCodes.UnregisteredBlockType,
}

public static class VoxelErrorCodeMap
{
    private static readonly int[] ContractStatuses =
    {
        Native.VoxelErrorCodes.UnknownSectionKey,
        Native.VoxelErrorCodes.UnknownChunkKey,
        Native.VoxelErrorCodes.SectionYOutOfRange,
        Native.VoxelErrorCodes.CoordinateOutOfBounds,
        Native.VoxelErrorCodes.SectionUnavailable,
        Native.VoxelErrorCodes.StaleSectionRevision,
        Native.VoxelErrorCodes.PaletteOverflow,
        Native.VoxelErrorCodes.SectionEncodingMismatch,
        Native.VoxelErrorCodes.SectionDigestMismatch,
        Native.VoxelErrorCodes.DirtySectionNotDurable,
        Native.VoxelErrorCodes.LightingInPayload,
        Native.VoxelErrorCodes.ChunkCarriesData,
        Native.VoxelErrorCodes.UnknownMaterialClass,
        Native.VoxelErrorCodes.MaterialClassNotACellLane,
        Native.VoxelErrorCodes.LiquidAutoPropagationUnsupported,
        Native.VoxelErrorCodes.CrossMaterialFaceMerge,
        Native.VoxelErrorCodes.EntityBindingMissing,
        Native.VoxelErrorCodes.EntityBindingOrphan,
        Native.VoxelErrorCodes.EntityBindingTypeMismatch,
        Native.VoxelErrorCodes.EntityBindingNotSparse,
        Native.VoxelErrorCodes.BusinessDataInPayload,
        Native.VoxelErrorCodes.BindingCommitSplit,
        Native.VoxelErrorCodes.BlockTypeScopeViolation,
        Native.VoxelErrorCodes.SystemReservedTypeMisuse,
        Native.VoxelErrorCodes.RoomLocalTypeWithoutMapping,
        Native.VoxelErrorCodes.PlayerTypeDeclaresBehavior,
        Native.VoxelErrorCodes.PaletteReclaimBeforeEscalation,
        Native.VoxelErrorCodes.DeadPaletteEntryInPayload,
        Native.VoxelErrorCodes.DeltaBaseRevisionMismatch,
        Native.VoxelErrorCodes.DeltaUsedForFirstDelivery,
        Native.VoxelErrorCodes.UnresolvedHitTreatedAsAir,
        Native.VoxelErrorCodes.UnresolvedHitTreatedAsSolid,
        Native.VoxelErrorCodes.QueryBufferOverflow,
        Native.VoxelErrorCodes.QueryResultDivergence,
        Native.VoxelErrorCodes.CollisionBehaviorNotFromMaterialTable,
        Native.VoxelErrorCodes.QueryMutatesWorld,
        Native.VoxelErrorCodes.WorldYOutOfRange,
        Native.VoxelErrorCodes.BlockCatalogNotDense,
        Native.VoxelErrorCodes.BlockCatalogNameReused,
        Native.VoxelErrorCodes.BlockCatalogRowIncomplete,
        Native.VoxelErrorCodes.ReadBudgetExceeded,
        Native.VoxelErrorCodes.ReadResultMissingRevision,
        Native.VoxelErrorCodes.WriteBatchTooLarge,
        Native.VoxelErrorCodes.UnstructuredMutationEntry,
        Native.VoxelErrorCodes.CellOffsetOutOfRange,
        Native.VoxelErrorCodes.ResidencyPinExceedsBudget,
        Native.VoxelErrorCodes.PinRegionNotReady,
        Native.VoxelErrorCodes.PinnedSectionEvicted,
        Native.VoxelErrorCodes.PinnedReadReturnedPending,
        Native.VoxelErrorCodes.UnknownBehaviorTemplate,
        Native.VoxelErrorCodes.CellReadMissingPresence,
        Native.VoxelErrorCodes.UnregisteredBlockType,
    };

    public static bool TryMap(int status, out VoxelErrorCode code)
    {
        var index = Array.IndexOf(ContractStatuses, status);
        if (index >= 0)
        {
            code = (VoxelErrorCode)status;
            return true;
        }

        code = VoxelErrorCode.Unknown;
        return false;
    }

    public static int ToStatus(VoxelErrorCode code)
    {
        var status = (int)code;
        if (!TryMap(status, out _))
        {
            throw new ArgumentOutOfRangeException(nameof(code));
        }

        return status;
    }
}

public sealed class VoxelNativeException : Exception
{
    internal VoxelNativeException(int status, string operation)
        : base(CreateMessage(status, operation))
    {
        Status = status;
        Code = VoxelErrorCodeMap.TryMap(status, out var code) ? code : VoxelErrorCode.Unknown;
    }

    public int Status { get; }
    public VoxelErrorCode Code { get; }

    private static string CreateMessage(int status, string operation)
        => VoxelErrorCodeMap.TryMap(status, out var code)
            ? $"Voxel operation '{operation}' failed with {code} ({status})."
            : $"Voxel operation '{operation}' failed with native status {status}.";
}

internal sealed class VoxelWriteTokenHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    private readonly Native.NativeEngineLease _lease;
    private readonly nint _world;
    private readonly VoxelAbortFn _abort;
    private int _leaseReferenceReleased;

    internal VoxelWriteTokenHandle(
        Native.NativeEngineLease lease,
        nint world,
        nint token,
        VoxelAbortFn abort,
        bool leaseReferenceAlreadyRetained)
        : base(ownsHandle: true)
    {
        ArgumentNullException.ThrowIfNull(lease);
        ArgumentNullException.ThrowIfNull(abort);
        if (token == 0)
        {
            throw new ArgumentOutOfRangeException(nameof(token));
        }

        _lease = lease;
        _world = world;
        _abort = abort;
        if (!leaseReferenceAlreadyRetained)
        {
            _lease.RetainNativeToken();
        }

        SetHandle(token);
    }

    internal nint Token
        => IsInvalid
            ? throw new ObjectDisposedException(nameof(VoxelWriteToken))
            : DangerousGetHandle();

    internal void MarkCompleted()
    {
        lock (this)
        {
            if (IsClosed)
            {
                return;
            }

            SetHandleAsInvalid();
            ReleaseLeaseReference();
        }
    }

    internal int Abort()
    {
        lock (this)
        {
            if (IsInvalid)
            {
                throw new ObjectDisposedException(nameof(VoxelWriteToken));
            }

            var status = _abort(_world, handle);
            if (status == 0)
            {
                SetHandleAsInvalid();
                ReleaseLeaseReference();
            }

            return status;
        }
    }

    protected override bool ReleaseHandle()
    {
        nint token;
        lock (this)
        {
            token = IsInvalid ? 0 : handle;
            handle = 0;
        }

        try
        {
            // SafeHandle cleanup must not be blocked by a disposed managed lease.
            // The lease keeps the native image mapped until this callback returns.
            if (token != 0)
            {
                _abort(_world, token);
            }
        }
        catch
        {
            // A finalizer cannot propagate native cleanup failures. The native token
            // is consumed by the abort attempt and the lease reference is released.
        }
        finally
        {
            ReleaseLeaseReference();
        }

        return true;
    }

    private void ReleaseLeaseReference()
    {
        if (Interlocked.Exchange(ref _leaseReferenceReleased, 1) == 0)
        {
            _lease.ReleaseNativeToken();
        }
    }
}

public sealed class VoxelWriteToken : IDisposable
{
    private readonly NativeVoxelWorld _owner;
    private readonly VoxelWriteTokenHandle _handle;

    internal VoxelWriteToken(
        NativeVoxelWorld owner,
        Native.NativeEngineLease lease,
        nint world,
        nint token,
        VoxelAbortFn abort,
        bool leaseReferenceAlreadyRetained)
    {
        _owner = owner;
        _handle = new VoxelWriteTokenHandle(lease, world, token, abort, leaseReferenceAlreadyRetained);
    }

    public bool IsCompleted => _handle.IsClosed;

    internal nint Handle
        => _handle.Token;

    internal void MarkCompleted() => _handle.MarkCompleted();

    internal void Abort()
    {
        var status = _handle.Abort();
        if (status != 0)
        {
            throw new VoxelNativeException(status, nameof(NativeVoxelWorld.Abort));
        }
    }

    internal bool IsOwnedBy(NativeVoxelWorld owner) => ReferenceEquals(_owner, owner);

    public void Dispose()
        => _handle.Dispose();
}

public sealed class NativeVoxelWorld
{
    private readonly Native.NativeEngineLease _lease;
    private readonly nint _world;
    private readonly Native.NativeEngineLoader.RootApi _api;

    internal NativeVoxelWorld(
        Native.NativeEngineLease lease,
        nint world,
        Native.NativeEngineLoader.RootApi api)
    {
        _lease = lease;
        _world = world;
        _api = api;
    }

    public static uint MaxCellsPerReadRequest => Native.AbiConstants.VoxelMaxCellsPerReadRequest;
    public static uint MaxEntriesPerWriteBatch => Native.AbiConstants.VoxelMaxEntriesPerWriteBatch;
    public static uint CellOffsetYStride => Native.AbiConstants.VoxelCellOffsetYStride;
    public static uint CellOffsetZStride => Native.AbiConstants.VoxelCellOffsetZStride;
    public static uint CellOffsetXStride => Native.AbiConstants.VoxelCellOffsetXStride;

    public VoxelBlockReadResult ReadCell(VoxelWorldCoordinate coordinate)
    {
        _lease.ThrowIfDisposed();
        var nativeCoordinate = new Native.VoxelWorldCoordinate
        {
            X = coordinate.X,
            Y = coordinate.Y,
            Z = coordinate.Z,
        };

        var coordinatePtr = Marshal.AllocHGlobal(Marshal.SizeOf<Native.VoxelWorldCoordinate>());
        var resultPtr = Marshal.AllocHGlobal(Marshal.SizeOf<Native.VoxelBlockReadCellResult>());
        try
        {
            Marshal.StructureToPtr(nativeCoordinate, coordinatePtr, false);
            var read = GetDelegate<VoxelReadCellFn>(_api.BlockReadCell, nameof(_api.BlockReadCell));
            ThrowIfFailed(read(_world, coordinatePtr, resultPtr), nameof(ReadCell));
            var result = Marshal.PtrToStructure<Native.VoxelBlockReadCellResult>(resultPtr);
            return ToPublic(result);
        }
        finally
        {
            Marshal.FreeHGlobal(coordinatePtr);
            Marshal.FreeHGlobal(resultPtr);
        }
    }

    public VoxelBatchReadResult ReadBox(
        VoxelBoxRequest request,
        VoxelBlockReadResult[] results,
        VoxelSectionSegment[] segments)
    {
        ArgumentNullException.ThrowIfNull(results);
        ArgumentNullException.ThrowIfNull(segments);
        return ReadBox(request, results.AsSpan(), segments.AsSpan());
    }

    public VoxelBatchReadResult ReadBox(
        VoxelBoxRequest request,
        Span<VoxelBlockReadResult> results,
        Span<VoxelSectionSegment> segments)
        => ReadBatch(
            new Native.VoxelBoxRequest
            {
                Min = ToNative(request.Min),
                Max = ToNative(request.Max),
            },
            _api.BlockReadBox,
            nameof(ReadBox),
            results,
            segments);

    public VoxelBatchReadResult ReadColumn(
        VoxelColumnRequest request,
        VoxelBlockReadResult[] results,
        VoxelSectionSegment[] segments)
    {
        ArgumentNullException.ThrowIfNull(results);
        ArgumentNullException.ThrowIfNull(segments);
        return ReadColumn(request, results.AsSpan(), segments.AsSpan());
    }

    public VoxelBatchReadResult ReadColumn(
        VoxelColumnRequest request,
        Span<VoxelBlockReadResult> results,
        Span<VoxelSectionSegment> segments)
        => ReadBatch(
            new Native.VoxelColumnRequest
            {
                X = request.X,
                Z = request.Z,
                MinY = request.MinY,
                MaxY = request.MaxY,
                Reserved = new byte[2],
            },
            _api.BlockReadColumn,
            nameof(ReadColumn),
            results,
            segments);

    public VoxelWriteToken PrepareWrite(
        ulong transactionId,
        VoxelBlockWriteEntry[] entries)
    {
        ArgumentNullException.ThrowIfNull(entries);
        return PrepareWrite(transactionId, entries.AsSpan());
    }

    public VoxelWriteToken PrepareWrite(
        ulong transactionId,
        ReadOnlySpan<VoxelBlockWriteEntry> entries)
    {
        _lease.ThrowIfDisposed();
        var entrySize = Marshal.SizeOf<Native.VoxelBlockWriteEntry>();
        var entryPtr = Marshal.AllocHGlobal(checked(entrySize * entries.Length));
        try
        {
            for (var index = 0; index < entries.Length; index++)
            {
                Marshal.StructureToPtr(ToNative(entries[index]), entryPtr + index * entrySize, false);
            }

            var prepare = GetDelegate<VoxelPrepareFn>(_api.BlockWritePrepare, nameof(_api.BlockWritePrepare));
            var abort = GetDelegate<VoxelAbortFn>(_api.BlockWriteAbort, nameof(_api.BlockWriteAbort));
            // Hold the image through native prepare and token-handle construction.
            // Disposal may mark the lease closed, but cannot unload while retained.
            _lease.RetainNativeToken();
            var leaseReferenceTransferred = false;
            nint token = 0;
            try
            {
                var status = prepare(_world, transactionId, entryPtr, checked((uint)entries.Length), out token);
                ThrowIfFailed(status, nameof(PrepareWrite));
                if (token == 0)
                {
                    throw new InvalidOperationException("Native voxel prepare returned a null token.");
                }

                var writeToken = new VoxelWriteToken(
                    this,
                    _lease,
                    _world,
                    token,
                    abort,
                    leaseReferenceAlreadyRetained: true);
                leaseReferenceTransferred = true;
                return writeToken;
            }
            catch
            {
                if (token != 0)
                {
                    try
                    {
                        abort(_world, token);
                    }
                    catch
                    {
                        // Preserve the prepare/constructor failure while making the
                        // best effort to consume a token that never reached a handle.
                    }
                }

                throw;
            }
            finally
            {
                if (!leaseReferenceTransferred)
                {
                    _lease.ReleaseNativeToken();
                }
            }
        }
        finally
        {
            Marshal.FreeHGlobal(entryPtr);
        }
    }

    public uint Commit(VoxelWriteToken token, VoxelWriteReceipt[] receipts)
    {
        ArgumentNullException.ThrowIfNull(receipts);
        return Commit(token, receipts.AsSpan());
    }

    public uint Commit(VoxelWriteToken token, Span<VoxelWriteReceipt> receipts)
    {
        ArgumentNullException.ThrowIfNull(token);
        if (!token.IsOwnedBy(this))
        {
            throw new ArgumentException("The write token belongs to a different voxel world.", nameof(token));
        }

        _lease.ThrowIfDisposed();
        var receiptSize = Marshal.SizeOf<Native.VoxelWriteReceipt>();
        var receiptPtr = Marshal.AllocHGlobal(checked(receiptSize * receipts.Length));
        var countPtr = Marshal.AllocHGlobal(sizeof(uint));
        try
        {
            var commit = GetDelegate<VoxelCommitFn>(_api.BlockWriteCommit, nameof(_api.BlockWriteCommit));
            var status = commit(_world, token.Handle, receiptPtr, checked((uint)receipts.Length), countPtr);
            ThrowIfFailed(status, nameof(Commit));
            var count = unchecked((uint)Marshal.ReadInt32(countPtr));
            if (count > (uint)receipts.Length)
            {
                throw new InvalidOperationException("Native voxel commit returned more receipts than the caller supplied.");
            }

            for (var index = 0; index < (int)count; index++)
            {
                receipts[index] = ToPublic(Marshal.PtrToStructure<Native.VoxelWriteReceipt>(receiptPtr + index * receiptSize));
            }

            token.MarkCompleted();
            return count;
        }
        finally
        {
            Marshal.FreeHGlobal(receiptPtr);
            Marshal.FreeHGlobal(countPtr);
        }
    }

    public void Abort(VoxelWriteToken token)
    {
        ArgumentNullException.ThrowIfNull(token);
        if (!token.IsOwnedBy(this))
        {
            throw new ArgumentException("The write token belongs to a different voxel world.", nameof(token));
        }

        token.Abort();
    }

    public VoxelSectionRevisionResult QuerySectionRevision(VoxelSectionKey sectionKey)
    {
        _lease.ThrowIfDisposed();
        var keyPtr = Marshal.AllocHGlobal(Marshal.SizeOf<Native.VoxelSectionKey>());
        var resultPtr = Marshal.AllocHGlobal(Marshal.SizeOf<Native.VoxelSectionRevisionResult>());
        try
        {
            Marshal.StructureToPtr(ToNative(sectionKey), keyPtr, false);
            var query = GetDelegate<VoxelRevisionFn>(_api.SectionRevisionQuery, nameof(_api.SectionRevisionQuery));
            ThrowIfFailed(query(_world, keyPtr, resultPtr), nameof(QuerySectionRevision));
            var result = Marshal.PtrToStructure<Native.VoxelSectionRevisionResult>(resultPtr);
            return new VoxelSectionRevisionResult(ToPublic(result.Presence), result.SectionRevision);
        }
        finally
        {
            Marshal.FreeHGlobal(keyPtr);
            Marshal.FreeHGlobal(resultPtr);
        }
    }

    private VoxelBatchReadResult ReadBatch<TRequest>(
        TRequest request,
        nint slot,
        string operation,
        Span<VoxelBlockReadResult> results,
        Span<VoxelSectionSegment> segments)
        where TRequest : struct
    {
        _lease.ThrowIfDisposed();
        var requestPtr = Marshal.AllocHGlobal(Marshal.SizeOf<TRequest>());
        var resultSize = Marshal.SizeOf<Native.VoxelBlockReadResult>();
        var segmentSize = Marshal.SizeOf<Native.VoxelSectionSegment>();
        var resultPtr = Marshal.AllocHGlobal(checked(resultSize * results.Length));
        var segmentPtr = Marshal.AllocHGlobal(checked(segmentSize * segments.Length));
        var resultCountPtr = Marshal.AllocHGlobal(sizeof(uint));
        var segmentCountPtr = Marshal.AllocHGlobal(sizeof(uint));
        var truncatedPtr = Marshal.AllocHGlobal(sizeof(byte));
        try
        {
            Marshal.StructureToPtr(request, requestPtr, false);
            var read = GetDelegate<VoxelReadBatchFn>(slot, operation);
            var status = read(
                _world,
                requestPtr,
                resultPtr,
                checked((uint)results.Length),
                resultCountPtr,
                segmentPtr,
                checked((uint)segments.Length),
                segmentCountPtr,
                truncatedPtr);
            ThrowIfFailed(status, operation);

            var resultCount = unchecked((uint)Marshal.ReadInt32(resultCountPtr));
            var segmentCount = unchecked((uint)Marshal.ReadInt32(segmentCountPtr));
            if (resultCount > (uint)results.Length || segmentCount > (uint)segments.Length)
            {
                throw new InvalidOperationException("Native voxel read returned more results than the caller supplied.");
            }

            for (var index = 0; index < (int)resultCount; index++)
            {
                results[index] = ToPublic(Marshal.PtrToStructure<Native.VoxelBlockReadResult>(resultPtr + index * resultSize));
            }

            for (var index = 0; index < (int)segmentCount; index++)
            {
                segments[index] = ToPublic(Marshal.PtrToStructure<Native.VoxelSectionSegment>(segmentPtr + index * segmentSize));
            }

            return new VoxelBatchReadResult(resultCount, segmentCount, Marshal.ReadByte(truncatedPtr) != 0);
        }
        finally
        {
            Marshal.FreeHGlobal(requestPtr);
            Marshal.FreeHGlobal(resultPtr);
            Marshal.FreeHGlobal(segmentPtr);
            Marshal.FreeHGlobal(resultCountPtr);
            Marshal.FreeHGlobal(segmentCountPtr);
            Marshal.FreeHGlobal(truncatedPtr);
        }
    }

    private static TDelegate GetDelegate<TDelegate>(nint slot, string slotName)
        where TDelegate : Delegate
    {
        if (slot == 0)
        {
            throw new InvalidOperationException($"Native voxel slot '{slotName}' is unavailable.");
        }

        return Marshal.GetDelegateForFunctionPointer<TDelegate>(slot);
    }

    private static void ThrowIfFailed(int status, string operation)
    {
        if (status != 0)
        {
            throw new VoxelNativeException(status, operation);
        }
    }

    private static Native.VoxelWorldCoordinate ToNative(VoxelWorldCoordinate value)
        => new() { X = value.X, Y = value.Y, Z = value.Z };

    private static Native.VoxelSectionKey ToNative(VoxelSectionKey value)
        => new() { X = value.X, Y = value.Y, Z = value.Z, Reserved = new byte[3] };

    private static Native.VoxelBlockWriteEntry ToNative(VoxelBlockWriteEntry value)
        => new()
        {
            SectionKey = ToNative(value.SectionKey),
            CellOffset = value.CellOffset,
            Reserved = new byte[2],
            BlockId = value.BlockId,
            ExpectedSectionRevision = value.ExpectedSectionRevision,
        };

    private static VoxelBlockReadResult ToPublic(Native.VoxelBlockReadCellResult value)
        => new(ToPublic(value.Presence), value.HasBlockId != 0, value.BlockId, value.SectionRevision);

    private static VoxelBlockReadResult ToPublic(Native.VoxelBlockReadResult value)
        => new(ToPublic(value.Presence), value.HasBlockId != 0, value.BlockId, value.SectionRevision);

    private static VoxelSectionSegment ToPublic(Native.VoxelSectionSegment value)
        => new(
            new VoxelSectionKey(value.SectionKey.X, value.SectionKey.Y, value.SectionKey.Z),
            ToPublic(value.Presence),
            value.SectionRevision,
            value.FirstResult,
            value.ResultCount);

    private static VoxelWriteReceipt ToPublic(Native.VoxelWriteReceipt value)
        => new(
            new VoxelSectionKey(value.SectionKey.X, value.SectionKey.Y, value.SectionKey.Z),
            value.UpToSectionRevision,
            value.WorldRevision);

    private static VoxelPresence ToPublic(Native.VoxelPresence value)
        => (VoxelPresence)(uint)value;

}

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int VoxelReadCellFn(nint world, nint coordinate, nint result);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int VoxelReadBatchFn(
    nint world,
    nint request,
    nint results,
    uint resultCapacity,
    nint resultCount,
    nint segments,
    uint segmentCapacity,
    nint segmentCount,
    nint truncated);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int VoxelPrepareFn(
    nint world,
    ulong transactionId,
    nint entries,
    uint entryCount,
    out nint token);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int VoxelCommitFn(
    nint world,
    nint token,
    nint receipts,
    uint capacity,
    nint count);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int VoxelAbortFn(nint world, nint token);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int VoxelRevisionFn(nint world, nint section, nint result);
