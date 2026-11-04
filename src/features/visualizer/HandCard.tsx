import { useTranslation } from "react-i18next";
import type { Hand } from "@/types/generated/Hand";

export function HandCard({ hand }: { hand: Hand }) {
  const { t } = useTranslation();
  const rows: [string, string][] = [
    [t("visualizer.confidence"), `${(hand.confidence * 100).toFixed(0)}%`],
    [t("visualizer.grab"), hand.grabStrength.toFixed(2)],
    [t("visualizer.pinch"), hand.pinchStrength.toFixed(2)],
    [t("visualizer.pinchDistance"), `${hand.pinchDistance.toFixed(0)} mm`],
    [t("visualizer.visible"), `${(hand.visibleTimeUs / 1_000_000).toFixed(1)} s`],
  ];
  return (
    <div className="w-44 rounded border border-border bg-card/85 p-2.5 text-xs">
      <p
        className={`mb-1 font-medium ${hand.side === "left" ? "text-foreground" : "text-primary"}`}
      >
        {hand.side === "left" ? t("visualizer.leftHand") : t("visualizer.rightHand")}
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
