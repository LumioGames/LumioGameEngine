namespace Lumio.Engine.NativeLoader;

public readonly struct NativeTimerHandle
{
    public NativeTimerHandle(uint index, uint generation, ulong context)
    { Index = index; Generation = generation; Context = context; }
    public uint Index { get; }
    public uint Generation { get; }
    public ulong Context { get; }
}

public readonly struct NativeTimerDrainRecord
{
    public NativeTimerDrainRecord(ulong due, ulong scheduleSequence, uint slotDispatchId)
    { Due = due; ScheduleSequence = scheduleSequence; SlotDispatchId = slotDispatchId; }
    public ulong Due { get; }
    public ulong ScheduleSequence { get; }
    public uint SlotDispatchId { get; }
}

public interface INativeTimerAbi
{
    int CreateManager(uint mode, out nint manager);
    int DestroyManager(nint manager);
    int RegisterDispatch(nint manager, uint dispatchId);
    int RegisterScope(nint manager, ulong scopeId, uint scopeKind, out uint generation);
    int TeardownScope(nint manager, ulong scopeId);
    int CreateSlot(nint manager, out nint slot);
    int BindSlot(nint manager, nint slot, uint dispatchId);
    int CloseSlot(nint manager, nint slot);
    int ScheduleOneShot(nint manager, ulong scopeId, uint scopeKind, uint scopeGeneration, ulong due, nint slot, out NativeTimerHandle handle);
    int ScheduleRepeating(nint manager, ulong scopeId, uint scopeKind, uint scopeGeneration, ulong firstDue, ulong interval, nint slot, out NativeTimerHandle handle);
    int Advance(nint manager, ulong toTick);
    int Pump(nint manager, ulong nowMs);
    int Cancel(nint manager, in NativeTimerHandle handle);
    int Drain(nint manager, Span<NativeTimerDrainRecord> records, out int count);
}
