# signal-listener

`signal-listener` is the ordinary Signal contract for the Listener component.
It carries start/stop/cancel/status capture requests, typed lifecycle conflict
replies, and implementation-failure replies. Runtime audio capture, durable disk
write, transcription, clipboard delivery, sockets, and state live in `listener`.

The checked-in generated schema artifact is refreshed with:

```sh
SIGNAL_LISTENER_UPDATE_SCHEMA_ARTIFACTS=1 cargo build --all-features
```
