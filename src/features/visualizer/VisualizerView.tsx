import { useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { LinkNotice } from "@/features/status/LinkNotice";
import { usePanel } from "@/stores/panel";
import { HandCard } from "./HandCard";
import { Scene } from "./Scene";
import { useOverlay } from "./useOverlay";

export function VisualizerView() {
  const { t } = useTranslation();
  const trackingFps = usePanel((s) => s.state?.stats.trackingFps ?? 0);
  const devices = usePanel((s) => s.state?.devices);
  const activeId = usePanel((s) => s.state?.activeDeviceId);
  const device = devices?.find((d) => d.id === activeId);
  const hFovDeg = device?.hFovDeg ?? 0;
  const vFovDeg = device?.vFovDeg ?? 0;
  const rangeMm = device?.rangeMm ?? 0;
  const volume = useMemo(() => ({ hFovDeg, vFovDeg, rangeMm }), [hFovDeg, vFovDeg, rangeMm]);

  const renderCounterRef = useRef(0);
  const { hands, renderFps } = useOverlay(renderCounterRef);

  return (
    <div className="relative h-full">
      <Scene volume={volume} renderCounterRef={renderCounterRef} />
      <div className="pointer-events-none absolute inset-x-4 top-4">
        <LinkNotice />
      </div>
      <div className="pointer-events-none absolute right-4 bottom-4 rounded border border-border bg-card/85 px-3 py-2 font-mono text-xs">
        <div>{t("visualizer.tracking", { fps: trackingFps.toFixed(0) })}</div>
        <div>{t("visualizer.render", { fps: renderFps.toFixed(0) })}</div>
      </div>
      <div className="pointer-events-none absolute bottom-4 left-4 flex gap-2">
        {hands.length === 0 && (
          <div className="rounded border border-border bg-card/85 px-3 py-2 text-xs text-muted-foreground">
            {t("visualizer.noHands")}
          </div>
        )}
        {hands.map((hand) => (
          <HandCard key={hand.id} hand={hand} />
        ))}
      </div>
    </div>
  );
}
