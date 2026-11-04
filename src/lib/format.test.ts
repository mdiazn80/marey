import { describe, expect, it } from "vitest";
import { hex } from "./format";

describe("hex", () => {
  it("pads to the requested digits", () => {
    expect(hex(3, 4)).toBe("0x0003");
    expect(hex(0xe8010000, 8)).toBe("0xe8010000");
  });

  it("does not pad by default", () => {
    expect(hex(0xa)).toBe("0xa");
  });
});
