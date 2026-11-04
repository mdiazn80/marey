import { activeFlags } from "@/features/devices/flags";
import { hex } from "@/lib/format";
import type { PanelState } from "@/types/generated/PanelState";

/**
 * Builds the plain-text report meant to be pasted into a bug report. It is always in English, so
 * maintainers can read it whatever language the app is using.
 */
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
    const flags = activeFlags(device.flags).map((flag) => flag.key);
    lines.push(
      `${device.id === activeDeviceId ? "* " : "  "}${device.product} (id ${device.id})`,
      `  serial: ${device.serial || "unknown"}`,
      `  firmware: ${device.firmware ?? "not exposed by LeapC"}`,
      `  pid: ${hex(device.pid, 4)}`,
      `  status: ${hex(device.rawStatus, 8)} [${flags.join(", ") || "no flags"}]`,
      `  baseline: ${device.baselineMm} mm, fov: ${device.hFovDeg.toFixed(1)} x ${device.vFovDeg.toFixed(1)} deg, range: ${device.rangeMm} mm (0 = not reported)`,
    );
  }
  lines.push(
    "",
    "[Policies reported by the service]",
    `raw: ${hex(policies.raw)}`,
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
