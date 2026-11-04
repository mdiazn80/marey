import { Card, Page, Row, StatusBadge } from "../../app/ui";
import { usePanel } from "../../stores/panel";
import type { LinkState } from "../../types/generated/LinkState";
import { LinkNotice } from "./LinkNotice";

const LINK_LABELS: Record<LinkState, string> = {
  starting: "Starting",
  sdkMissing: "Software not found",
  serviceNotRunning: "Service not running",
  connecting: "Connecting",
  connected: "Connected",
};

export function StatusView() {
  const state = usePanel((s) => s.state);
  if (!state) return <Page title="Status">Waiting for the backend…</Page>;
  const { link, devices, stats } = state;
  const connected = link.state === "connected";
  return (
    <Page title="Status">
      <LinkNotice />
      <Card title="Tracking service">
        <Row
          label="Connection"
          value={
            <StatusBadge tone={connected ? "good" : "warn"}>{LINK_LABELS[link.state]}</StatusBadge>
          }
        />
        <Row label="Service version" value={link.serviceVersion ?? "—"} />
        <Row label="LeapC client version" value={link.clientVersion ?? "—"} />
        <Row label="Connected devices" value={devices.length} />
      </Card>
      <Card title="Data flow">
        <Row label="Tracking rate" value={`${stats.trackingFps.toFixed(1)} fps`} />
        <Row label="Image rate" value={`${stats.imageFps.toFixed(1)} fps`} />
        <Row label="Frames received" value={stats.framesReceived} />
      </Card>
    </Page>
  );
}
