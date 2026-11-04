import { create } from "zustand";
import * as tauri from "../lib/tauri";
import type { LogEntry } from "../types/generated/LogEntry";
import type { PanelState } from "../types/generated/PanelState";
import type { Settings } from "../types/generated/Settings";
import { receiveFrame, receiveImages } from "./streams";

export type View =
  "status" | "devices" | "visualizer" | "cameras" | "settings" | "logs" | "diagnostics";

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
  clearLogs: () => void;
}

const message = (error: unknown) => (typeof error === "string" ? error : String(error));

export const usePanel = create<PanelStore>((set, get) => ({
  view: "status",
  state: null,
  logs: [],
  actionError: null,
  setView: (view) => set({ view }),
  selectDevice: async (id) => {
    set({ actionError: null });
    try {
      await tauri.selectDevice(id);
    } catch (error) {
      set({ actionError: message(error) });
    }
  },
  changeSettings: async (changes) => {
    const current = get().state?.settings;
    if (!current) return;
    set({ actionError: null });
    try {
      await tauri.updateSettings({ ...current, ...changes });
    } catch (error) {
      set({ actionError: message(error) });
    }
  },
  clearLogs: () => set({ logs: [] }),
}));

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
