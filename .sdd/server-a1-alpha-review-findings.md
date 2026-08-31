# Server A1-alpha Review Findings

Review target: `C:\\Users\\g923\\orca\\workspaces\\LumioServer\\server-a1-alpha-integration`

This is a review-only handoff. No source, test, Workflow, or candidate status was changed.

## P1: Synthetic Authority Revision

`mvp-host/src/Lumio.Server.MvpHost.Session/SessionRegistry.cs:810-817` increments
`authorityRevision` during reconnect when the world source has not advanced. This
creates a revision that is not owned or emitted by WorldSlot/Simulation. A later
`FullGraphComposition` poll reads the real simulation revision and calls
`NotifyAuthorityRevision`; the monotonic check then rejects the real value at
`:345-348`. The composition ignores that result at
`mvp-host/src/Lumio.Server.MvpHost.App/FullGraphComposition.cs:312-319`, so the
session can keep a fabricated revision and later deltas can collide with or skip
the actual world revision. The increment also survives a failed snapshot enqueue.

### Required fix

1. Remove the reconnect-side increment entirely. A reconnect FullSnapshot may use
   the current authoritative revision; uniqueness comes from its snapshot id and
   session epoch, not from inventing a world revision.
2. Keep `authorityRevision` writable only through `NotifyAuthorityRevision` while
   holding the owner gate. Treat a lower source revision as a fault/explicit
   rejected synchronization, never as a reason to advance locally.
3. In `FullGraphComposition`, update `observedRevision` only after an accepted
   `NotifyAuthorityRevision`; on rejection, surface the failure and stop/fault the
   owner loop rather than dropping the ack.
4. Replace the current tests that assert reconnect revision `> lastDeltaRevision`
   (`SessionBehaviorTests.cs:424-448` and
   `HostSmokeClientAcceptanceTests.cs:278-306`) with an assertion that the
   reconnect snapshot equals the latest WorldSlot/Simulation revision and that no
   `RevisionConflict` is ignored. Add a snapshot-enqueue-failure case proving the
   revision remains unchanged.

The existing `SendDelta` path also advances the session cursor only on DeltaAck.
That is acceptable if reconnect always snapshots from the authoritative source;
do not “fix” it by synthetic cursor/revision increments.

### Specification conflict that remains

The card/design's A1 step 14 explicitly requires the reconnect FullSnapshot
revision to be strictly greater than the last Delta (`docs/specs/2026-08-28-mvp-csharp-host-design.md:1219-1221`), while the same design requires the vector to be mechanically filled from the single owner `AuthorityRevision` and the reference simulation advances that value only on a real mutation. The updated integration assertion at
`mvp-host/tests/Lumio.Server.MvpHost.Integration.Tests/HostSmokeClientAcceptanceTests.cs:304-307`
now expects equality (`1`) in a no-post-disconnect-mutation scenario. That test
weakens a frozen acceptance condition. The correct resolution is an
authoritative owner-side publication/commit that genuinely advances the source
revision before reconnect, or an architecture/card revision explicitly relaxing
step 14; a Session-local increment is not a valid resolution. Until one of those
is published, keep this path `RETURN` rather than accepting the equality test.

## P1: Proof Constructor Trust Boundary

The current diff adds `InternalsVisibleTo` for the whole production
`Lumio.Server.MvpHost.Session` and `Lumio.Server.MvpHost.WorldSlot` assemblies in
`mvp-host/src/Lumio.Server.MvpHost.HostContracts/Lumio.Server.MvpHost.HostContracts.csproj:13-18`.
Because `TransportAuthenticationEvidence` has an `internal` constructor at
`SupportTypes.cs:40-52`, every type in those assemblies can manufacture a proof,
despite the requirement that only the transport carrier/adapter issue it. The
public `CarrierAccept.AuthenticationEvidence` and
`ConnectionEvent.HandshakeEnvelope.AuthenticationEvidence` init properties are
reference-carriage points, not issuer authentication; the one-shot bit prevents
reuse but cannot distinguish a fabricated proof from a carrier-issued one.

### Required fix

1. Remove production `InternalsVisibleTo` entries for `Session` and `WorldSlot`.
   Session already calls the public atomic `TryConsume`; WorldSlot has no reason
   to see this type. Keep only the WebSocket adapter friend (and an explicitly
   test-only friend if test construction is unavoidable).
2. Prefer a private constructor plus an internal carrier-only factory if the
   compiler shape permits it. If the current internal constructor is retained,
   add an architecture/reflection test that the only production source containing
   `new TransportAuthenticationEvidence` is
   `Transport.WebSocket/WebSocketByteCarrier.cs`, and that no Session/WorldSlot
   production assembly is an IVT friend.
3. Keep evidence immutable and one-shot. Do not add a public constructor or a
   public factory. If `TryConsume` is made internal, split construction and
   consumption into separate capability types; do not re-add Session IVT merely
   to reach the constructor.
4. Add a test that reflection over production assemblies finds no proof
   constructor/factory use outside the carrier, while the existing replay tests
   continue to prove atomic single consumption.

## P1/P2: Transport Event-Overflow Retirement

The latest overlay at `TransportService.cs:512-548` now closes ingress/egress,
calls `carrier.Close`, cancels the idle timer, reserves a terminal Closed event,
and removes the registry entry when a non-terminal event cannot enter the event
outbox. This resolves the previously observed overflow leak. Preserve this
ordering: emit/reserve the terminal event before removing the entry, then make
all later operations return `StaleConnectionGeneration`.

Two residual checks remain:

* `TransportService.Dispose()` (`:345-360`) closes queues and removes entries but
  does not call `carrier.Close` or cancel each entry's idle timer. A standalone
  TransportService disposal can therefore leave the underlying carrier/socket and
  timer alive. Add a disposal test with a counting carrier/timer and perform the
  same close/cancel sequence before registry removal. The composition currently
  disposes the carrier separately, but that is not a substitute for the
  Transport service's own lifecycle contract.
* The terminal reserve is an unbounded `Queue<ConnectionEvent>`. If the event
  outbox remains full while many connections overflow, reserve memory is
  unbounded. Bound it to the documented reserved capacity (at least one slot per
  live terminal event), and on reserve exhaustion escalate via the documented
  fail-stop/diagnostic path rather than silently dropping `Closed`.

The current `BoundedQueueTest` only checks registry count and event presence;
extend it to assert carrier close, timer cancellation, and stale-command
rejection after both normal close and overflow fallback.

## Current Decision

Until the synthetic revision path and production IVT boundary are corrected and
independently re-reviewed, the Server candidate remains `RETURN`/unaccepted.
The Runtime command candidate `79528044f758d188844270bc7e55decce2a7b0cc` remains
unaccepted, and R-00141 remains blocked by the unpublished executable
`LumioBinV1` authority.

## Follow-up Snapshot (19:00 CST)

The shared overlay now removes the reconnect-side revision increment and changes
`FullGraphComposition` to retain `observedRevision` only after an accepted
`NotifyAuthorityRevision`; a rejected synchronization marks the owner fatal.
The HostContracts project currently grants internals only to the WebSocket carrier
and the session test assembly, so Session/WorldSlot can no longer call the proof
constructor. Transport overflow fallback and `Dispose()` now close the carrier,
cancel idle timers, clear queues, reserve a terminal event, and remove the entry.

The same snapshot exposes a new contract concern: `IWorldSlotReleasePort` is now
`public` and `WorldSlotCommand.ReleaseAdmission` is a public nested command
(`mvp-host/src/Lumio.Server.MvpHost.HostContracts/WorldSlotContracts.cs:21-24,
:113-124`). The card freezes the public admission command set and describes
release as cleanup, so this broadens the cross-module public surface to solve the
friend-assembly compile issue. Either keep the release capability internal while
providing a narrowly scoped bridge that does not expose proof construction, or
obtain an explicit contract revision; do not silently accept the added public
command/interface as a local implementation detail.

The compile-time reason is visible in `SessionRegistry`: Session must access the
release capability, while the HostContracts IVT list is also the proof
constructor boundary. A lower-risk shape is to keep Session off the IVT list,
make `ReleaseAdmission` a non-public nested command, and expose only the narrow
cleanup capability needed by the composition (or move that capability behind an
explicitly authorized bridge). Re-adding Session IVT solely to hide the release
port would reopen the proof-forgery path.

There is a parallel public-surface expansion in
`mvp-host/src/Lumio.Server.MvpHost.HostContracts/SessionContracts.cs:42-51`:
`ConnectionTerminated` is public, while the session card explicitly consumes
`SessionCommand (six derived commands)`. This makes a transport-internal termination command
part of the frozen cross-module contract. Keep it internal (or use an explicitly
authorized event/bridge) and add a shape test counting the six public derived
commands; otherwise the contract gate should remain `RETURN`.

These are implementation improvements, but they are not yet an independent
acceptance: the overlay remains uncommitted and requires fresh Release builds,
the full host verification script, and targeted tests for the exact revision and
issuer-boundary assertions. The terminal reserve remains an unbounded queue and
should be bounded or explicitly justified before final acceptance.

## Fresh Verification Failure (19:05 CST)

Command:

`dotnet test tests/Lumio.Server.MvpHost.Session.Tests/Lumio.Server.MvpHost.Session.Tests.csproj -c Release --no-restore -p:BuildInParallel=false`

Exit code: `1`. Production projects built, but the test project did not compile:
`SessionBehaviorTests.cs:212` reports `CS0308` because `Assembly.GetCustomAttributes<T>()`
was resolved to the non-generic `GetCustomAttributes(bool)` API; line `213` then
reports `CS1061` on `AssemblyName`. The newly added friend-boundary test therefore
has no green evidence yet and must be corrected (for example, use
`GetCustomAttributes(typeof(InternalsVisibleToAttribute))` and cast) before any
review claim can advance.

The same snapshot did pass fresh Transport (`42/42`) and WorldSlot (`25/25`)
Release tests, but those green counts do not offset the Session compile failure.

## Follow-up Compile Failure (19:13 CST)

While the fixer was restoring the frozen public command shape, a fresh build of
`src/Lumio.Server.MvpHost.Session/Lumio.Server.MvpHost.Session.csproj` exited `1`:
`SessionRegistry.cs:42` reports `CS0426` because `SessionCommand.ConnectionTerminated`
is no longer defined in HostContracts. The new `Observability/ICommittedReservationReleasePort`
file is present but is not referenced by Session/WorldSlot, so it does not yet
resolve either the termination command or release path. This is a real broken
intermediate overlay; do not review it as accepted until the bridge is fully
wired and the complete solution builds.

## Additional P1: Delta ACK Is Not Correlated

`SessionRegistry.HandleInboundCore` (`:316-329`) parses only `toRevision` from a
`DeltaAck` and `ServerConnectionSession.TryAcknowledgeDelta` accepts any value
greater than or equal to `LastSnapshotRevision`. It never records or checks the
`confirmationSequence` emitted by `MvpEnvelopeWriter.WriteDelta`, nor the
`fromRevision`/base snapshot of the outstanding delta. A client can therefore
send a schema-valid ACK for a revision it never received (for example, ACK
`toRevision=100` while the server's cursor is at 0); the session advances its
cursor and the next real update starts at 100, silently skipping revisions 1-99.
Track a bounded pending-delta identity on successful enqueue, parse both ACK
fields, and accept only the matching sequence/toRevision (with an explicitly
idempotent replay rule). Add a regression test for a forged/jumped ACK and for
an ACK arriving after reconnect.

## Additional P1: Owner-Lane and Cleanup Failure Semantics

`SessionRegistry` uses `ownerGate` as a mutual-exclusion lock, but it does not
identify or enforce the named owner thread. `HandleConnectionEvent` routes a
termination command and immediately calls `PumpOnce` (`SessionRegistry.cs:480-512`),
and `BeginDrain`/`Kick` do the same (`:405-433`). A transport callback or admin
thread can therefore execute the reducer and mutate session maps on a non-owner
thread; the lock serializes memory access but does not satisfy the owner-lane
contract or prevent cross-owner reentrancy. Either make those paths enqueue-only
and let the owner pump consume them, or add a concrete owner-thread assertion
and a typed completion mechanism that never runs `Process` on the caller thread.

Release compensation remains lossy: `Compensate` and
`ReleaseCommittedReservation` log a failed `ReleaseAdmission` but remove the
admission/session tracking or transition the session terminal anyway
(`SessionRegistry.cs:1413-1469`). If the WorldSlot queue is closed or the epoch is
stale, the committed reservation can remain occupied with no retry record,
eventually exhausting capacity. Keep a bounded retry/dead-letter record and
escalate a non-recoverable release failure before dropping the tracking entry;
add tests for queue-full, stale-epoch, and repeated cleanup calls.

`WriteAdminAudit` increments `auditSequence` outside `ownerGate`
(`SessionRegistry.cs:1562-1591`). Concurrent admin calls can emit duplicate or
out-of-order event sequences. Allocate the sequence atomically or under the
owner lane, and test parallel admin calls for unique monotonic audit IDs.
