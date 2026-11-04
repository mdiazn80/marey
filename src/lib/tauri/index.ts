// Typed wrappers around Tauri. This is the only module allowed to import from @tauri-apps.
import { getVersion } from "@tauri-apps/api/app";
import { Channel, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { LogEntry } from "../../types/generated/LogEntry";
import type { PanelState } from "../../types/generated/PanelState";
import type { Settings } from "../../types/generated/Settings";
import type { TrackingFrame } from "../../types/generated/TrackingFrame";

const STATE_EVENT = "marey://state";
const LOG_EVENT = "marey://log";

export const getAppVersion = () => getVersion();
export const openExternalUrl = (url: string) => openUrl(url);
export const getState = () => invoke<PanelState>("get_state");
export const getLogs = () => invoke<LogEntry[]>("get_logs");
export const selectDevice = (id: number) => invoke<void>("select_device", { id });
export const updateSettings = (settings: Settings) => invoke<void>("update_settings", { settings });
export const copyText = (text: string) => invoke<void>("copy_text", { text });
export const exportText = (fileName: string, contents: string) =>
  invoke<string | null>("export_text", { fileName, contents });

export const onState = (handler: (state: PanelState) => void): Promise<UnlistenFn> =>
  listen<PanelState>(STATE_EVENT, (event) => handler(event.payload));

export const onLog = (handler: (entry: LogEntry) => void): Promise<UnlistenFn> =>
  listen<LogEntry>(LOG_EVENT, (event) => handler(event.payload));

/** Starts the high-rate streams. A later call replaces the previous subscription. */
export function subscribeStreams(
  onFrame: (frame: TrackingFrame) => void,
  onImages: (message: ArrayBuffer) => void,
): Promise<void> {
  const frames = new Channel<TrackingFrame>(onFrame);
  const images = new Channel<ArrayBuffer>(onImages);
  return invoke("subscribe_streams", { frames, images });
}
