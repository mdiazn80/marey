import type { Tone } from "@/components/status";
import type { DeviceFlags } from "@/types/generated/DeviceFlags";

/** Their names are in `devices.flags.*`. */
const FLAGS: { key: keyof DeviceFlags; tone: Tone }[] = [
  { key: "streaming", tone: "good" },
  { key: "paused", tone: "info" },
  { key: "robust", tone: "info" },
  { key: "smudged", tone: "warn" },
  { key: "lowResource", tone: "warn" },
  { key: "unknownFailure", tone: "bad" },
  { key: "badCalibration", tone: "bad" },
  { key: "badFirmware", tone: "bad" },
  { key: "badTransport", tone: "bad" },
  { key: "badControl", tone: "bad" },
];

export const activeFlags = (flags: DeviceFlags) => FLAGS.filter((flag) => flags[flag.key]);
