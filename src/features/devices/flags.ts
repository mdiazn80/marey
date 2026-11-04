import type { DeviceFlags } from "../../types/generated/DeviceFlags";

export type FlagTone = "good" | "warn" | "bad" | "info";

const FLAGS: { key: keyof DeviceFlags; label: string; tone: FlagTone }[] = [
  { key: "streaming", label: "Streaming", tone: "good" },
  { key: "paused", label: "Paused", tone: "info" },
  { key: "robust", label: "Robust mode", tone: "info" },
  { key: "smudged", label: "Smudged", tone: "warn" },
  { key: "lowResource", label: "Low resource", tone: "warn" },
  { key: "unknownFailure", label: "Unknown failure", tone: "bad" },
  { key: "badCalibration", label: "Bad calibration", tone: "bad" },
  { key: "badFirmware", label: "Bad firmware", tone: "bad" },
  { key: "badTransport", label: "Bad transport", tone: "bad" },
  { key: "badControl", label: "Bad control", tone: "bad" },
];

export const activeFlags = (flags: DeviceFlags) => FLAGS.filter((flag) => flags[flag.key]);
