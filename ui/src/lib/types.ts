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
  captureShortcut: string;
  reviewTime: string;
  dailyLimit: number;
  launchAtLogin: boolean;
  appearance: "system" | "light" | "dark";
  reducedMotion: boolean;
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

export interface TodayView {
  dueCount: number;
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
