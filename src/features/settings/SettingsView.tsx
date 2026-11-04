import { useTranslation } from "react-i18next";
import { Card, Page, WaitingPage } from "@/components/layout";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { LANGUAGES } from "@/i18n/languages";
import { usePanel } from "@/stores/panel";
import type { TrackingMode } from "@/types/generated/TrackingMode";
import { Field, RateField, SwitchField } from "./fields";

const MODES = ["desktop", "hmd", "screenTop"] as const;

/** The backend limits rates to these values, in `Settings::sanitized`. */
const MAX_FRAME_RATE_HZ = 240;
const MAX_IMAGE_RATE_HZ = 120;

export function SettingsView() {
  const { t, i18n } = useTranslation();
  const settings = usePanel((s) => s.state?.settings);
  const reported = usePanel((s) => s.state?.trackingMode);
  const change = usePanel((s) => s.changeSettings);
  const policies = usePanel((s) => s.state?.policies);
  const hasDevice = usePanel((s) => (s.state?.devices.length ?? 0) > 0);
  if (!settings) return <WaitingPage title={t("views.settings")} />;

  /** Says so when the service does not report the option as requested. */
  const reportedHint = (hint: string, requested: boolean, actual: boolean | undefined) =>
    hasDevice && actual !== undefined && actual !== requested
      ? t("settings.reportedMismatch", {
          hint,
          requested: requested ? t("common.on") : t("common.off"),
          actual: actual ? t("common.on") : t("common.off"),
        })
      : hint;

  return (
    <Page title={t("views.settings")}>
      <Card title={t("settings.general")}>
        <Field id="language" label={t("settings.language")}>
          <Select
            value={i18n.resolvedLanguage}
            onValueChange={(language) => void i18n.changeLanguage(language)}
          >
            <SelectTrigger id="language" className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LANGUAGES.map((language) => (
                <SelectItem key={language.code} value={language.code}>
                  {language.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
      </Card>
      <Card title={t("settings.tracking")}>
        <Field
          id="tracking-mode"
          label={t("settings.trackingMode")}
          hint={
            reported && reported !== "unknown"
              ? t("settings.deviceReports", { mode: t(`settings.modes.${reported}`) })
              : undefined
          }
        >
          <Select
            value={settings.trackingMode}
            onValueChange={(value) => void change({ trackingMode: value as TrackingMode })}
          >
            <SelectTrigger id="tracking-mode" className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {MODES.map((mode) => (
                <SelectItem key={mode} value={mode}>
                  {t(`settings.modes.${mode}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <SwitchField
          id="paused"
          label={t("settings.pause")}
          hint={t("settings.pauseHint")}
          checked={settings.paused}
          onChange={(paused) => void change({ paused })}
        />
        <SwitchField
          id="imagesEnabled"
          label={t("settings.images")}
          hint={reportedHint(t("settings.imagesHint"), settings.imagesEnabled, policies?.images)}
          checked={settings.imagesEnabled}
          onChange={(imagesEnabled) => void change({ imagesEnabled })}
        />
        <SwitchField
          id="backgroundFrames"
          label={t("settings.backgroundFrames")}
          hint={reportedHint(
            t("settings.backgroundFramesHint"),
            settings.backgroundFrames,
            policies?.backgroundFrames,
          )}
          checked={settings.backgroundFrames}
          onChange={(backgroundFrames) => void change({ backgroundFrames })}
        />
      </Card>
      <Card title={t("settings.displayRates")}>
        <RateField
          id="frameRateHz"
          label={t("settings.frameRate")}
          max={MAX_FRAME_RATE_HZ}
          value={settings.frameRateHz}
          onChange={(frameRateHz) => void change({ frameRateHz })}
        />
        <RateField
          id="imageRateHz"
          label={t("settings.imageRate")}
          max={MAX_IMAGE_RATE_HZ}
          value={settings.imageRateHz}
          onChange={(imageRateHz) => void change({ imageRateHz })}
        />
      </Card>
      <p className="text-xs text-muted-foreground">{t("settings.savedNote")}</p>
    </Page>
  );
}
