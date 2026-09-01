import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CaptureCandidate } from "../lib/types";

export type CaptureReady = { requestId: string; candidate: CaptureCandidate };

export interface WindowsCaptureBackend {
  listenReady(handler: (event: CaptureReady) => void): Promise<() => void>;
  focus(): Promise<void>;
  releaseFocus(): Promise<void>;
  hide(requestId: string): Promise<void>;
}

export const tauriWindowsCaptureBackend: WindowsCaptureBackend = {
  listenReady: (handler) => listen<CaptureReady>("capture-ready", ({ payload }) => handler(payload)),
  focus: () => invoke("focus_capture_window"),
  releaseFocus: () => invoke("release_capture_window_focus"),
  hide: (requestId) => invoke("hide_capture_window", { requestId }),
};
