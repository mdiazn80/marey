import { useEffect, useRef, useState } from "react";
import { Button, Notice, Page } from "../../app/ui";
import { usePanel } from "../../stores/panel";
import { streams, type StereoImages } from "../../stores/streams";
import { LinkNotice } from "../status/LinkNotice";

/** Treat the image stream as stopped after this long, in milliseconds. */
const STALE_AFTER_MS = 1000;

function drawPlane(
  canvas: HTMLCanvasElement,
  plane: Uint8Array,
  width: number,
  height: number,
  store: { image: ImageData | null },
) {
  const context = canvas.getContext("2d");
  if (!context) return;
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
    store.image = null;
  }
  // Reusing one ImageData avoids allocating about 600 KB for each camera on each frame.
  const image = store.image ?? context.createImageData(width, height);
  store.image = image;
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

type StreamStatus = "waiting" | "live" | "unsupported";

export function CamerasView() {
  const imagesEnabled = usePanel((s) => s.state?.settings.imagesEnabled ?? false);
  const changeSettings = usePanel((s) => s.changeSettings);
  const left = useRef<HTMLCanvasElement>(null);
  const right = useRef<HTMLCanvasElement>(null);
  const [status, setStatus] = useState<StreamStatus>("waiting");

  useEffect(() => {
    if (!imagesEnabled) return;
    let animation = 0;
    let drawn: StereoImages | null = null;
    const leftStore = { image: null as ImageData | null };
    const rightStore = { image: null as ImageData | null };
    const draw = () => {
      animation = requestAnimationFrame(draw);
      const images = streams.images;
      const live = images !== null && performance.now() - streams.imageTime < STALE_AFTER_MS;
      if (!live) {
        setStatus("waiting");
        return;
      }
      if (images.bytesPerPixel !== 1) {
        setStatus("unsupported");
        return;
      }
      setStatus("live");
      if (images === drawn || !left.current || !right.current) return;
      drawn = images;
      drawPlane(left.current, images.left, images.width, images.height, leftStore);
      drawPlane(right.current, images.right, images.width, images.height, rightStore);
    };
    animation = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(animation);
  }, [imagesEnabled]);

  return (
    <Page
      title="Cameras"
      actions={
        <Button onClick={() => void changeSettings({ imagesEnabled: !imagesEnabled })}>
          {imagesEnabled ? "Stop images" : "Allow images"}
        </Button>
      }
    >
      <LinkNotice />
      {!imagesEnabled && (
        <Notice tone="muted" title="Images are off">
          Allow images to see the infrared view of each camera.
        </Notice>
      )}
      {imagesEnabled && status === "waiting" && (
        <Notice tone="warn" title="Waiting for images from the device…" />
      )}
      {imagesEnabled && status === "unsupported" && (
        <Notice tone="warn" title="This device's image format is not supported yet" />
      )}
      {imagesEnabled && (
        <div className="grid grid-cols-2 gap-3">
          {[
            { label: "Left camera", ref: left },
            { label: "Right camera", ref: right },
          ].map((camera) => (
            <figure key={camera.label}>
              <canvas
                ref={camera.ref}
                aria-label={camera.label}
                className="w-full rounded border border-white/10 bg-black"
                style={{ imageRendering: "pixelated" }}
              />
              <figcaption className="mt-1 text-xs text-white/50">{camera.label}</figcaption>
            </figure>
          ))}
        </div>
      )}
    </Page>
  );
}
