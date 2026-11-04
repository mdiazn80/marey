/** Keeps one reusable pixel buffer per canvas. */
export interface PlaneBuffer {
  image: ImageData | null;
}

/** Draws one 8-bit camera plane as grayscale. */
export function drawPlane(
  canvas: HTMLCanvasElement,
  plane: Uint8Array,
  width: number,
  height: number,
  buffer: PlaneBuffer,
) {
  const context = canvas.getContext("2d");
  if (!context) return;
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
    buffer.image = null;
  }
  // Reusing one ImageData avoids allocating about 600 KB for each camera on each frame.
  const image = buffer.image ?? context.createImageData(width, height);
  buffer.image = image;
  const pixels = image.data;
  for (let source = 0, target = 0; source < plane.length; source += 1, target += 4) {
    const value = plane[source];
    pixels[target] = value;
    pixels[target + 1] = value;
    pixels[target + 2] = value;
    pixels[target + 3] = 255;
  }
  context.putImageData(image, 0, 0);
  // Extension point: hand joints projected to pixels would be drawn here, with
  // LeapRectilinearToPixel results delivered alongside the image.
}
