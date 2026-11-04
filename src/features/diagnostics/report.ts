import type { PanelState } from "../../types/generated/PanelState";
import { activeFlags } from "../devices/flags";

/** Builds the plain-text report meant to be pasted into a bug report. */
export function buildReport(state: PanelState): string {
  const { link, devices, activeDeviceId, policies, settings, stats, trackingMode } = state;
  const lines = [
    "Marey diagnostics",
    `Generated: ${new Date().toISOString()}`,
    `Simulation mode: ${link.mock ? "yes (synthetic data)" : "no"}`,
    "",
    "[Service]",
    `Link: ${link.state}${link.detail ? ` (${link.detail})` : ""}`,
    `Service version: ${link.serviceVersion ?? "unknown"}`,
    `LeapC client version: ${link.clientVersion ?? "unknown"}`,
    "",
    "[Devices]",
  ];
  if (devices.length === 0) lines.push("none");
  for (const device of devices) {
    const flags = activeFlags(device.flags).map((flag) => flag.label);
    lines.push(
      `${device.id === activeDeviceId ? "* " : "  "}${device.product} (id ${device.id})`,
      `  serial: ${device.serial || "unknown"}`,
      `  firmware: ${device.firmware ?? "not exposed by LeapC"}`,
      `  pid: 0x${device.pid.toString(16).padStart(4, "0")}`,
      `  status: 0x${device.rawStatus.toString(16).padStart(8, "0")} [${flags.join(", ") || "no flags"}]`,
      `  baseline: ${device.baselineMm} mm, fov: ${device.hFovDeg.toFixed(1)} x ${device.vFovDeg.toFixed(1)} deg, range: ${device.rangeMm} mm (0 = not reported)`,
    );
  }
  lines.push(
    "",
    "[Policies reported by the service]",
    `raw: 0x${policies.raw.toString(16)}`,
    `images: ${policies.images}`,
    `background frames: ${policies.backgroundFrames}`,
    `allow pause/resume: ${policies.allowPauseResume}`,
    `optimize HMD: ${policies.optimizeHmd}`,
    `optimize screentop: ${policies.optimizeScreenTop}`,
    `map points: ${policies.mapPoints}`,
    `tracking mode: ${trackingMode}`,
    "",
    "[Requested settings]",
    JSON.stringify(settings),
    "",
    "[Frame stats]",
    `tracking: ${stats.trackingFps.toFixed(1)} fps (device reports ${stats.reportedFps.toFixed(1)})`,
    `images: ${stats.imageFps.toFixed(1)} fps`,
    `frames received/sent/dropped: ${stats.framesReceived}/${stats.framesSent}/${stats.framesDropped}`,
    `images received/sent: ${stats.imagesReceived}/${stats.imagesSent}`,
  );
  return lines.join("\n");
}
