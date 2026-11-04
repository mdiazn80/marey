import { useTranslation } from "react-i18next";
import { Card, Page, Row } from "@/components/layout";
import { StatusBadge } from "@/components/status";
import { Button } from "@/components/ui/button";
import { LinkNotice } from "@/features/status/LinkNotice";
import { hex } from "@/lib/format";
import { usePanel } from "@/stores/panel";
import type { DeviceSummary } from "@/types/generated/DeviceSummary";
import { activeFlags } from "./flags";

function DeviceCard({ device, isActive }: { device: DeviceSummary; isActive: boolean }) {
  const { t } = useTranslation();
  const selectDevice = usePanel((s) => s.selectDevice);
  const trackingFps = usePanel((s) => s.state?.stats.trackingFps ?? 0);
  const flags = activeFlags(device.flags);
  return (
    <Card>
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h2 className="font-heading text-sm font-medium">{device.product}</h2>
          {isActive && <StatusBadge tone="info">{t("devices.active")}</StatusBadge>}
        </div>
        {!isActive && (
          <Button variant="outline" size="sm" onClick={() => void selectDevice(device.id)}>
            {t("devices.makeActive")}
          </Button>
        )}
      </div>
      <Row label={t("devices.serial")} value={device.serial || "—"} />
      <Row
        label={t("devices.firmware")}
        value={device.firmware ?? t("devices.firmwareNotExposed")}
      />
      <Row label={t("devices.productId")} value={hex(device.pid, 4)} />
      <Row label={t("devices.baseline")} value={`${device.baselineMm.toFixed(1)} mm`} />
      <Row
        label={t("devices.fieldOfView")}
        value={
          device.hFovDeg > 0
            ? `${device.hFovDeg.toFixed(0)}° × ${device.vFovDeg.toFixed(0)}°`
            : t("common.notReported")
        }
      />
      <Row
        label={t("devices.range")}
        value={device.rangeMm > 0 ? `${device.rangeMm.toFixed(0)} mm` : t("common.notReported")}
      />
      {isActive && (
        <Row
          label={t("devices.trackingRate")}
          value={t("common.fps", { value: trackingFps.toFixed(0) })}
        />
      )}
      <Row label={t("devices.rawStatus")} value={hex(device.rawStatus, 8)} />
      <div className="mt-3 flex flex-wrap gap-1.5">
        {flags.length === 0 && <StatusBadge tone="muted">{t("devices.noFlags")}</StatusBadge>}
        {flags.map((flag) => (
          <StatusBadge key={flag.key} tone={flag.tone}>
            {t(`devices.flags.${flag.key}`)}
          </StatusBadge>
        ))}
      </div>
    </Card>
  );
}

export function DevicesView() {
  const { t } = useTranslation();
  const devices = usePanel((s) => s.state?.devices ?? []);
  const activeId = usePanel((s) => s.state?.activeDeviceId ?? null);
  return (
    <Page title={t("views.devices")}>
      <LinkNotice />
      {devices.map((device) => (
        <DeviceCard key={device.id} device={device} isActive={device.id === activeId} />
      ))}
    </Page>
  );
}
