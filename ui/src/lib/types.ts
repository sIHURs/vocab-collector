export type WordStatus = "learning" | "mastered" | "paused";
export type ReviewRating = "forgot" | "remembered";

export type CaptureFailureCode =
  | "permission_required"
  | "permission_denied"
  | "empty_selection"
  | "unsupported_element"
  | "translation_unavailable"
  | "translation_failed"
  | "cancelled"
  | "operation";

export interface CaptureFailure {
  code: CaptureFailureCode;
  message: string;
}

export interface PlatformCapabilities {
  selectionCapture: boolean;
  selectionBounds: boolean;
  screenshotOcr: boolean;
  translation: boolean;
  nonActivatingWindow: boolean;
}

export interface Settings {
  sourceLanguage: string;
  targetLanguage: string;
  selectionCaptureShortcut: string;
  regionOcrCaptureShortcut: string;
  reviewTime: string;
  dailyLimit: number;
  recentCapturesLimit: number;
  launchAtLogin: boolean;
  appearance: "system" | "light" | "dark";
  reducedMotion: boolean;
}

export interface TranslationResult {
  translatedText: string;
  sourceLanguage: string;
  targetLanguage: string;
}

export interface SystemSettingsStatus {
  selectionShortcutError?: string;
  regionOcrShortcutError?: string;
  autostartError?: string;
  notificationError?: string;
}

export interface SettingsApplyResult extends SystemSettingsStatus {
  settings: Settings;
}

export interface CaptureInput {
  selectedText: string;
  sentence: string;
  translation?: string;
  sourceApp?: string;
  sourceTitle?: string;
  sourceUrl?: string;
  captureOrigin?: "manual" | "accessibility" | "ocr";
}

export interface CaptureCard {
  wordId: string;
  encounterId: string;
  displayForm: string;
  translation?: string;
  context: string;
  encounterCount: number;
  isExistingWord: boolean;
}

export interface CaptureCandidate {
  selectedText: string;
  sentence: string;
  sourceApp?: string;
  sourceTitle?: string;
  sourceUrl?: string;
  selectionBounds?: { x: number; y: number; width: number; height: number };
  origin: "manual" | "accessibility" | "ocr";
}

export interface Encounter {
  id: string;
  wordId: string;
  selectedText: string;
  sentence: string;
  sourceApp?: string;
  sourceTitle?: string;
  sourceUrl?: string;
  captureOrigin: "manual" | "accessibility" | "ocr";
  capturedAt: string;
  updatedAt: string;
  deletedAt?: string;
}

export interface WordListItem {
  id: string;
  displayForm: string;
  translation?: string;
  status: WordStatus;
  encounterCount: number;
  nextReviewAt?: string;
  lastSeenAt: string;
}

export interface ReviewCard {
  wordId: string;
  displayForm: string;
  translation?: string;
  context?: string;
}

export interface ReviewResult {
  wordId: string;
  rating: ReviewRating;
  reviewedAt: string;
  previousDueAt: string;
  nextDueAt: string;
  previousStability: number;
  stability: number;
  difficulty: number;
  lapseCount: number;
  encounterCount: number;
  repeatedForgetting: boolean;
}

export interface ReviewSessionInsight {
  reviewedCount: number;
  rememberedCount: number;
  forgottenCount: number;
  attentionWordIds: string[];
  nextDayDueCount: number;
}

export interface TodayView {
  totalDueCount: number;
  plannedReviewCount: number;
  estimatedMinutes: number;
  reviewQueue: ReviewCard[];
  recentCaptures: WordListItem[];
  settings: Settings;
}

export interface WordDetail {
  item: WordListItem;
  lemma: string;
  partOfSpeech?: string;
  encounters: Encounter[];
}
