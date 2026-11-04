# Roadmap

## v0.1 (current)

Service and device status, 3D hand visualizer, camera view, tracking settings, logs, diagnostics and
mock mode.

## Open problems

- **Stream stalls on an original controller with service 6.2.0:** frames stopped twice on one day. The
  service stayed connected, kept listing the device and used no CPU, and the Control Panel stopped
  updating as well. Replugging restored the stream the first time, and the service also crashed and
  restarted at about that moment (exit code 5, with a dump in `/etc/ultraleap/crashes`). The cause of
  the stalls is unknown.
- **Device status on the original controller:** it stays at `0x0` while streaming, and field of view
  and range read 0. Do not use the streaming flag to decide whether data is flowing.
- **Background frames policy:** the service reports it as off after we request it, yet frames still
  reached unfocused clients in our tests.
- **Firmware version:** LeapC 6.2 has no public call for it, so the UI says it is not exposed.

## Extension points, not implemented

| Feature                                                   | Where it plugs in                                                 |
| --------------------------------------------------------- | ----------------------------------------------------------------- |
| Recording and playback (LeapC recording API)              | A new `TrackingSource` for playback; an `EventSink` for recording |
| Hyperion: Hinting API, tracking models, performance modes | New `Command` variants and `leapc::Connection` methods            |
| AR marker tracking (fiducials)                            | A connection config flag and a new `SourceEvent`                  |
| Device transform and multi-device fusion                  | `Core`, which already tracks several devices                      |
| Gesture detection and pose recognition                    | A consumer of `TrackingFrame`, independent of Tauri               |
| Output bridges: OSC, MIDI, WebSocket, OpenXR              | Additional `EventSink` implementations                            |
| Projected hand points on camera images                    | Marked in `CamerasView.tsx` (`LeapRectilinearToPixel`)            |
| Raw camera access outside LeapC (UVC/libusb)              | A new `TrackingSource`; see the licensing note below              |
| Windows and Linux packaging                               | Paths in `crates/leapc` and `crates/leapc-sys/build.rs`           |

## Licensing note for future work

The Ultraleap SDK licence restricts using the SDK or its Image API to build hand tracking or
competing software. Keep any independent tracking work separate from the SDK and from images obtained
through it, and review the licence before that work starts or before publishing this code.
