import { useTranslation } from "react-i18next";
import { Card, Page, Row, WaitingPage } from "@/components/layout";
import { StatusBadge } from "@/components/status";
import { usePanel } from "@/stores/panel";
import { LinkNotice } from "./LinkNotice";

export function StatusView() {
  const { t } = useTranslation();
  const state = usePanel((s) => s.state);
  if (!state) return <WaitingPage title={t("views.status")} />;
  const { link, devices, stats } = state;
  const connected = link.state === "connected";
  return (
    <Page title={t("views.status")}>
      <LinkNotice />
      <Card title={t("status.trackingService")}>
        <Row
          label={t("status.connection")}
          value={
            <StatusBadge tone={connected ? "good" : "warn"}>
              {t(`link.states.${link.state}`)}
            </StatusBadge>
          }
        />
        <Row label={t("status.serviceVersion")} value={link.serviceVersion ?? "—"} />
        <Row label={t("status.clientVersion")} value={link.clientVersion ?? "—"} />
        <Row label={t("status.connectedDevices")} value={devices.length} />
      </Card>
      <Card title={t("status.dataFlow")}>
        <Row
          label={t("status.trackingRate")}
          value={t("common.fps", { value: stats.trackingFps.toFixed(1) })}
        />
        <Row
          label={t("status.imageRate")}
          value={t("common.fps", { value: stats.imageFps.toFixed(1) })}
        />
        <Row label={t("status.framesReceived")} value={stats.framesReceived} />
      </Card>
    </Page>
  );
}
