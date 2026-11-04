import { useTranslation } from "react-i18next";
import { Notice } from "@/components/status";
import { usePanel } from "@/stores/panel";

/** Explains the current connection problem, or a device that is listed but sends no frames. */
export function LinkNotice() {
  const { t } = useTranslation();
  const state = usePanel((s) => s.state);
  if (!state) return null;
  const { link, devices, activeDeviceId, settings, stats } = state;

  switch (link.state) {
    case "starting":
      return <Notice tone="muted" title={t("link.starting")} />;
    case "sdkMissing":
      return (
        <Notice tone="bad" title={t("link.sdkMissingTitle")}>
          <p>{t("link.sdkMissingBody")}</p>
          {link.detail && <p className="selectable mt-1 font-mono text-xs">{link.detail}</p>}
        </Notice>
      );
    case "serviceNotRunning":
      return (
        <Notice tone="bad" title={t("link.serviceNotRunningTitle")}>
          {t("link.serviceNotRunningBody")}
        </Notice>
      );
    case "connecting":
      return (
        <Notice tone="warn" title={t("link.connectingTitle")}>
          {t("link.connectingBody")}
        </Notice>
      );
    case "connected":
      break;
  }

  if (devices.length === 0) {
    return (
      <Notice tone="warn" title={t("link.noDeviceTitle")}>
        {t("link.noDeviceBody")}
      </Notice>
    );
  }
  const active = devices.find((d) => d.id === activeDeviceId);
  // The rate, not the status flags: an original controller reports status 0 even while streaming.
  if (active && !settings.paused && stats.trackingFps === 0) {
    return (
      <Notice tone="warn" title={t("link.noFramesTitle")}>
        {t("link.noFramesBody", { product: active.product })}
      </Notice>
    );
  }
  return null;
}
