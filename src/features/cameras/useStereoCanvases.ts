import { useEffect, useRef, useState } from "react";
import { currentImages, type StereoImages } from "@/stores/streams";
import { drawPlane, type PlaneBuffer } from "./drawPlane";

export type StreamStatus = "waiting" | "live" | "unsupported";

/** Draws the latest images on two canvases from an animation loop, while `enabled`. */
export function useStereoCanvases(enabled: boolean) {
  const left = useRef<HTMLCanvasElement>(null);
  const right = useRef<HTMLCanvasElement>(null);
  const [status, setStatus] = useState<StreamStatus>("waiting");

  useEffect(() => {
    if (!enabled) return;
    let animation = 0;
    let drawn: StereoImages | null = null;
    const leftBuffer: PlaneBuffer = { image: null };
    const rightBuffer: PlaneBuffer = { image: null };
    const draw = () => {
      animation = requestAnimationFrame(draw);
      const images = currentImages();
      if (!images) {
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
      drawPlane(left.current, images.left, images.width, images.height, leftBuffer);
      drawPlane(right.current, images.right, images.width, images.height, rightBuffer);
    };
    animation = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(animation);
  }, [enabled]);

  return { left, right, status };
}
