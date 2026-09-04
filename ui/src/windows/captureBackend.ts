import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CaptureCandidate, CaptureCard, CaptureFailure, PlatformCapabilities, Settings, TranslationResult } from "../lib/types";

export type CaptureReady = { requestId: string; candidate: CaptureCandidate };
export type CaptureError = { requestId: string; failure: CaptureFailure };
type CaptureErrorWire = { requestId: string; code: CaptureFailure["code"]; message: string };
export type OcrCandidate = { text: string; bounds: { x: number; y: number; width: number; height: number }; confidence: number };
export type OcrCandidatesReady = { requestId: string; candidates: OcrCandidate[]; ambiguous: boolean };
export type RegionOcrStart = { requestId: string };

export interface WindowsCaptureBackend {
  listenReady(handler: (event: CaptureReady) => void): Promise<() => void>;
  listenError(handler: (event: CaptureError) => void): Promise<() => void>;
  listenOcrCandidate(handler: (event: OcrCandidatesReady) => void): Promise<() => void>;
  listenRegionOcrStart(handler: (event: RegionOcrStart) => void): Promise<() => void>;
  focus(): Promise<void>;
  releaseFocus(): Promise<void>;
  close(requestId: string): Promise<void>;
  hide(requestId: string): Promise<void>;
  apply(requestId: string, correction: { selectedText: string; sentence: string; translation?: string }): Promise<void>;
  save(requestId: string, withoutTranslation: boolean): Promise<CaptureCard>;
  undo(requestId: string, encounterId: string): Promise<void>;
  recognizeRegion(requestId: string, region: { x: number; y: number; width: number; height: number }): Promise<void>;
  startRegionOcr(): Promise<string>;
  openManualCapture(): Promise<void>;
  confirmOcr(requestId: string, selectedText: string, sentence: string): Promise<void>;
  getCapabilities(): Promise<PlatformCapabilities>;
  getSettings(): Promise<Settings>;
  translate(requestId: string, text: string, sourceLanguage: string, targetLanguage: string): Promise<TranslationResult>;
}

export const tauriWindowsCaptureBackend: WindowsCaptureBackend = {
  listenReady: (handler) => listen<CaptureReady>("capture-ready", ({ payload }) => handler(payload)),
  listenError: (handler) => listen<CaptureErrorWire>("capture-error", ({ payload }) => handler({ requestId: payload.requestId, failure: { code: payload.code, message: payload.message } })),
  listenOcrCandidate: (handler) => listen<OcrCandidatesReady>("ocr-candidate", ({ payload }) => handler(payload)),
  listenRegionOcrStart: (handler) => listen<RegionOcrStart>("region-ocr-start", ({ payload }) => handler(payload)),
  focus: () => invoke("focus_capture_window"),
  releaseFocus: () => invoke("release_capture_window_focus"),
  close: (requestId) => invoke("close_capture_window", { requestId }),
  hide: (requestId) => invoke("hide_capture_window", { requestId }),
  apply: (requestId, correction) => invoke("correct_native_capture", { requestId, selectedText: correction.selectedText, sentence: correction.sentence, manualTranslation: correction.translation }),
  save: (requestId, withoutTranslation) => invoke<CaptureCard>("save_native_capture", { requestId, withoutTranslation }),
  undo: (requestId, encounterId) => invoke("undo_native_capture", { requestId, encounterId }),
  recognizeRegion: (requestId, region) => invoke("capture_ocr_region", { requestId, region }),
  startRegionOcr: () => invoke<string>("start_region_ocr_capture"),
  openManualCapture: () => invoke("open_manual_capture"),
  confirmOcr: (requestId, selectedText, sentence) => invoke("confirm_ocr", { requestId, selectedText, sentence }),
  getCapabilities: () => invoke("get_platform_capabilities"),
  getSettings: () => invoke("get_settings"),
  translate: (requestId, text, sourceLanguage, targetLanguage) => invoke("translate_text", { requestId, text, sourceLanguage, targetLanguage }),
};
