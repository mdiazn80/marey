import {
  Activity,
  Camera,
  Cpu,
  Hand,
  Settings as SettingsIcon,
  ScrollText,
  Stethoscope,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useState } from "react";
import lockup from "@/assets/brand/marey-lockup-on-dark.svg";
import { CamerasView } from "@/features/cameras/CamerasView";
import { DevicesView } from "@/features/devices/DevicesView";
import { DiagnosticsView } from "@/features/diagnostics/DiagnosticsView";
import { LogsView } from "@/features/diagnostics/LogsView";
import { SettingsView } from "@/features/settings/SettingsView";
import { StatusView } from "@/features/status/StatusView";
import { VisualizerView } from "@/features/visualizer/VisualizerView";
import { cn } from "@/lib/utils";
import { connectBackend, usePanel, type View } from "@/stores/panel";
import { Notice } from "./ui";

const NAVIGATION: { view: View; label: string; icon: LucideIcon }[] = [
  { view: "status", label: "Status", icon: Activity },
  { view: "devices", label: "Devices", icon: Cpu },
  { view: "visualizer", label: "Visualizer", icon: Hand },
  { view: "cameras", label: "Cameras", icon: Camera },
  { view: "settings", label: "Settings", icon: SettingsIcon },
  { view: "logs", label: "Logs", icon: ScrollText },
  { view: "diagnostics", label: "Diagnostics", icon: Stethoscope },
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
        className="flex w-48 shrink-0 flex-col gap-0.5 border-r border-sidebar-border bg-sidebar p-3"
      >
        <img src={lockup} alt="Marey" className="mb-5 h-8 w-fit px-2 pt-1" />
        {NAVIGATION.map((item) => (
          <button
            key={item.view}
            type="button"
            aria-current={view === item.view ? "page" : undefined}
            onClick={() => setView(item.view)}
            className={cn(
              "flex items-center gap-2.5 rounded-md px-2 py-1.5 text-left transition-colors",
              view === item.view
                ? "bg-sidebar-accent text-sidebar-accent-foreground"
                : "text-muted-foreground hover:bg-sidebar-accent/60 hover:text-sidebar-foreground",
            )}
          >
            <item.icon className={cn("size-4", view === item.view && "text-primary")} />
            {item.label}
          </button>
        ))}
      </nav>
      <main className="flex min-w-0 flex-1 flex-col">
        {mock && (
          <div className="bg-primary/15 px-4 py-1.5 text-center text-xs font-medium text-primary">
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
