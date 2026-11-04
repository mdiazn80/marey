import { useEffect, useState } from "react";
import { CamerasView } from "../features/cameras/CamerasView";
import { DevicesView } from "../features/devices/DevicesView";
import { DiagnosticsView } from "../features/diagnostics/DiagnosticsView";
import { LogsView } from "../features/diagnostics/LogsView";
import { SettingsView } from "../features/settings/SettingsView";
import { StatusView } from "../features/status/StatusView";
import { VisualizerView } from "../features/visualizer/VisualizerView";
import { connectBackend, usePanel, type View } from "../stores/panel";
import { Notice } from "./ui";

const NAVIGATION: { view: View; label: string }[] = [
  { view: "status", label: "Status" },
  { view: "devices", label: "Devices" },
  { view: "visualizer", label: "Visualizer" },
  { view: "cameras", label: "Cameras" },
  { view: "settings", label: "Settings" },
  { view: "logs", label: "Logs" },
  { view: "diagnostics", label: "Diagnostics" },
];

function Content({ view }: { view: View }) {
  switch (view) {
    case "status":
      return <StatusView />;
    case "devices":
      return <DevicesView />;
    case "visualizer":
      return <VisualizerView />;
    case "cameras":
      return <CamerasView />;
    case "settings":
      return <SettingsView />;
    case "logs":
      return <LogsView />;
    case "diagnostics":
      return <DiagnosticsView />;
  }
}

export function App() {
  const view = usePanel((s) => s.view);
  const setView = usePanel((s) => s.setView);
  const mock = usePanel((s) => s.state?.link.mock ?? false);
  const actionError = usePanel((s) => s.actionError);
  const [startupError, setStartupError] = useState<string | null>(null);

  useEffect(() => {
    connectBackend().catch((error) => setStartupError(String(error)));
  }, []);

  return (
    <div className="flex h-full">
      <nav
        aria-label="Views"
        className="flex w-44 shrink-0 flex-col gap-0.5 border-r border-white/10 p-3"
      >
        <p className="mb-4 px-2 pt-1 text-sm font-semibold tracking-wide text-white">Marey</p>
        {NAVIGATION.map((item) => (
          <button
            key={item.view}
            type="button"
            aria-current={view === item.view ? "page" : undefined}
            onClick={() => setView(item.view)}
            className={`rounded px-2 py-1.5 text-left ${
              view === item.view
                ? "bg-white/10 text-white"
                : "text-white/60 hover:bg-white/5 hover:text-white"
            }`}
          >
            {item.label}
          </button>
        ))}
      </nav>
      <main className="flex min-w-0 flex-1 flex-col">
        {mock && (
          <div className="bg-fuchsia-600/25 px-4 py-1.5 text-center text-xs font-medium text-fuchsia-200">
            Simulation mode: all device data shown is synthetic (MAREY_MOCK=1)
          </div>
        )}
        {(startupError || actionError) && (
          <div className="px-6 pt-4">
            <Notice
              tone="bad"
              title={startupError ? "Could not reach the backend" : "That did not work"}
            >
              {startupError ?? actionError}
            </Notice>
          </div>
        )}
        <div className="min-h-0 flex-1">
          <Content view={view} />
        </div>
      </main>
    </div>
  );
}
