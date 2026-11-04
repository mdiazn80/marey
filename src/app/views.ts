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
import type { ComponentType } from "react";
import { CamerasView } from "@/features/cameras/CamerasView";
import { DevicesView } from "@/features/devices/DevicesView";
import { DiagnosticsView } from "@/features/diagnostics/DiagnosticsView";
import { LogsView } from "@/features/diagnostics/LogsView";
import { SettingsView } from "@/features/settings/SettingsView";
import { StatusView } from "@/features/status/StatusView";
import { VisualizerView } from "@/features/visualizer/VisualizerView";
import type { View } from "@/stores/panel";

/** Every view of the app, in the order the sidebar lists them. Their names are in `views.*`. */
export const VIEWS: Record<View, { icon: LucideIcon; Component: ComponentType }> = {
  visualizer: { icon: Hand, Component: VisualizerView },
  status: { icon: Activity, Component: StatusView },
  devices: { icon: Cpu, Component: DevicesView },
  cameras: { icon: Camera, Component: CamerasView },
  settings: { icon: SettingsIcon, Component: SettingsView },
  logs: { icon: ScrollText, Component: LogsView },
  diagnostics: { icon: Stethoscope, Component: DiagnosticsView },
};
