// Latest stream data, kept outside React so high-rate updates never cause a re-render.
// Consumers read it from their own animation loop.
import type { Hand } from "@/types/generated/Hand";
import type { TrackingFrame } from "@/types/generated/TrackingFrame";

/** A stereo image message: a 16-byte little-endian header, then the left and right planes. */
export interface StereoImages {
  width: number;
  height: number;
  bytesPerPixel: number;
  frameId: number;
  left: Uint8Array;
  right: Uint8Array;
}

const IMAGE_HEADER_LENGTH = 16;
/** A stream counts as stopped after this long without data, in milliseconds. */
const FRAME_STALE_AFTER_MS = 500;
const IMAGE_STALE_AFTER_MS = 1000;

export function decodeImages(buffer: ArrayBuffer): StereoImages | null {
  if (buffer.byteLength < IMAGE_HEADER_LENGTH) return null;
  const header = new DataView(buffer, 0, IMAGE_HEADER_LENGTH);
  const width = header.getUint32(0, true);
  const height = header.getUint32(4, true);
  const bytesPerPixel = header.getUint32(8, true);
  const frameId = header.getUint32(12, true);
  const planeLength = width * height * bytesPerPixel;
  if (buffer.byteLength < IMAGE_HEADER_LENGTH + planeLength * 2) return null;
  return {
    width,
    height,
    bytesPerPixel,
    frameId,
    left: new Uint8Array(buffer, IMAGE_HEADER_LENGTH, planeLength),
    right: new Uint8Array(buffer, IMAGE_HEADER_LENGTH + planeLength, planeLength),
  };
}

const streams = {
  frame: null as TrackingFrame | null,
  /** `performance.now()` when `frame` arrived. */
  frameTime: 0,
  images: null as StereoImages | null,
  imageTime: 0,
};

/** The hands in the latest frame, or none once the frame stream has stopped. */
export function currentHands(): Hand[] {
  const fresh = performance.now() - streams.frameTime < FRAME_STALE_AFTER_MS;
  return fresh ? (streams.frame?.hands ?? []) : [];
}

/** The latest images, or null once the image stream has stopped. */
export function currentImages(): StereoImages | null {
  const fresh = performance.now() - streams.imageTime < IMAGE_STALE_AFTER_MS;
  return fresh ? streams.images : null;
}

export function receiveFrame(frame: TrackingFrame) {
  streams.frame = frame;
  streams.frameTime = performance.now();
}

export function receiveImages(buffer: ArrayBuffer) {
  const images = decodeImages(buffer);
  if (images) {
    streams.images = images;
    streams.imageTime = performance.now();
  }
}
