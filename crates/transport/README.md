# Transport

`--transport` reads one JSON object per stdin line in either rendered mode or
with `--norender`. Without `--transport`, neither mode uses stdin/stdout for
commands. `tick` is an absolute target frame. Each missing frame runs one Bevy update at a fixed 16,667 microseconds.
Frame 0 is reserved for startup and is not exposed as a tick.

Send `{"tick":3,"input":{"w":true,"shift":true,"look":[20,-5]}}` to
hold W and Shift through frames 1-3 and apply the mouse-look delta once, on
frame 1. `input` and each of its fields are optional. Valid fields are
`w`, `a`, `s`, `d`, `shift` (booleans) and `look` (two finite mouse-motion
values). Held keys persist across requests, including when `input` is omitted
or empty. Set an individual key to `false` to release it. `look` applies once
per request and does not persist. `{"tick":4}` advances one frame while retaining
held keys; `{"tick":5,"input":{"w":false}}` releases W. The controller has no
collision yet.

Each response includes `tick` and `player`: position in meters `[x,y,z]`, yaw
and pitch in radians, movement `[x,y]`, and running state. `player` is `null`
when no player exists (for example with a custom main plugin). Past or repeated
ticks, malformed JSON, invalid input, and unknown fields produce a JSON `error`
response without advancing or changing controls. Empty lines are ignored.
Each response is flushed. EOF exits successfully. Bevy logs go to stderr;
stdout is reserved for transport responses. Rendered transport keeps
Bevy's window event loop. Both modes wait for the core to signal readiness in
TransportTimeline; its first_tick offsets loading updates so gameplay starts at
tick 1. Loading updates do not consume requested ticks or apply their controls.
After readiness, idle window events do not count as simulation ticks. An asset
load failure exits the app with an error before transport becomes ready.
Rendering and video capture have not been visually verified in this mode. The
external Python launcher and agent manager are separate work.

Example (run by the user):

```sh
printf '{"tick":3,"input":{"w":true}}\n{"tick":4}\n' | cargo run -- --norender --transport
printf '{"tick":3,"input":{"w":true}}\n{"tick":4}\n' | cargo run -- --transport
```
