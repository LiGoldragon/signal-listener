# signal-listener - architecture

`signal-listener` is the ordinary peer-callable wire contract for Listener, the
speech-to-text component family.

## Role

The contract carries the working capture/transcription surface:

```text
ListenerOperation                 ListenerReply
Start(StartCapture)               Started(CaptureStarted)
Stop(StopCapture)                 Stopped(CaptureStopped)
Cancel(CancelCapture)             Cancelled(CaptureCancelled)
Status(StatusRequest)             StatusReported(CaptureStatusReport)
                                   CaptureAlreadyActive(active session)
                                   NoActiveCapture
                                   CaptureSessionMismatch(active, requested)
                                   RequestUnimplemented(reason)
```

`Start` begins a daemon-owned capture session using the configured default input.
`Stop` closes that session, allowing the daemon to transcribe the durable capture
and deliver text to the configured outputs. `Cancel` closes that session while
retaining the durable artifact and without requesting transcription or delivery.
`Status` reports whether the single active capture slot is idle or writing one
durable audio artifact. The first output target in the contract is
`SystemClipboard`.

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
- Schema generation machinery lives in `schema-rust-next`.

## Code Map

- `schema/lib.schema` is the authored contract vocabulary.
- `build.rs` runs the standard `schema-rust-next` contract driver.
- `src/schema/lib.rs` is the generated checked-in artifact.
- `src/lib.rs` re-exports generated nouns and adds small accessors.
- `tests/round_trip.rs` proves frame and NOTA round trips.

## Invariants

- This crate is wire-only: no daemon runtime, no actors, no storage, no Tokio.
- Runtime state names stay out of the public contract unless the caller must act
  on them.
- Default builds are binary-first; NOTA projection is behind `nota-text`.
