using System.Runtime.InteropServices;
using System.Runtime.CompilerServices;
using Sdk = Lumio.Engine.SDK;
using Xunit;

namespace Lumio.Engine.NativeLoader.Tests;

public sealed class VoxelFacadeTests
{
    private static readonly Sdk.VoxelReadCellFn ReadCell = ReadCellImpl;
    private static readonly Sdk.VoxelReadBatchFn ReadBatch = ReadBatchImpl;
    private static readonly Sdk.VoxelPrepareFn Prepare = PrepareImpl;
    private static readonly Sdk.VoxelPrepareFn PrepareWaitsForLeaseDispose = PrepareWaitsForLeaseDisposeImpl;
    private static readonly Sdk.VoxelCommitFn Commit = CommitImpl;
    private static readonly Sdk.VoxelAbortFn Abort = AbortImpl;
    private static readonly Sdk.VoxelRevisionFn Revision = RevisionImpl;
    private static int AbortCalls;
    private static ManualResetEventSlim? PrepareEntered;
    private static ManualResetEventSlim? LeaseDisposed;

    [Fact]
    public void CellReadPreservesPendingPresenceWithoutInventingAir()
    {
        var api = CreateApi(
            blockReadCell: Pointer(ReadCell));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);

        var result = world.ReadCell(new Sdk.VoxelWorldCoordinate(4, 255, -7));

        Assert.Equal(Sdk.VoxelPresence.Pending, result.Presence);
        Assert.False(result.HasBlockId);
        Assert.Equal(0u, result.BlockId);
        Assert.Equal(17ul, result.SectionRevision);
    }

    [Fact]
    public void BoxReadWritesCallerBuffersAndReturnsNativeCounts()
    {
        var api = CreateApi(blockReadBox: Pointer(ReadBatch));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);
        var cells = new Sdk.VoxelBlockReadResult[2];
        var segments = new Sdk.VoxelSectionSegment[1];

        var outcome = world.ReadBox(
            new Sdk.VoxelBoxRequest(new Sdk.VoxelWorldCoordinate(0, 1, 0), new Sdk.VoxelWorldCoordinate(1, 1, 0)),
            cells,
            segments);

        Assert.Equal(2u, outcome.ResultCount);
        Assert.Equal(1u, outcome.SegmentCount);
        Assert.True(outcome.Truncated);
        Assert.Equal(Sdk.VoxelPresence.Ready, cells[0].Presence);
        Assert.Equal(0x80000123u, cells[0].BlockId);
        Assert.Equal(22ul, cells[0].SectionRevision);
        Assert.Equal(2u, segments[0].ResultCount);
    }

    [Fact]
    public void WritePrepareCommitUsesOpaqueTokenAndCallerReceipts()
    {
        var api = CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteCommit: Pointer(Commit),
            blockWriteAbort: Pointer(Abort));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);
        var token = world.PrepareWrite(
            42,
            new[] { new Sdk.VoxelBlockWriteEntry(new Sdk.VoxelSectionKey(1, 2, 3), 4095, uint.MaxValue, 9) });
        var receipts = new Sdk.VoxelWriteReceipt[1];

        var count = world.Commit(token, receipts);

        Assert.Equal(1u, count);
        Assert.Equal(1, receipts[0].SectionKey.X);
        Assert.Equal(10ul, receipts[0].UpToSectionRevision);
        Assert.Equal(99ul, receipts[0].WorldRevision);
        Assert.True(token.IsCompleted);
    }

    [Fact]
    public void LateTokenDisposeAbortsAfterLeaseDisposeAndReleasesLeaseReference()
    {
        AbortCalls = 0;
        var lease = CreateLease(CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteAbort: Pointer(Abort)));
        var world = lease.CreateVoxelWorld((nint)0x1240);
        var token = world.PrepareWrite(43, Array.Empty<Sdk.VoxelBlockWriteEntry>());

        lease.Dispose();

        token.Dispose();
        token.Dispose();

        Assert.True(token.IsCompleted);
        Assert.Equal(1, AbortCalls);
        Assert.Equal(0, lease.ActiveNativeTokenCount);
    }

    [Fact]
    public async Task PrepareRetainsLeaseBeforeConcurrentDisposeCanUnloadNativeLibrary()
    {
        AbortCalls = 0;
        using var prepareEntered = new ManualResetEventSlim(false);
        using var leaseDisposed = new ManualResetEventSlim(false);
        var cancellationToken = TestContext.Current.CancellationToken;
        PrepareEntered = prepareEntered;
        LeaseDisposed = leaseDisposed;
        NativeEngineLease? lease = null;
        try
        {
            lease = CreateLease(CreateApi(
                blockWritePrepare: Pointer(PrepareWaitsForLeaseDispose),
                blockWriteAbort: Pointer(Abort)));
            var world = lease.CreateVoxelWorld((nint)0x1244);
            var prepareTask = Task.Run(
                () => world.PrepareWrite(46, Array.Empty<Sdk.VoxelBlockWriteEntry>()),
                cancellationToken);

            Assert.True(prepareEntered.Wait(TimeSpan.FromSeconds(5), cancellationToken));
            var disposeTask = Task.Run(
                () =>
                {
                    lease.Dispose();
                    leaseDisposed.Set();
                },
                cancellationToken);
            Assert.True(leaseDisposed.Wait(TimeSpan.FromSeconds(5), cancellationToken));
            await disposeTask;

            using var token = await prepareTask;
            Assert.False(token.IsCompleted);
            Assert.Equal(1, lease.ActiveNativeTokenCount);
            token.Dispose();
            Assert.Equal(1, AbortCalls);
            Assert.Equal(0, lease.ActiveNativeTokenCount);
            Assert.True(disposeTask.IsCompletedSuccessfully);
        }
        finally
        {
            PrepareEntered = null;
            LeaseDisposed = null;
            lease?.Dispose();
        }
    }

    [Fact]
    public void ForgottenTokenIsAbortedBySafeHandleFinalization()
    {
        AbortCalls = 0;
        var weakToken = CreateForgottenToken();
        for (var attempt = 0; attempt < 20 && weakToken.IsAlive; attempt++)
        {
            GC.Collect();
            GC.WaitForPendingFinalizers();
            GC.Collect();
            Thread.Sleep(10);
        }

        Assert.False(weakToken.IsAlive);
        Assert.Equal(1, AbortCalls);
    }

    [Fact]
    public void TokenCannotCrossVoxelWorldOwnershipBoundary()
    {
        AbortCalls = 0;
        using var lease = CreateLease(CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteAbort: Pointer(Abort)));
        var first = lease.CreateVoxelWorld((nint)0x1241);
        var second = lease.CreateVoxelWorld((nint)0x1242);
        using var token = first.PrepareWrite(44, Array.Empty<Sdk.VoxelBlockWriteEntry>());

        Assert.Throws<ArgumentException>(() => second.Abort(token));
        Assert.False(token.IsCompleted);
        Assert.Equal(1, lease.ActiveNativeTokenCount);
    }

    [Fact]
    public void SectionRevisionQueryPreservesNativePresenceAndRevision()
    {
        var api = CreateApi();
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);

        var result = world.QuerySectionRevision(new Sdk.VoxelSectionKey(-2, 15, 8));

        Assert.Equal(Sdk.VoxelPresence.Ready, result.Presence);
        Assert.Equal(27ul, result.SectionRevision);
    }

    [Fact]
    public void EveryContractStatusHasAStableManagedErrorCode()
    {
        for (var status = 1000; status <= 1050; status++)
        {
            Assert.True(Sdk.VoxelErrorCodeMap.TryMap(status, out var code), $"status {status}");
            Assert.Equal(status, Sdk.VoxelErrorCodeMap.ToStatus(code));
        }
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static WeakReference CreateForgottenToken()
    {
        var lease = CreateLease(CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteAbort: Pointer(Abort)));
        var world = lease.CreateVoxelWorld((nint)0x1243);
        var token = world.PrepareWrite(45, Array.Empty<Sdk.VoxelBlockWriteEntry>());
        lease.Dispose();
        return new WeakReference(token);
    }

    private static NativeEngineLease CreateLease(NativeEngineLoader.RootApi api)
        => new(0, api, "fake", "build", "abi", "sha", api.Ping);

    private static NativeEngineLoader.RootApi CreateApi(
        nint blockReadCell = 0,
        nint blockReadBox = 0,
        nint blockWritePrepare = 0,
        nint blockWriteCommit = 0,
        nint blockWriteAbort = 0)
        => new()
        {
            AbiVersion = AbiConstants.AbiVersion,
            StructSize = 304,
            AbiHash = new byte[32],
            BuildId = new byte[16],
            Ping = 1,
            CreateClrHost = 1,
            ClrHostCall = 1,
            DestroyClrHost = 1,
            BlockReadCell = blockReadCell,
            BlockReadBox = blockReadBox,
            BlockReadColumn = blockReadBox,
            BlockWritePrepare = blockWritePrepare,
            BlockWriteCommit = blockWriteCommit,
            BlockWriteAbort = blockWriteAbort,
            SectionRevisionQuery = Pointer(Revision),
        };

    private static nint Pointer(Delegate callback)
        => Marshal.GetFunctionPointerForDelegate(callback);

    private static int ReadCellImpl(nint _, nint __, nint result)
    {
        Marshal.StructureToPtr(new VoxelBlockReadCellResult
        {
            Presence = VoxelPresence.Pending,
            HasBlockId = 0,
            Reserved = new byte[3],
            BlockId = 0,
            SectionRevision = 17,
        }, result, false);
        return 0;
    }

    private static int ReadBatchImpl(nint _, nint __, nint results, uint resultCapacity, nint resultCount, nint segments, uint segmentCapacity, nint segmentCount, nint truncated)
    {
        Marshal.WriteInt32(resultCount, 2);
        Marshal.WriteInt32(segmentCount, 1);
        Marshal.WriteByte(truncated, 1);
        if (resultCapacity > 0)
        {
            Marshal.StructureToPtr(new VoxelBlockReadResult
            {
                Presence = VoxelPresence.Ready,
                HasBlockId = 1,
                Reserved = new byte[3],
                BlockId = 0x80000123,
                SectionRevision = 22,
            }, results, false);
        }

        if (segmentCapacity > 0)
        {
            Marshal.StructureToPtr(new VoxelSectionSegment
            {
                SectionKey = new VoxelSectionKey { X = 1, Y = 2, Z = 3, Reserved = new byte[3] },
                Presence = VoxelPresence.Ready,
                SectionRevision = 22,
                FirstResult = 0,
                ResultCount = 2,
            }, segments, false);
        }

        return 0;
    }

    private static int PrepareImpl(nint _, ulong __, nint ___, uint ____, out nint token)
    {
        token = (nint)0x99;
        return 0;
    }

    private static int PrepareWaitsForLeaseDisposeImpl(nint _, ulong __, nint ___, uint ____, out nint token)
    {
        token = (nint)0x9a;
        PrepareEntered?.Set();
        if (LeaseDisposed is null || !LeaseDisposed.Wait(TimeSpan.FromSeconds(5)))
        {
            throw new TimeoutException("The lease was not disposed while native prepare was in flight.");
        }

        return 0;
    }

    private static int CommitImpl(nint _, nint __, nint receipts, uint capacity, nint count)
    {
        Marshal.WriteInt32(count, 1);
        if (capacity > 0)
        {
            Marshal.StructureToPtr(new VoxelWriteReceipt
            {
                SectionKey = new VoxelSectionKey { X = 1, Y = 2, Z = 3, Reserved = new byte[3] },
                UpToSectionRevision = 10,
                WorldRevision = 99,
            }, receipts, false);
        }

        return 0;
    }

    private static int AbortImpl(nint _, nint __)
    {
        Interlocked.Increment(ref AbortCalls);
        return 0;
    }

    private static int RevisionImpl(nint _, nint __, nint result)
    {
        Marshal.StructureToPtr(new VoxelSectionRevisionResult
        {
            Presence = VoxelPresence.Ready,
            Reserved = new byte[4],
            SectionRevision = 27,
        }, result, false);
        return 0;
    }
}
