import { describe, expect, it } from "vitest";
import { decodeImages } from "./streams";

/** The same message as the `image_message_layout_is_stable` test in `streaming/mod.rs`. */
function message(): ArrayBuffer {
  const bytes = new Uint8Array(24);
  const header = new DataView(bytes.buffer);
  header.setUint32(0, 2, true);
  header.setUint32(4, 2, true);
  header.setUint32(8, 1, true);
  header.setUint32(12, 2, true);
  bytes.set([1, 2, 3, 4, 5, 6, 7, 8], 16);
  return bytes.buffer;
}

describe("decodeImages", () => {
  it("reads the header and both planes", () => {
    const images = decodeImages(message());
    expect(images).toMatchObject({ width: 2, height: 2, bytesPerPixel: 1, frameId: 2 });
    expect(Array.from(images?.left ?? [])).toEqual([1, 2, 3, 4]);
    expect(Array.from(images?.right ?? [])).toEqual([5, 6, 7, 8]);
  });

  it("rejects a message shorter than its header", () => {
    expect(decodeImages(new ArrayBuffer(15))).toBeNull();
  });

  it("rejects a message shorter than its planes", () => {
    expect(decodeImages(message().slice(0, 23))).toBeNull();
  });
});
