# signal-listener - architecture

`signal-listener` is the ordinary peer-callable wire contract for Listener, the
speech-to-text component family.

## Role

The contract carries the working capture/transcription surface:

```text
ListenerOperation                 ListenerReply
Start(StartCapture)               Started(CaptureStarted) | MaintenanceLeaseActive(epoch)
Stop(StopCapture)                 CompletionRequested(CaptureCompletionRequested)
Cancel(CancelCapture)             CancellationRequested(CaptureCancellationRequested)
Toggle(ToggleCapture)             Started(CaptureStarted) | CompletionRequested(CaptureCompletionRequested)
Status(StatusRequest)             StatusReported(recording/finalizing/transcribing/delivered/error)
AcquireMaintenance({})            MaintenanceLeaseGranted(epoch), held by this connection
ReleaseMaintenance({})            MaintenanceLeaseReleased | MaintenanceLeaseNotHeld
                                   CaptureAlreadyActive(active session)
                                   NoActiveCapture
                                   CaptureSessionMismatch(active, requested)
                                   RequestUnimplemented(reason)
```

`Start` begins a daemon-owned capture session using the configured default input.
`Stop` promptly acknowledges its single accepted completion request, then the daemon
finalizes, transcribes, and delivers asynchronously exactly once. `Cancel` closes
that session while retaining the durable artifact and without requesting
transcription or delivery. `Toggle` starts an idle slot or requests graceful
completion; it never lowers to discard. `Status` reports the current lifecycle
phase without transcript text. The first output target is `SystemClipboard`.

`AcquireMaintenance` is a connection-bound, FIFO maintenance lease request. The
daemon waits event-driven for idle, gates new starts when the request is queue
front, then grants the epoch-scoped lease to that same live connection.
`ReleaseMaintenance` releases it explicitly; connection EOF and daemon restart
release or invalidate it automatically. No updater command travels in this
contract.

## Owned

- The ordinary Listener operation and reply vocabulary.
- The shared `ListenerDaemonConfiguration` record consumed by Listener startup
  and the meta contract.
- The generated `Frame` type over `signal-frame`.
- Round-trip witnesses for request/reply frames and NOTA projection.

## Not Owned

- Audio capture, PipeWire/WirePlumber/pipewire-pulse integration, BlueZ
  behavior, durable write implementation, transcription execution, and clipboard
  mutation live in `listener`.
- Owner-only configuration and policy traffic lives in `meta-signal-listener`.
- Schema generation machinery lives in `schema-rust`.

## Code Map

- `schema/lib.schema` is the authored contract vocabulary.
- `build.rs` runs the standard `schema-rust` contract driver.
- `src/schema/lib.rs` is the generated checked-in artifact.
- `src/lib.rs` re-exports generated nouns and adds small accessors.
- `tests/round_trip.rs` proves frame and NOTA round trips.

## Invariants

- This crate is wire-only: no daemon runtime, no actors, no storage, no Tokio.
- Runtime state names stay out of the public contract unless the caller must act
  on them.
- Default builds are binary-first; NOTA projection is behind `nota-text`.
