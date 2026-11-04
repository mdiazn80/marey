import { useEffect, useState, type RefObject } from "react";
import { currentHands } from "@/stores/streams";
import type { Hand } from "@/types/generated/Hand";

/** How often the text overlay refreshes. The 3D scene itself runs at the display rate. */
const REFRESH_INTERVAL_MS = 200;
const RENDER_FPS_WINDOW_MS = 1000;

/** Samples the data shown over the scene at a rate that is comfortable for React. */
export function useOverlay(renderCounterRef: RefObject<number>) {
  const [overlay, setOverlay] = useState<{ hands: Hand[]; renderFps: number }>({
    hands: [],
    renderFps: 0,
  });

  useEffect(() => {
    let lastTime = performance.now();
    let lastCount = 0;
    let renderFps = 0;
    const timer = setInterval(() => {
      const now = performance.now();
      if (now - lastTime >= RENDER_FPS_WINDOW_MS) {
        renderFps = ((renderCounterRef.current - lastCount) * 1000) / (now - lastTime);
        lastCount = renderCounterRef.current;
        lastTime = now;
      }
      setOverlay({ hands: currentHands(), renderFps });
    }, REFRESH_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [renderCounterRef]);

  return overlay;
}
