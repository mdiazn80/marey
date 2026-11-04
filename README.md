# Marey

An open-source desktop control panel for Leap Motion and Ultraleap hand tracking devices: service and
device status, a live 3D hand view, infrared camera images, tracking settings, logs and diagnostics.

Marey talks to the Ultraleap tracking service through LeapC. It does not include the service or
LeapC, and it does not track hands by itself. macOS on Apple silicon is the first supported platform.

> **Known issue:** with Ultraleap Hand Tracking 6.2.0, an original Leap Motion Controller is listed
> but sent no frames in our tests. See [Troubleshooting](#troubleshooting). Mock mode works without
> any device.

## Prerequisites

- Rust (stable) and the Xcode Command Line Tools, which provide libclang
- Node.js and pnpm
- Ultraleap Hand Tracking, which provides the service and the `LeapC.h` used at build time
- Optional: [Task](https://taskfile.dev) for the shortcuts below

## Setup and running

```bash
pnpm install
```

```bash
pnpm tauri dev
```

With Task, `task dev` does the same and `task --list` shows every operation.

### Mock mode

Run the app with synthetic hands and images, with no device or service:

```bash
MAREY_MOCK=1 pnpm tauri dev
```

A banner stays visible while it is on. Synthetic data never appears outside this mode.

To build without the Ultraleap SDK at all, which leaves only mock mode, start the frontend and the
backend separately:

```bash
pnpm dev
```

```bash
MAREY_MOCK=1 cargo run -p marey --no-default-features
```

### Configuration

| Variable           | Purpose                                                          |
| ------------------ | ---------------------------------------------------------------- |
| `LEAPSDK_LIB_PATH` | Directory holding the LeapC library, instead of the default path |
| `MAREY_MOCK`       | `1` turns on simulation; any other value leaves it off           |

## Development

```bash
task check
```

This runs the type check, formatting checks, Clippy, ESLint and the tests, none of which need a
device. After changing a DTO in `src-tauri/src/models`, run `task types` to regenerate the TypeScript
types.

See [ARCHITECTURE.md](docs/ARCHITECTURE.md), [ROADMAP.md](docs/ROADMAP.md) and the
[decision records](docs/adr).

## Troubleshooting

**"Ultraleap tracking software not found"**: Marey could not load LeapC. Install the software or set
`LEAPSDK_LIB_PATH`. The exact path it tried is shown in the message.

**Build fails with "LeapC.h was not found"**: install the software, set `LEAPSDK_LIB_PATH`, or build
with `--no-default-features`.

**"The tracking service is not running"**: start Ultraleap Hand Tracking. Marey reconnects by itself.

**"Device detected, but it is not streaming"**: the service lists the device but sends no frames.
Check whether the Ultraleap Control Panel shows hands. If it does not, the problem is between the
service and the device, not in Marey. Ultraleap's 6.x software is reported to target newer devices,
so an original Leap Motion Controller may need an older tracking version.

**Camera view stays on "Waiting for images"**: images need a streaming device. Settings shows a note
when the service does not report a requested option as on.

## Licence and third parties

Marey's own licence has not been chosen yet. Use of the Ultraleap software and SDK is governed by
Ultraleap's licence, shipped as `LeapSDK/LICENSE.md` in its installation. This project is not
affiliated with or endorsed by Ultraleap.
