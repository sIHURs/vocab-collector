import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CaptureCandidate, CaptureCard, CaptureFailure, PlatformCapabilities } from "../lib/types";

export type CaptureReady = { requestId: string; candidate: CaptureCandidate };
export type CaptureError = { requestId: string; failure: CaptureFailure };
export type OcrCandidate = { text: string; bounds: { x: number; y: number; width: number; height: number }; confidence: number };
export type OcrCandidatesReady = { requestId: string; candidates: OcrCandidate[]; ambiguous: boolean };

export interface WindowsCaptureBackend {
  listenReady(handler: (event: CaptureReady) => void): Promise<() => void>;
  listenError(handler: (event: CaptureError) => void): Promise<() => void>;
  listenOcrCandidate(handler: (event: OcrCandidatesReady) => void): Promise<() => void>;
  focus(): Promise<void>;
  releaseFocus(): Promise<void>;
  hide(requestId: string): Promise<void>;
  save(requestId: string, correction: { selectedText: string; sentence: string; translation?: string }, withoutTranslation: boolean): Promise<CaptureCard>;
  undo(requestId: string, encounterId: string): Promise<void>;
  startOcr(requestId: string): Promise<void>;
  confirmOcr(requestId: string, candidateIndex: number): Promise<void>;
  getCapabilities(): Promise<PlatformCapabilities>;
}

export const tauriWindowsCaptureBackend: WindowsCaptureBackend = {
  listenReady: (handler) => listen<CaptureReady>("capture-ready", ({ payload }) => handler(payload)),
  listenError: (handler) => listen<CaptureError>("capture-error", ({ payload }) => handler(payload)),
  listenOcrCandidate: (handler) => listen<OcrCandidatesReady>("ocr-candidate", ({ payload }) => handler(payload)),
  focus: () => invoke("focus_capture_window"),
  releaseFocus: () => invoke("release_capture_window_focus"),
  hide: (requestId) => invoke("hide_capture_window", { requestId }),
  save: async (requestId, correction, withoutTranslation) => {
    await invoke("correct_native_capture", { requestId, selectedText: correction.selectedText, sentence: correction.sentence, manualTranslation: correction.translation });
    return invoke<CaptureCard>("save_native_capture", { requestId, withoutTranslation });
  },
  undo: (requestId, encounterId) => invoke("undo_native_capture", { requestId, encounterId }),
  startOcr: (requestId) => invoke("capture_with_ocr", { requestId }),
  confirmOcr: (requestId, candidateIndex) => invoke("confirm_ocr", { requestId, candidateIndex }),
  getCapabilities: () => invoke("get_platform_capabilities"),
};
