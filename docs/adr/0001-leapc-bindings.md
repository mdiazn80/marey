# ADR 0001: LeapC bindings

Status: accepted, 2026-11-04

## Context

Marey talks to the Ultraleap tracking service through the LeapC C API. The app must start and
explain the problem when the Ultraleap software is missing, and it must not bundle or redistribute
`libLeapC.dylib`. The machine used for this decision has Ultraleap Hand Tracking 6.2.0 on arm64.

## Evaluation of `leaprs`

`leaprs` 0.2.2 (June 2024), built on `leap-sys` 0.2 (July 2022), was tested on
`aarch64-apple-darwin`:

- It compiles, and the five checked struct sizes match the installed `LeapC.h`.
- Its pregenerated bindings predate this SDK. They lack `eLeapDevicePID_LMC2`,
  `eLeapEventType_Fiducial`, `LeapSetDeviceHints` and `LeapGetDeviceTemperatures`, and `leaprs` maps
  unknown PIDs to `Unknown`, so a Leap Motion Controller 2 would not be identified.
- It links `libLeapC` at build time. The binary then fails at launch when the library is missing,
  before any UI can explain why, and it needs `DYLD_LIBRARY_PATH` or an rpath to run.

## Decision

Use two internal crates:

- `crates/leapc-sys` generates bindings with `bindgen` from the `LeapC.h` on the build machine, in
  **dynamic loading** mode.
- `crates/leapc` is a thin safe wrapper. It resolves the library at runtime from
  `LEAPSDK_LIB_PATH`, then the default platform path.

The header and the generated code are not committed.

This departs from the initial brief in one way: the brief asked for an rpath, so that `pnpm tauri
dev` works without manual steps. Loading by absolute path meets that goal without any rpath, and it
also gives a clear "software not found" state instead of a launch failure.

## Consequences

- A missing SDK is a normal application state, retried every few seconds.
- Symbols absent from another SDK version produce an error when used, not at load time.
- Building needs the Ultraleap software and libclang (included with the Xcode tools). Without the
  SDK, build with `--no-default-features`, which leaves only mock mode.
- A struct layout change between the build machine's SDK and the user's SDK would not be detected at
  load time. `LEAP_DEVICE_INFO` and similar structs carry a `size` field, but event structs do not.
  The client and service versions are shown in Diagnostics to help spot this.
