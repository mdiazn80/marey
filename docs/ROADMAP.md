# Roadmap

## v0.1 (current)

Service and device status, 3D hand visualizer, camera view, tracking settings, logs, diagnostics and
mock mode.

## Open problems

- **Original controller on service 6.2.0:** the service lists the device (PID `0x0003`), but in our
  tests it stayed at status `0x0` and sent no frames. It also reported the background frames policy
  as off after we requested it. See the README's troubleshooting section.
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
