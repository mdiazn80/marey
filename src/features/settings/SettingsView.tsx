import type { ChangeEvent, ReactNode } from "react";
import { Card, Page } from "../../app/ui";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { usePanel } from "../../stores/panel";
import type { TrackingMode } from "../../types/generated/TrackingMode";

const MODES: { value: TrackingMode; label: string }[] = [
  { value: "desktop", label: "Desktop" },
  { value: "hmd", label: "Head-mounted" },
  { value: "screenTop", label: "Screentop" },
];

function Field({
  id,
  label,
  hint,
  children,
}: {
  id: string;
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-6 border-b border-border py-2.5 last:border-0">
      <div className="space-y-0.5">
        <Label htmlFor={id}>{label}</Label>
        {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
      </div>
      {children}
    </div>
  );
}

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
          id="tracking-mode"
          label="Tracking mode"
          hint={reported && reported !== "unknown" ? `Device reports: ${reported}` : undefined}
        >
          <Select
            value={settings.trackingMode}
            onValueChange={(value) => void change({ trackingMode: value as TrackingMode })}
          >
            <SelectTrigger id="tracking-mode" className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {MODES.map((mode) => (
                <SelectItem key={mode.value} value={mode.value}>
                  {mode.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field
          id="paused"
          label="Pause tracking"
          hint="Pauses the service for every app that uses it"
        >
          <Switch
            id="paused"
            checked={settings.paused}
            onCheckedChange={(checked) => void change({ paused: checked })}
          />
        </Field>
        <Field
          id="imagesEnabled"
          label="Allow images"
          hint={reportedHint(
            "Lets the Cameras view receive infrared images",
            settings.imagesEnabled,
            policies?.images,
          )}
        >
          <Switch
            id="imagesEnabled"
            checked={settings.imagesEnabled}
            onCheckedChange={(checked) => void change({ imagesEnabled: checked })}
          />
        </Field>
        <Field
          id="backgroundFrames"
          label="Background frames"
          hint={reportedHint(
            "Keep receiving data while this window is not focused",
            settings.backgroundFrames,
            policies?.backgroundFrames,
          )}
        >
          <Switch
            id="backgroundFrames"
            checked={settings.backgroundFrames}
            onCheckedChange={(checked) => void change({ backgroundFrames: checked })}
          />
        </Field>
      </Card>
      <Card title="Display rates">
        <Field id="frameRateHz" label="Tracking frames to the UI" hint="1 to 240 Hz">
          <Input
            id="frameRateHz"
            type="number"
            min={1}
            max={240}
            className="w-20"
            value={settings.frameRateHz}
            onChange={rate("frameRateHz")}
          />
        </Field>
        <Field id="imageRateHz" label="Camera images to the UI" hint="1 to 120 Hz">
          <Input
            id="imageRateHz"
            type="number"
            min={1}
            max={120}
            className="w-20"
            value={settings.imageRateHz}
            onChange={rate("imageRateHz")}
          />
        </Field>
      </Card>
      <p className="text-xs text-muted-foreground">
        Saved automatically and applied again on every start and device connection.
      </p>
    </Page>
  );
}
