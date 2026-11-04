import { describe, expect, it } from "vitest";
import type { DeviceFlags } from "@/types/generated/DeviceFlags";
import { activeFlags } from "./flags";

const NO_FLAGS: DeviceFlags = {
  streaming: false,
  paused: false,
  robust: false,
  smudged: false,
  lowResource: false,
  unknownFailure: false,
  badCalibration: false,
  badFirmware: false,
  badTransport: false,
  badControl: false,
};

describe("activeFlags", () => {
  it("is empty when nothing is set", () => {
    expect(activeFlags(NO_FLAGS)).toEqual([]);
  });

  it("lists what is set, with a tone that matches its severity", () => {
    const flags = activeFlags({ ...NO_FLAGS, streaming: true, badFirmware: true });
    expect(flags.map(({ key, tone }) => [key, tone])).toEqual([
      ["streaming", "good"],
      ["badFirmware", "bad"],
    ]);
  });
});
