# Transport

`--norender` reads one JSON object per stdin line. Send `{"tick":1}` to advance
to absolute frame 1. Each valid request runs one Bevy update per missing frame
at a fixed 16,667 microseconds per frame and replies with `{"tick":1}` on
stdout. A later `{"tick":3}` runs frames 2 and 3 and replies with `{"tick":3}`.
Frame 0 is reserved for startup and is not exposed as a tick.

Past or repeated ticks, malformed JSON, and unknown fields produce a JSON
`error` response without advancing. Empty lines are ignored. Each response is
flushed. EOF exits successfully. No gameplay inputs or state snapshots are
implemented yet. Headless logging is disabled so stdout contains only transport
responses.

Example (run by the user):

```sh
printf '{"tick":1}\n{"tick":3}\n' | cargo run -- --norender
```
