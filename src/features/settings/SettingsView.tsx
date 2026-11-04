import type { ChangeEvent, ReactNode } from "react";
import { Card, Page } from "../../app/ui";
import { usePanel } from "../../stores/panel";
import type { TrackingMode } from "../../types/generated/TrackingMode";

const MODES: { value: TrackingMode; label: string }[] = [
  { value: "desktop", label: "Desktop" },
  { value: "hmd", label: "Head-mounted" },
  { value: "screenTop", label: "Screentop" },
];

function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <label className="flex items-center justify-between gap-6 border-b border-white/5 py-2.5 last:border-0">
      <span>
        <span className="block">{label}</span>
        {hint && <span className="block text-xs text-white/45">{hint}</span>}
      </span>
      {children}
    </label>
  );
}

const CONTROL = "rounded border border-white/15 bg-[#14171c] px-2 py-1 text-white";

export function SettingsView() {
  const settings = usePanel((s) => s.state?.settings);
  const reported = usePanel((s) => s.state?.trackingMode);
  const change = usePanel((s) => s.changeSettings);
  const policies = usePanel((s) => s.state?.policies);
  const hasDevice = usePanel((s) => (s.state?.devices.length ?? 0) > 0);
  if (!settings) return <Page title="Settings">Waiting for the backend…</Page>;

  /** Says so when the service does not report the option as requested. */
  const reportedHint = (base: string, requested: boolean, actual: boolean | undefined) =>
    hasDevice && actual !== undefined && actual !== requested
      ? `${base}. Requested ${requested ? "on" : "off"}, but the service reports ${actual ? "on" : "off"}`
      : base;

  const rate = (key: "frameRateHz" | "imageRateHz") => (event: ChangeEvent<HTMLInputElement>) => {
    const value = Number(event.target.value);
    if (Number.isFinite(value) && value >= 1) void change({ [key]: Math.round(value) });
  };

  return (
    <Page title="Settings">
      <Card title="Tracking">
        <Field
          label="Tracking mode"
          hint={reported && reported !== "unknown" ? `Device reports: ${reported}` : undefined}
        >
          <select
            className={CONTROL}
            value={settings.trackingMode}
            onChange={(e) => void change({ trackingMode: e.target.value as TrackingMode })}
          >
            {MODES.map((mode) => (
              <option key={mode.value} value={mode.value}>
                {mode.label}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Pause tracking">
          <input
            type="checkbox"
            checked={settings.paused}
            onChange={(e) => void change({ paused: e.target.checked })}
          />
        </Field>
        <Field
          label="Allow images"
          hint={reportedHint(
            "Lets the Cameras view receive infrared images",
            settings.imagesEnabled,
            policies?.images,
          )}
        >
          <input
            type="checkbox"
            checked={settings.imagesEnabled}
            onChange={(e) => void change({ imagesEnabled: e.target.checked })}
          />
        </Field>
        <Field
          label="Background frames"
          hint={reportedHint(
            "Keep receiving data while this window is not focused",
            settings.backgroundFrames,
            policies?.backgroundFrames,
          )}
        >
          <input
            type="checkbox"
            checked={settings.backgroundFrames}
            onChange={(e) => void change({ backgroundFrames: e.target.checked })}
          />
        </Field>
      </Card>
      <Card title="Display rates">
        <Field label="Tracking frames to the UI" hint="1 to 240 Hz">
          <input
            type="number"
            min={1}
            max={240}
            className={`${CONTROL} w-20`}
            value={settings.frameRateHz}
            onChange={rate("frameRateHz")}
          />
        </Field>
        <Field label="Camera images to the UI" hint="1 to 120 Hz">
          <input
            type="number"
            min={1}
            max={120}
            className={`${CONTROL} w-20`}
            value={settings.imageRateHz}
            onChange={rate("imageRateHz")}
          />
        </Field>
      </Card>
      <p className="text-xs text-white/45">
        Saved automatically and applied again on every start and device connection.
      </p>
    </Page>
  );
}
