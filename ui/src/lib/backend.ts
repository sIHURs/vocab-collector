import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CaptureCard, CaptureInput, Encounter, PlatformCapabilities, ReviewRating, ReviewResult, ReviewSessionInsight, Settings,
  SettingsApplyResult, SystemSettingsStatus, TodayView,
  WordDetail, WordListItem,
} from "./types";

export interface Backend {
  capture(input: CaptureInput): Promise<CaptureCard>;
  undoCapture(encounterId: string): Promise<void>;
  getToday(): Promise<TodayView>;
  listWords(): Promise<WordListItem[]>;
  getWord(wordId: string): Promise<WordDetail>;
  submitReview(wordId: string, rating: ReviewRating, submissionId?: string): Promise<ReviewResult>;
  getReviewSessionInsight(results: ReviewResult[], nextDayEnd: string): Promise<ReviewSessionInsight>;
  getSettings(): Promise<Settings>;
  updateSettings(settings: Settings): Promise<void>;
  replaceShortcut(candidate: string): Promise<Settings>;
  applyWindowsSettings?(settings: Settings): Promise<SettingsApplyResult>;
  getWindowsSettingsStatus?(): Promise<SystemSettingsStatus>;
  listenLibraryChanged?(handler: () => void): Promise<() => void>;
  listenOpenManualCapture?(handler: () => void): Promise<() => void>;
  getPlatformCapabilities(): Promise<PlatformCapabilities>;
}

type DemoWord = WordDetail & { due: boolean };

const defaultSettings: Settings = {
  sourceLanguage: "en", targetLanguage: "de", selectionCaptureShortcut: "Alt+Shift+V",
  regionOcrCaptureShortcut: "Alt+Shift+O",
  reviewTime: "18:00", dailyLimit: 5, launchAtLogin: false, appearance: "system",
  reducedMotion: false, recentCapturesLimit: 20,
};

const unavailablePlatformCapabilities: PlatformCapabilities = {
  selectionCapture: false,
  selectionBounds: false,
  screenshotOcr: false,
  translation: false,
  nonActivatingWindow: false,
};

const id = (): string => globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;

export class DemoBackend implements Backend {
  private words: DemoWord[] = [];
  private settings = { ...defaultSettings };
  private reviewSubmissions = new Map<string, ReviewResult>();

  constructor(seed = true) {
    if (seed) {
      const samples = [
        ["serendipity", "glücklicher Zufall", "Finding something valuable by chance."],
        ["nuance", "Feinheit", "The essay captures every nuance of the debate."],
        ["resilient", "widerstandsfähig", "Small habits make a resilient practice."],
      ];
      for (const [word, translation, sentence] of samples) {
        this.addSeed(word, translation, sentence);
      }
    }
  }

  private addSeed(word: string, translation: string, sentence: string) {
    const wordId = id();
    const capturedAt = new Date(Date.now() - this.words.length * 3_600_000).toISOString();
    const encounter: Encounter = {
      id: id(), wordId, selectedText: word, sentence, sourceApp: "Safari",
      sourceTitle: "Reading notes", captureOrigin: "manual", capturedAt, updatedAt: capturedAt,
    };
    this.words.push({
      item: { id: wordId, displayForm: word, translation, status: "learning",
        encounterCount: 1, nextReviewAt: capturedAt, lastSeenAt: capturedAt },
      lemma: word, partOfSpeech: "word", encounters: [encounter], due: true,
    });
  }

  async capture(input: CaptureInput): Promise<CaptureCard> {
    const normalized = input.selectedText.trim().toLowerCase();
    let detail = this.words.find((word) => word.lemma === normalized);
    const existing = Boolean(detail);
    if (!detail) {
      const wordId = id();
      const now = new Date().toISOString();
      detail = {
        item: { id: wordId, displayForm: input.selectedText.trim(), translation: input.translation,
          status: "learning", encounterCount: 0, nextReviewAt: now, lastSeenAt: now },
        lemma: normalized, encounters: [], due: true,
      };
      this.words.unshift(detail);
    }
    const now = new Date().toISOString();
    const encounter: Encounter = {
      id: id(), wordId: detail.item.id, selectedText: input.selectedText.trim(),
      sentence: input.sentence.trim().replace(/\s+/g, " "), sourceApp: input.sourceApp,
      sourceTitle: input.sourceTitle, sourceUrl: input.sourceUrl,
      captureOrigin: input.captureOrigin ?? "manual",
      capturedAt: now, updatedAt: now,
    };
    detail.encounters.unshift(encounter);
    detail.item.encounterCount = detail.encounters.length;
    detail.item.lastSeenAt = now;
    if (input.translation) detail.item.translation = input.translation;
    return { wordId: detail.item.id, encounterId: encounter.id,
      displayForm: detail.item.displayForm, translation: detail.item.translation,
      context: encounter.sentence, encounterCount: detail.encounters.length,
      isExistingWord: existing };
  }

  async undoCapture(encounterId: string) {
    for (const word of this.words) {
      word.encounters = word.encounters.filter((encounter) => encounter.id !== encounterId);
      word.item.encounterCount = word.encounters.length;
    }
  }

  async getToday(): Promise<TodayView> {
    const allDue = this.words.filter((word) => word.due);
    const due = allDue.slice(0, this.settings.dailyLimit);
    return {
      totalDueCount: allDue.length, plannedReviewCount: due.length,
      estimatedMinutes: due.length ? Math.max(1, Math.ceil(due.length / 3)) : 0,
      reviewQueue: due.map((word) => ({ wordId: word.item.id,
        displayForm: word.item.displayForm, translation: word.item.translation,
        context: word.encounters[0]?.sentence })),
      recentCaptures: this.words.map((word) => ({ ...word.item })).slice(0, this.settings.recentCapturesLimit),
      settings: { ...this.settings },
    };
  }

  async listWords() { return this.words.map((word) => ({ ...word.item })); }
  async getWord(wordId: string) {
    const word = this.words.find((candidate) => candidate.item.id === wordId);
    if (!word) throw new Error("word not found");
    return structuredClone(word) as WordDetail;
  }
  async submitReview(wordId: string, rating: ReviewRating, submissionId: string = id()): Promise<ReviewResult> {
    const existing = this.reviewSubmissions.get(submissionId);
    if (existing) {
      if (existing.wordId !== wordId || existing.rating !== rating) throw new Error("review submission conflict");
      return { ...existing };
    }
    const word = this.words.find((candidate) => candidate.item.id === wordId);
    if (!word) throw new Error("word not found");
    const reviewedAt = new Date().toISOString();
    const previousDueAt = word.item.nextReviewAt ?? reviewedAt;
    const nextDueAt = new Date(Date.now() + (rating === "forgot" ? 86_400_000 : 259_200_000)).toISOString();
    word.due = false;
    word.item.nextReviewAt = nextDueAt;
    const result = { wordId, rating, reviewedAt, previousDueAt, nextDueAt,
      previousStability: 1, stability: rating === "forgot" ? 0.5 : 3,
      difficulty: rating === "forgot" ? 5.5 : 4.85, lapseCount: rating === "forgot" ? 1 : 0,
      encounterCount: word.encounters.length, repeatedForgetting: false };
    this.reviewSubmissions.set(submissionId, result);
    return { ...result };
  }
  async getReviewSessionInsight(results: ReviewResult[], nextDayEnd: string): Promise<ReviewSessionInsight> {
    return {
      reviewedCount: results.length,
      rememberedCount: results.filter((result) => result.rating === "remembered").length,
      forgottenCount: results.filter((result) => result.rating === "forgot").length,
      attentionWordIds: results.filter((result) => result.repeatedForgetting).map((result) => result.wordId),
      nextDayDueCount: this.words.filter((word) =>
        word.item.status === "learning" && Boolean(word.item.nextReviewAt) &&
        new Date(word.item.nextReviewAt!).getTime() <= new Date(nextDayEnd).getTime()).length,
    };
  }
  async getSettings() { return { ...this.settings }; }
  async updateSettings(settings: Settings) { this.settings = { ...settings }; }
  async replaceShortcut(candidate: string) { this.settings.selectionCaptureShortcut = candidate; return { ...this.settings }; }
  async applyWindowsSettings(settings: Settings): Promise<SettingsApplyResult> {
    await this.updateSettings(settings);
    return { settings: await this.getSettings() };
  }
  async getWindowsSettingsStatus(): Promise<SystemSettingsStatus> { return {}; }
  async getPlatformCapabilities() { return { ...unavailablePlatformCapabilities }; }
}

class TauriBackend implements Backend {
  capture(input: CaptureInput) {
    return invoke<CaptureCard>("capture_word", { request: {
      selectedText: input.selectedText, lemma: input.selectedText, sentence: input.sentence,
      sourceLanguage: "en", targetLanguage: "de", translation: input.translation,
      partOfSpeech: undefined, sourceApp: input.sourceApp, sourceTitle: input.sourceTitle,
      sourceUrl: input.sourceUrl, captureOrigin: input.captureOrigin ?? "manual",
      capturedAt: new Date().toISOString(),
    }});
  }
  undoCapture(encounterId: string) { return invoke<void>("undo_capture", { encounterId }); }
  getToday() { return invoke<TodayView>("get_today"); }
  listWords() { return invoke<WordListItem[]>("list_words"); }
  getWord(wordId: string) { return invoke<WordDetail>("get_word", { wordId }); }
  submitReview(wordId: string, rating: ReviewRating, submissionId = id()) {
    return invoke<ReviewResult>("submit_review", { submissionId, wordId, rating });
  }
  getReviewSessionInsight(results: ReviewResult[], nextDayEnd: string) {
    return invoke<ReviewSessionInsight>("get_review_session_insight", { results, nextDayEnd });
  }
  getSettings() { return invoke<Settings>("get_settings"); }
  updateSettings(settings: Settings) { return invoke<void>("update_settings", { settings }); }
  replaceShortcut(candidate: string) { return invoke<Settings>("replace_shortcut", { candidate }); }
  applyWindowsSettings(settings: Settings) {
    return invoke<SettingsApplyResult>("apply_windows_settings", { settings });
  }
  getWindowsSettingsStatus() { return invoke<SystemSettingsStatus>("get_windows_settings_status"); }
  listenLibraryChanged(handler: () => void) {
    return listen("library-changed", handler);
  }
  listenOpenManualCapture(handler: () => void) {
    return listen("open-manual-capture", handler);
  }
  getPlatformCapabilities() { return invoke<PlatformCapabilities>("get_platform_capabilities"); }
}

export const createBackend = (): Backend => "__TAURI_INTERNALS__" in globalThis
  ? new TauriBackend()
  : new DemoBackend(true);

export const backend: Backend = createBackend();
