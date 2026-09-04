using System.Runtime.InteropServices;

namespace Lumio.Engine.NativeLoader;

public sealed class NativeLoaderTimerAbi : INativeTimerAbi, IDisposable
{
    private readonly NativeEngineLease _lease;
    private readonly CreateManagerDelegate _create; private readonly DestroyManagerDelegate _destroy;
    private readonly RegisterDispatchDelegate _dispatch; private readonly RegisterScopeDelegate _scope;
    private readonly TeardownScopeDelegate _teardown;
    private readonly CreateSlotDelegate _slot; private readonly BindSlotDelegate _bind;
    private readonly CloseSlotDelegate _close; private readonly ScheduleOneShotDelegate _oneShot; private readonly ScheduleRepeatingDelegate _repeat; private readonly CancelDelegate _cancel; private readonly AdvanceDelegate _advance; private readonly PumpDelegate _pump; private readonly DrainDelegate _drain;
    private bool _disposed;
    private NativeLoaderTimerAbi(NativeEngineLease lease, CreateManagerDelegate create, DestroyManagerDelegate destroy, RegisterDispatchDelegate dispatch, RegisterScopeDelegate scope, TeardownScopeDelegate teardown, CreateSlotDelegate slot, BindSlotDelegate bind, CloseSlotDelegate close, ScheduleOneShotDelegate oneShot, ScheduleRepeatingDelegate repeat, CancelDelegate cancel, AdvanceDelegate advance, PumpDelegate pump, DrainDelegate drain)
    { _lease = lease; _create = create; _destroy = destroy; _dispatch = dispatch; _scope = scope; _teardown = teardown; _slot = slot; _bind = bind; _close = close; _oneShot = oneShot; _repeat = repeat; _cancel = cancel; _advance = advance; _pump = pump; _drain = drain; }

    public static NativeLoaderTimerAbi Load(string nativePath)
    {
        var lease = NativeEngineLoader.LoadFromBuildInfo(nativePath);
        var library = lease.LibraryHandle;
        if (library == 0) { lease.Dispose(); throw new InvalidOperationException("NativeEngineLease did not expose a loaded module handle."); }
        var entry = Marshal.GetDelegateForFunctionPointer<GetApiDelegate>(NativeLibrary.GetExport(library, AbiConstants.EntrySymbol));
        if (entry(AbiConstants.AbiVersion, out nint address) != 0 || address == 0) { lease.Dispose(); throw new InvalidOperationException("Native engine entry rejected ABI version."); }
        var table = Marshal.PtrToStructure<RootApiWithTimers>(address);
        if (table.StructSize < (uint)Marshal.SizeOf<RootApiWithTimers>() || table.TimerCreateManager == 0 || table.TimerDestroyManager == 0 || table.TimerRegisterDispatch == 0 || table.TimerRegisterScope == 0 || table.TimerTeardownScope == 0 || table.TimerCreateSlot == 0 || table.TimerBindSlot == 0 || table.TimerCloseSlot == 0 || table.TimerScheduleOneShot == 0 || table.TimerScheduleRepeating == 0 || table.TimerCancel == 0 || table.TimerAdvance == 0 || table.TimerPump == 0 || table.TimerDrain == 0) { lease.Dispose(); throw new InvalidOperationException("Native root table is missing timer_* slots."); }
        return new NativeLoaderTimerAbi(lease, D<CreateManagerDelegate>(table.TimerCreateManager), D<DestroyManagerDelegate>(table.TimerDestroyManager), D<RegisterDispatchDelegate>(table.TimerRegisterDispatch), D<RegisterScopeDelegate>(table.TimerRegisterScope), D<TeardownScopeDelegate>(table.TimerTeardownScope), D<CreateSlotDelegate>(table.TimerCreateSlot), D<BindSlotDelegate>(table.TimerBindSlot), D<CloseSlotDelegate>(table.TimerCloseSlot), D<ScheduleOneShotDelegate>(table.TimerScheduleOneShot), D<ScheduleRepeatingDelegate>(table.TimerScheduleRepeating), D<CancelDelegate>(table.TimerCancel), D<AdvanceDelegate>(table.TimerAdvance), D<PumpDelegate>(table.TimerPump), D<DrainDelegate>(table.TimerDrain));
        T D<T>(nint p) where T : Delegate => Marshal.GetDelegateForFunctionPointer<T>(p);
    }
    public int CreateManager(uint mode, out nint manager) => _create(mode, out manager);
    public int DestroyManager(nint manager) => _destroy(manager);
    public int RegisterDispatch(nint manager, uint dispatchId) => _dispatch(manager, dispatchId);
    public int RegisterScope(nint manager, ulong scopeId, uint scopeKind, out uint generation) => _scope(manager, scopeId, scopeKind, out generation);
    public int TeardownScope(nint manager, ulong scopeId) => _teardown(manager, scopeId);
    public int CreateSlot(nint manager, out nint slot) => _slot(manager, out slot);
    public int BindSlot(nint manager, nint slot, uint dispatchId) => _bind(manager, slot, dispatchId);
    public int CloseSlot(nint manager, nint slot) => _close(manager, slot);
    public int ScheduleOneShot(nint manager, ulong scopeId, uint scopeKind, uint scopeGeneration, ulong due, nint slot, out NativeTimerHandle handle) { var s = _oneShot(manager, scopeId, scopeKind, scopeGeneration, due, slot, out var h); handle = new NativeTimerHandle(h.Index, h.Generation, h.Context); return s; }
    public int ScheduleRepeating(nint manager, ulong scopeId, uint scopeKind, uint scopeGeneration, ulong firstDue, ulong interval, nint slot, out NativeTimerHandle handle) { var s = _repeat(manager, scopeId, scopeKind, scopeGeneration, firstDue, interval, slot, out var h); handle = new NativeTimerHandle(h.Index, h.Generation, h.Context); return s; }
    public int Advance(nint manager, ulong toTick) => _advance(manager, toTick);
    public int Pump(nint manager, ulong nowMs) => _pump(manager, nowMs);
    public int Cancel(nint manager, in NativeTimerHandle handle) { var h = new NativeTimerAbiHandle { Index = handle.Index, Generation = handle.Generation, Context = handle.Context }; return _cancel(manager, in h); }
    public int Drain(nint manager, Span<NativeTimerDrainRecord> records, out int count) { var b = new NativeDrainRecord[Math.Max(records.Length, 1)]; var pin = GCHandle.Alloc(b, GCHandleType.Pinned); try { var s = _drain(manager, pin.AddrOfPinnedObject(), (uint)records.Length, out var n); count = (int)n; for (var i = 0; i < Math.Min(count, records.Length); i++) records[i] = new NativeTimerDrainRecord(b[i].Due, b[i].ScheduleSequence, b[i].SlotDispatchId); return s; } finally { pin.Free(); } }
    public void Dispose() { if (!_disposed) { _disposed = true; _lease.Dispose(); } }
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int GetApiDelegate(uint version, out nint api);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int CreateManagerDelegate(uint mode, out nint manager);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int DestroyManagerDelegate(nint manager);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int RegisterDispatchDelegate(nint manager, uint id);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int RegisterScopeDelegate(nint manager, ulong id, uint kind, out uint generation);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int TeardownScopeDelegate(nint manager, ulong id);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int CreateSlotDelegate(nint manager, out nint slot);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int BindSlotDelegate(nint manager, nint slot, uint id);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int CloseSlotDelegate(nint manager, nint slot);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int ScheduleOneShotDelegate(nint manager, ulong scope, uint kind, uint generation, ulong due, nint slot, out NativeTimerAbiHandle handle);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int ScheduleRepeatingDelegate(nint manager, ulong scope, uint kind, uint generation, ulong due, ulong interval, nint slot, out NativeTimerAbiHandle handle);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int CancelDelegate(nint manager, in NativeTimerAbiHandle handle);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int AdvanceDelegate(nint manager, ulong tick);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int PumpDelegate(nint manager, ulong nowMs);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] private delegate int DrainDelegate(nint manager, nint records, uint capacity, out uint count);
    [StructLayout(LayoutKind.Sequential)] private struct NativeTimerAbiHandle { public uint Index, Generation; public ulong Context; }
    [StructLayout(LayoutKind.Sequential)] private struct NativeDrainRecord { public uint HandleIndex, HandleGeneration; public ulong HandleContext, Due, ScheduleSequence; public uint SlotDispatchId, Pad; }
    [StructLayout(LayoutKind.Sequential)] private struct RootApiWithTimers { public uint AbiVersion, StructSize; [MarshalAs(UnmanagedType.ByValArray, SizeConst=32)] public byte[] AbiHash; [MarshalAs(UnmanagedType.ByValArray, SizeConst=16)] public byte[] BuildId; public nint Ping, CreateClrHost, ClrHostCall, DestroyClrHost, TimerCreateManager, TimerDestroyManager, TimerRegisterDispatch, TimerRegisterScope, TimerTeardownScope, TimerCreateSlot, TimerBindSlot, TimerCloseSlot, TimerScheduleOneShot, TimerScheduleRepeating, TimerCancel, TimerAdvance, TimerPump, TimerDrain; }
}
