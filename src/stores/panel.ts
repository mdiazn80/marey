import { create } from "zustand";
import * as tauri from "@/lib/tauri";
import type { LogEntry } from "@/types/generated/LogEntry";
import type { PanelState } from "@/types/generated/PanelState";
import type { Settings } from "@/types/generated/Settings";
import { receiveFrame, receiveImages } from "./streams";

export type View =
  "visualizer" | "status" | "devices" | "cameras" | "settings" | "logs" | "diagnostics";

const MAX_LOGS = 2000;

interface PanelStore {
  view: View;
  state: PanelState | null;
  logs: LogEntry[];
  /** The latest failed user action, shown until the next attempt. */
  actionError: string | null;
  setView: (view: View) => void;
  selectDevice: (id: number) => Promise<void>;
  changeSettings: (changes: Partial<Settings>) => Promise<void>;
  reportError: (message: string) => void;
  clearLogs: () => void;
}

export const usePanel = create<PanelStore>((set, get) => {
  /** Runs a backend action, replacing the previous error with the outcome of this one. */
  const attempt = async (action: () => Promise<void>) => {
    set({ actionError: null });
    try {
      await action();
    } catch (error) {
      set({ actionError: String(error) });
    }
  };

  return {
    view: "visualizer",
    state: null,
    logs: [],
    actionError: null,
    setView: (view) => set({ view }),
    selectDevice: (id) => attempt(() => tauri.selectDevice(id)),
    changeSettings: async (changes) => {
      const current = get().state?.settings;
      if (current) await attempt(() => tauri.updateSettings({ ...current, ...changes }));
    },
    reportError: (message) => set({ actionError: message }),
    clearLogs: () => set({ logs: [] }),
  };
});

let connecting: Promise<void> | null = null;

/** Connects the store to the backend, once per page load. */
export function connectBackend(): Promise<void> {
  connecting ??= (async () => {
    await tauri.onState((state) => usePanel.setState({ state }));
    await tauri.onLog((entry) =>
      usePanel.setState(({ logs }) => ({ logs: [...logs.slice(-(MAX_LOGS - 1)), entry] })),
    );
    await tauri.subscribeStreams(receiveFrame, receiveImages);
    const [state, logs] = await Promise.all([tauri.getState(), tauri.getLogs()]);
    usePanel.setState({ state, logs });
  })();
  return connecting;
}
