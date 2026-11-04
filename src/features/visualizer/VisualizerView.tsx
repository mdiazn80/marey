import { useEffect, useMemo, useRef, useState } from "react";
import { usePanel } from "../../stores/panel";
import { streams } from "../../stores/streams";
import type { Hand } from "../../types/generated/Hand";
import { LinkNotice } from "../status/LinkNotice";
import { Scene } from "./Scene";

/** How often the text overlay refreshes. The 3D scene itself runs at the display rate. */
const OVERLAY_INTERVAL_MS = 200;

function HandCard({ hand }: { hand: Hand }) {
  const rows: [string, string][] = [
    ["Confidence", `${(hand.confidence * 100).toFixed(0)}%`],
    ["Grab", hand.grabStrength.toFixed(2)],
    ["Pinch", hand.pinchStrength.toFixed(2)],
    ["Pinch distance", `${hand.pinchDistance.toFixed(0)} mm`],
    ["Visible", `${(hand.visibleTimeUs / 1_000_000).toFixed(1)} s`],
  ];
  return (
    <div className="w-44 rounded border border-border bg-card/85 p-2.5 text-xs">
      <p
        className={`mb-1 font-medium ${hand.side === "left" ? "text-foreground" : "text-primary"}`}
      >
        {hand.side === "left" ? "Left hand" : "Right hand"}
      </p>
      {rows.map(([label, value]) => (
        <div key={label} className="flex justify-between">
          <span className="text-muted-foreground">{label}</span>
          <span className="font-mono">{value}</span>
        </div>
      ))}
    </div>
  );
}

export function VisualizerView() {
  const stats = usePanel((s) => s.state?.stats);
  const devices = usePanel((s) => s.state?.devices);
  const activeId = usePanel((s) => s.state?.activeDeviceId);
  const device = devices?.find((d) => d.id === activeId);
  const hFovDeg = device?.hFovDeg ?? 0;
  const vFovDeg = device?.vFovDeg ?? 0;
  const rangeMm = device?.rangeMm ?? 0;
  const volume = useMemo(() => ({ hFovDeg, vFovDeg, rangeMm }), [hFovDeg, vFovDeg, rangeMm]);

  const renderCounterRef = useRef(0);
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
      if (now - lastTime >= 1000) {
        renderFps = ((renderCounterRef.current - lastCount) * 1000) / (now - lastTime);
        lastCount = renderCounterRef.current;
        lastTime = now;
      }
      const fresh = now - streams.frameTime < 500;
      setOverlay({ hands: fresh ? (streams.frame?.hands ?? []) : [], renderFps });
    }, OVERLAY_INTERVAL_MS);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="relative h-full">
      <Scene volume={volume} renderCounterRef={renderCounterRef} />
      <div className="pointer-events-none absolute inset-x-4 top-4">
        <LinkNotice />
      </div>
      <div className="pointer-events-none absolute right-4 bottom-4 rounded border border-border bg-card/85 px-3 py-2 font-mono text-xs">
        <div>Tracking {(stats?.trackingFps ?? 0).toFixed(0)} fps</div>
        <div>Render {overlay.renderFps.toFixed(0)} fps</div>
      </div>
      <div className="pointer-events-none absolute bottom-4 left-4 flex gap-2">
        {overlay.hands.length === 0 && (
          <div className="rounded border border-border bg-card/85 px-3 py-2 text-xs text-muted-foreground">
            No hands in view
          </div>
        )}
        {overlay.hands.map((hand) => (
          <HandCard key={hand.id} hand={hand} />
        ))}
      </div>
    </div>
  );
}
