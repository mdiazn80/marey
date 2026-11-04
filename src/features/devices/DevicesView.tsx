import { Badge, Button, Card, Page, Row } from "../../app/ui";
import { usePanel } from "../../stores/panel";
import { LinkNotice } from "../status/LinkNotice";
import { activeFlags } from "./flags";

export function DevicesView() {
  const devices = usePanel((s) => s.state?.devices ?? []);
  const activeId = usePanel((s) => s.state?.activeDeviceId ?? null);
  const selectDevice = usePanel((s) => s.selectDevice);
  const trackingFps = usePanel((s) => s.state?.stats.trackingFps ?? 0);
  return (
    <Page title="Devices">
      <LinkNotice />
      {devices.map((device) => {
        const flags = activeFlags(device.flags);
        const isActive = device.id === activeId;
        return (
          <Card key={device.id}>
            <div className="mb-3 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-medium text-white">{device.product}</h2>
                {isActive && <Badge tone="info">Active</Badge>}
              </div>
              {!isActive && (
                <Button onClick={() => void selectDevice(device.id)}>Make active</Button>
              )}
            </div>
            <Row label="Serial number" value={device.serial || "—"} />
            <Row label="Firmware" value={device.firmware ?? "Not exposed by LeapC"} />
            <Row label="Product ID" value={`0x${device.pid.toString(16).padStart(4, "0")}`} />
            <Row label="Camera baseline" value={`${device.baselineMm.toFixed(1)} mm`} />
            <Row
              label="Field of view"
              value={
                device.hFovDeg > 0
                  ? `${device.hFovDeg.toFixed(0)}° × ${device.vFovDeg.toFixed(0)}°`
                  : "Not reported"
              }
            />
            <Row
              label="Range"
              value={device.rangeMm > 0 ? `${device.rangeMm.toFixed(0)} mm` : "Not reported"}
            />
            {isActive && <Row label="Tracking rate" value={`${trackingFps.toFixed(0)} fps`} />}
            <Row label="Raw status" value={`0x${device.rawStatus.toString(16).padStart(8, "0")}`} />
            <div className="mt-3 flex flex-wrap gap-1.5">
              {flags.length === 0 && <Badge tone="muted">No status flags set</Badge>}
              {flags.map((flag) => (
                <Badge key={flag.key} tone={flag.tone}>
                  {flag.label}
                </Badge>
              ))}
            </div>
          </Card>
        );
      })}
    </Page>
  );
}
