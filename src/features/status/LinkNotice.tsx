import { Notice } from "../../app/ui";
import { usePanel } from "../../stores/panel";

/** Explains the current connection problem, or a device that is listed but sends no frames. */
export function LinkNotice() {
  const state = usePanel((s) => s.state);
  if (!state) return null;
  const { link, devices, activeDeviceId, settings, stats } = state;

  switch (link.state) {
    case "starting":
      return <Notice tone="muted" title="Starting…" />;
    case "sdkMissing":
      return (
        <Notice tone="bad" title="Ultraleap tracking software not found">
          <p>
            Install Ultraleap Hand Tracking, or set LEAPSDK_LIB_PATH to the SDK lib directory. Marey
            retries every few seconds. To try the app without it, start it with MAREY_MOCK=1.
          </p>
          {link.detail && <p className="selectable mt-1 font-mono text-xs">{link.detail}</p>}
        </Notice>
      );
    case "serviceNotRunning":
      return (
        <Notice tone="bad" title="The tracking service is not running">
          Start Ultraleap Hand Tracking. Marey reconnects when the service comes back.
        </Notice>
      );
    case "connecting":
      return (
        <Notice tone="warn" title="Connecting to the tracking service…">
          If this does not clear, check that Ultraleap Hand Tracking is running.
        </Notice>
      );
    case "connected":
      break;
  }

  if (devices.length === 0) {
    return (
      <Notice tone="warn" title="No device connected">
        Plug in a Leap Motion or Ultraleap device. It appears here automatically.
      </Notice>
    );
  }
  const active = devices.find((d) => d.id === activeDeviceId);
  // The rate, not the status flags: an original controller reports status 0 even while streaming.
  if (active && !settings.paused && stats.trackingFps === 0) {
    return (
      <Notice tone="warn" title="Device detected, but no frames are arriving">
        The service lists {active.product} but is not sending tracking frames. Unplug the device and
        plug it back in. Another app may also have paused tracking for the whole service.
      </Notice>
    );
  }
  return null;
}
