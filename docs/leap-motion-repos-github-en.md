# Leap Motion / Ultraleap — GitHub repos as reference sources

Search date: 2026-11-04 (GitHub API, ~400 repos reviewed; noise such as leap.nvim, Magic Leap or leapcast filtered out).
Format: ⭐ stars · language · last push. ARCH = archived.

## Key context before choosing sources
- There are three software generations: **SDK v2/v3** (old C++/Python API with built-in gestures), **Orion v4** (included a WebSocket server used by LeapJS) and **Gemini v5+** (C API **LeapC**, no built-in WebSocket). Most older repos depend on v2/v3 or on the v4 WebSocket.
- For a new project, the sensible foundation is **LeapC (Gemini)**. If you follow the Myo project's stack (Tauri + Rust), the natural path is `leaprs`.

## Top picks to start with
1. **plule/leaprs** — ⭐16 · Rust · 2025-03 — safe LeapC wrapper on top of `leap-sys`; works on Linux and Windows. Low-level, no worker thread or async API (you would build that yourself, as you did with btleplug). https://github.com/plule/leaprs
2. **ultraleap/leapc-python-bindings** — ⭐87 · Python · 2024-06 — official Ultraleap bindings for Gemini LeapC, with examples. Ideal for fast prototyping. https://github.com/ultraleap/leapc-python-bindings
3. **ultraleap/UltraleapTrackingWebSocket** — ⭐17 · C · 2025-02 — LeapJS-compatible WebSocket server (protocol v6) for Gemini v5+; brings back the whole older web ecosystem. https://github.com/ultraleap/UltraleapTrackingWebSocket
4. **leapmotion/leapjs** — ⭐2008 · JS · 2025-02 — official JavaScript client; the best-documented frame/hand/finger data model. https://github.com/leapmotion/leapjs
5. **ksandom/handWavey** — ⭐24 · Java · 2025-11 — desktop control by hand (mouse, clicks, gestures), Linux X / Windows / macOS. The closest thing to a "Myo Connect" for the Leap. https://github.com/ksandom/handWavey
6. **leapmotion/leapuvc** — ⭐126 · Python · 2024-04 — access to the Leap Motion Controller's IR images via UVC: LED brightness, exposure, gain, distortion, stereo depth maps, ArUco. Opens uses beyond hand tracking (requires a firmware update from Windows; Leap SDK license). https://github.com/leapmotion/leapuvc

## 1. SDKs and bindings
| Repo | ⭐ | Language | Push | Note |
|---|---|---|---|---|
| [plule/leaprs](https://github.com/plule/leaprs) | 16 | Rust | 2025-03 | Safe LeapC wrapper (see above) |
| [urholaukkarinen/leap-sys](https://github.com/urholaukkarinen/leap-sys) | 7 | Rust | 2022-07 | Generated FFI bindings for LeapC (basis of leaprs) |
| [SludgePhD/leapcpp-rs](https://github.com/SludgePhD/leapcpp-rs) | 0 | Rust | 2022 | Bindings to the legacy SDK (v2/v3) |
| [ultraleap/leapc-python-bindings](https://github.com/ultraleap/leapc-python-bindings) | 87 | Python | 2024-06 | Official, Gemini |
| [leapmotion/LeapCxx](https://github.com/leapmotion/LeapCxx) | 53 | C++ | 2018 | Implements the old C++ API on top of LeapC (bridge for v2/v3 code) |
| [ano0002/Leap-Motion-Python-3.8](https://github.com/ano0002/Leap-Motion-Python-3.8) | 23 | Python | 2025-01 | SDK v3.2 recompiled for Python 3.8 |
| [Cipulot/Leap-Motion-Python-3](https://github.com/Cipulot/Leap-Motion-Python-3) | 13 | Python | 2021 | Python 3 wrappers for the old SDK |
| [blacklight/leap-sdk-python3](https://github.com/blacklight/leap-sdk-python3) | 17 | Makefile | 2020 | Python 3 module builder |
| [ultraleap/UnityPlugin](https://github.com/ultraleap/UnityPlugin) | 597 | C# | 2026-07 | Official Unity SDK (the best maintained; useful as an interaction reference) |
| [ultraleap/UnrealPlugin](https://github.com/ultraleap/UnrealPlugin) | 282 | C++ | 2024-06 | Official Unreal SDK |

## 2. Web and WebSocket
| Repo | ⭐ | Language | Push | Note |
|---|---|---|---|---|
| [leapmotion/leapjs](https://github.com/leapmotion/leapjs) | 2008 | JS | 2025-02 | Official client |
| [ultraleap/UltraleapTrackingWebSocket](https://github.com/ultraleap/UltraleapTrackingWebSocket) | 17 | C | 2025-02 | LeapJS-compatible server for Gemini |
| [leapmotion/javascript](https://github.com/leapmotion/javascript) | 121 | JS | 2023 | LeapJS examples |
| [leapmotion/leapjs-rigged-hand](https://github.com/leapmotion/leapjs-rigged-hand) | 102 | CoffeeScript | 2023 | Rigged 3D hand (visualization in the UI) |
| [leapmotion/leapjs-playback](https://github.com/leapmotion/leapjs-playback) | 48 | JS | 2016 | Record and play back frames (model for session recording) |
| [leapmotion/leapjs-network](https://github.com/leapmotion/leapjs-network) | 36 | JS | 2014 | Send data P2P over WebRTC |
| [openleap/aframe-leap-hands](https://github.com/openleap/aframe-leap-hands) | 166 | JS | 2018 | A-Frame VR component |
| [spren9er/threeleapcontrols](https://github.com/spren9er/threeleapcontrols) / [leapmotion/Leap-Three-Camera-Controls](https://github.com/leapmotion/Leap-Three-Camera-Controls) | 134 / 63 | JS | 2014–16 | Three.js camera controls |
| [ultraleap/TouchFreeWebTooling](https://github.com/ultraleap/TouchFreeWebTooling) | 16 | TS | 2024-05 | Tooling for touchless web apps |

## 3. Desktop / mouse control
| Repo | ⭐ | Language | Push | Note |
|---|---|---|---|---|
| [ksandom/handWavey](https://github.com/ksandom/handWavey) | 24 | Java | 2025-11 | The most complete and maintained |
| [ksandom/installUltraleap](https://github.com/ksandom/installUltraleap) | 14 | Shell | 2025-11 | Scripts to install Ultraleap software (useful on Linux) |
| [vhaloo/LeapController](https://github.com/vhaloo/LeapController) | 4 | Python | 2026-06 | Gesture-based mouse controller, full-featured and recent |
| [konarkm/leap-control](https://github.com/konarkm/leap-control) | 0 | Python | 2026-04 | Gesture-first local controller for macOS |
| [openleap/PyLeapMouse](https://github.com/openleap/PyLeapMouse) | 174 | Python | 2021 | Leap mouse in Python (classic) |
| [rpav/xleapmouse](https://github.com/rpav/xleapmouse) | 23 | C++ | 2019 | Mouse for Linux/X with Qt GUI |
| [ultraleap/TouchFree](https://github.com/ultraleap/TouchFree) | 28 | C# | ARCH | Official Windows service turning tracking into cursor/touches |
| [vdods/SwipeyJoeMcDesktop](https://github.com/vdods/SwipeyJoeMcDesktop) | 13 | Python | 2015 | Gesture sequences → keyboard shortcuts |
| [hsocasnavarro/DesktopLeapr](https://github.com/hsocasnavarro/DesktopLeapr) | 78 | Python | 2013 | Desktop navigator for Linux |
| [clwillingham/LEAP-OSXGestureController](https://github.com/clwillingham/LEAP-OSXGestureController) | 40 | C++ | 2013 | Spaces/Mission Control on macOS |

## 4. Music, art and OSC / MIDI outputs
| Repo | ⭐ | Language | Push | Note |
|---|---|---|---|---|
| [genekogan/LeapMotionOSC](https://github.com/genekogan/LeapMotionOSC) | 23 | C++ | 2017 | OSC streaming + derived statistics |
| [n1ckfg/ManosOsc](https://github.com/n1ckfg/ManosOsc) | 59 | Processing | 2019 | OSC, MIDI and AE/Maya animation scripts |
| [balandinodidonato/LeapMapper](https://github.com/balandinodidonato/LeapMapper) | 16 | Processing | 2017 | Mapping to MIDI and OSC (same author as MyoMapper → common pattern for both devices) |
| [cpmpercussion/ultraleap-osc-sender](https://github.com/cpmpercussion/ultraleap-osc-sender) | 2 | Python | 2024-05 | OSC sender for modern Ultraleap (same author as myo-to-osc) |
| [ThatGuyThimo/leapmotion-osc](https://github.com/ThatGuyThimo/leapmotion-osc) | 84 | C# | 2023-10 | Finger tracking for VRChat via OSC |
| [revmischa/leapmidi](https://github.com/revmischa/leapmidi) | 23 | C++ | 2014 | Gesture-based MIDI controller |
| [genekogan/LeapMotion-SuperCollider-Instruments](https://github.com/genekogan/LeapMotion-SuperCollider-Instruments) | 23 | SuperCollider | 2021 | Sound instruments |
| [celtera/ultraleap](https://github.com/celtera/ultraleap) | 27 | Max | 2026-07 | Ultraleap objects for Max (current) |
| [JulesFrancoise/leapmotion-for-max](https://github.com/JulesFrancoise/leapmotion-for-max) | 25 | Max | 2023 | Max external |
| [vvvv/VL.Devices.Ultraleap](https://github.com/vvvv/VL.Devices.Ultraleap) | 12 | C# | 2024-11 | Package for vvvv (VJ) |
| [ofTheo/ofxLeapMotion](https://github.com/ofTheo/ofxLeapMotion) / [genekogan/ofxLeapMotion2](https://github.com/genekogan/ofxLeapMotion2) | 136 / 61 | C++ | 2018 | openFrameworks addons |
| [geekychris/leap_motion_harp](https://github.com/geekychris/leap_motion_harp) | 0 | Rust | 2026-10 | Laser harp in Rust (recent Leap + Rust example) |

## 5. Gesture recognition and data
| Repo | ⭐ | Language | Push | Note |
|---|---|---|---|---|
| [roboleary/LeapTrainer.js](https://github.com/roboleary/LeapTrainer.js) | 370 | JS | 2015 | Gesture/pose learning and recognition (template-based) |
| [kaathleen/LeapGesture-library](https://github.com/kaathleen/LeapGesture-library) | 37 | C++ | 2014 | Gesture recognition library |
| [uacaps/MotionGestureRecognizers-CSharp](https://github.com/uacaps/MotionGestureRecognizers-CSharp) | 42 | C# | 2013 | UIGestureRecognizer-style gestures |
| [niharikabalachandra/Gesture-Recognition-Using-RNN](https://github.com/niharikabalachandra/Gesture-Recognition-Using-RNN) | 20 | Python | 2017 | RNN on 3D Leap data |
| [LookUpMark/dylem-grid](https://github.com/LookUpMark/dylem-grid) | 1 | Python | 2026-01 | Dynamic gestures with BiLSTM |
| [seanschneeweiss/RoSeMotion](https://github.com/seanschneeweiss/RoSeMotion) | 23 | Python | 2024-03 | Hand motion capture for musculoskeletal simulation |
| [ApocalyVec/mGesf](https://github.com/ApocalyVec/mGesf) | 42 | Python | 2023 | Sensor fusion for micro-gestures (idea applicable to Leap + Myo) |
| [VenkateshBH99/Continuous-Sign-Language-using-Leap-Motion-Sensor](https://github.com/VenkateshBH99/Continuous-Sign-Language-using-Leap-Motion-Sensor) | 16 | Notebook | 2020 | Continuous sign language |

## 6. VR / AR and XR
| Repo | ⭐ | Language | Push | Note |
|---|---|---|---|---|
| [leapmotion/ProjectNorthStar](https://github.com/leapmotion/ProjectNorthStar) | 1833 | C | 2025-10 | Leap Motion's open-source AR headset |
| [SDraw/driver_leap](https://github.com/SDraw/driver_leap) | 161 | C++ | ARCH | SteamVR driver for Leap Motion |
| [jcorvinus/HandshakeVR](https://github.com/jcorvinus/HandshakeVR) | 63 | C# | 2024 | Leap ↔ SteamVR Skeletal Input compatibility layer |
| [ultraleap/OpenXRHandTracking](https://github.com/ultraleap/OpenXRHandTracking) | 47 | — | ARCH | OpenXR layer for XR_EXT_hand_tracking |
| [ultraleap/Unity-Hand-Interaction-Experiments](https://github.com/ultraleap/Unity-Hand-Interaction-Experiments) | 82 | — | 2024 | Hand interaction experiments |
| [ultraleap/XR-Keyboard](https://github.com/ultraleap/XR-Keyboard) | 56 | C# | 2023 | Reusable physical XR keyboard |

## 7. Low level, drivers and Linux
| Repo | ⭐ | Language | Push | Note |
|---|---|---|---|---|
| [leapmotion/leapuvc](https://github.com/leapmotion/leapuvc) | 126 | Python/C | 2024-04 | IR images via UVC (see above) |
| [elinalijouvni/OpenLeap](https://github.com/elinalijouvni/OpenLeap) | 142 | C | 2013 | Attempt at an open-source driver (USB reverse engineering) |
| [bugzy/leap-fedora-rpm](https://github.com/bugzy/leap-fedora-rpm) | 20 | Makefile | 2015 | RPM package for Fedora |

## 8. Robotics and IoT (integration ideas)
- [pearsonkyle/pyduino_leapmotion](https://github.com/pearsonkyle/pyduino_leapmotion) — ⭐13 — robotic arm with Python + Arduino.
- [Michdo93/leap_control](https://github.com/Michdo93/leap_control) — ⭐0 · 2026-02 — ROS2 node publishing `cmd_vel` from Ultraleap.
- [hybridgroup/cylon-leapmotion](https://github.com/hybridgroup/cylon-leapmotion) — ⭐36 — adaptor for the Cylon.js robotics framework.
- [xseignard/leapLamp](https://github.com/xseignard/leapLamp) — ⭐110 — lamp controlled with Leap + Arduino + Node.
