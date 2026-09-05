import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AchievedCaptureConflict, AchievedWordListItem, CaptureCard, CaptureInput, Encounter, GlobalInsight, PlatformCapabilities, ReviewRating, ReviewResult, ReviewSessionInsight, Settings,
  SettingsApplyResult, SystemSettingsStatus, TodayView,
  WordDetail, WordListItem,
} from "./types";

export interface Backend {
  capture(input: CaptureInput): Promise<CaptureCard>;
  findAchievedCapture?(input: CaptureInput): Promise<AchievedCaptureConflict | null>;
  restoreAchievedAndCapture?(wordId: string, input: CaptureInput): Promise<CaptureCard>;
  undoCapture(encounterId: string): Promise<void>;
  getToday(): Promise<TodayView>;
  listWords(): Promise<WordListItem[]>;
  listAchievedWords?(): Promise<AchievedWordListItem[]>;
  achieveWord?(wordId: string): Promise<AchievedWordListItem>;
  unachieveWords?(wordIds: string[]): Promise<number>;
  deleteAchievedWords?(wordIds: string[]): Promise<number>;
  runLifecycleSweep?(): Promise<{ achievedCount: number; purgedCount: number }>;
  getWord(wordId: string): Promise<WordDetail>;
  submitReview(wordId: string, rating: ReviewRating, submissionId?: string): Promise<ReviewResult>;
  getReviewSessionInsight(submissionIds: string[], nextDayEnd: string): Promise<ReviewSessionInsight>;
  getGlobalInsight?(): Promise<GlobalInsight>;
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
  automaticAchieveEnabled: false, achievedRetentionDays: 30,
};

const unavailablePlatformCapabilities: PlatformCapabilities = {
  selectionCapture: false,
  selectionBounds: false,
  screenshotOcr: false,
  translation: false,
  nonActivatingWindow: false,
};

const id = (): string => globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
const achievedTiming = (deleteAfter: string) => {
  const remainingDays = Math.max(0, Math.ceil((new Date(deleteAfter).getTime() - Date.now()) / 86_400_000));
  return { remainingDays, urgency: remainingDays <= 2 ? "urgent" as const : remainingDays <= 7 ? "warning" as const : "normal" as const };
};

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

  async listWords() { return this.words.filter((word) => !word.item.achievedAt).map((word) => ({ ...word.item })); }
  async listAchievedWords(): Promise<AchievedWordListItem[]> {
    return this.words.filter((word) => word.item.achievedAt && word.item.deleteAfter).map((word) => ({
      id: word.item.id, lemma: word.lemma, displayForm: word.item.displayForm, translation: word.item.translation,
      encounterCount: word.item.encounterCount, achievedAt: word.item.achievedAt!, deleteAfter: word.item.deleteAfter!,
      ...achievedTiming(word.item.deleteAfter!),
    }));
  }

  async findAchievedCapture(input: CaptureInput): Promise<AchievedCaptureConflict | null> {
    const normalized = input.selectedText.trim().toLowerCase();
    const word = this.words.find((candidate) => candidate.lemma === normalized && candidate.item.achievedAt && candidate.item.deleteAfter);
    return word ? { wordId: word.item.id, displayForm: word.item.displayForm, achievedAt: word.item.achievedAt!, deleteAfter: word.item.deleteAfter! } : null;
  }
  async restoreAchievedAndCapture(wordId: string, input: CaptureInput): Promise<CaptureCard> {
    const word = this.words.find((candidate) => candidate.item.id === wordId && candidate.item.achievedAt);
    if (!word) throw new Error("Achieved Vocabulary Item changed; try capturing again");
    word.item.status = "learning";
    word.item.achievedAt = undefined;
    word.item.deleteAfter = undefined;
    return this.capture(input);
  }
  async achieveWord(wordId: string): Promise<AchievedWordListItem> {
    const word = this.words.find((candidate) => candidate.item.id === wordId);
    if (!word || word.item.status !== "mastered" || word.item.achievedAt) throw new Error("only an active Mastered Vocabulary Item can be Achieved");
    const achievedAt = new Date();
    const deleteAfter = new Date(achievedAt.getTime() + (this.settings.achievedRetentionDays ?? 30) * 86_400_000);
    word.item.achievedAt = achievedAt.toISOString(); word.item.deleteAfter = deleteAfter.toISOString();
    return { id: word.item.id, lemma: word.lemma, displayForm: word.item.displayForm, translation: word.item.translation,
      encounterCount: word.item.encounterCount, achievedAt: word.item.achievedAt, deleteAfter: word.item.deleteAfter,
      ...achievedTiming(word.item.deleteAfter) };
  }
  async unachieveWords(wordIds: string[]): Promise<number> {
    for (const wordId of wordIds) {
      const word = this.words.find((candidate) => candidate.item.id === wordId && candidate.item.achievedAt);
      if (!word) throw new Error("Vocabulary Item is not Achieved");
    }
    for (const word of this.words.filter((candidate) => wordIds.includes(candidate.item.id))) {
      word.item.achievedAt = undefined; word.item.deleteAfter = undefined;
    }
    return wordIds.length;
  }
  async deleteAchievedWords(wordIds: string[]): Promise<number> {
    if (wordIds.some((wordId) => !this.words.some((candidate) => candidate.item.id === wordId && candidate.item.achievedAt))) throw new Error("Vocabulary Item is not Achieved");
    this.words = this.words.filter((candidate) => !wordIds.includes(candidate.item.id));
    return wordIds.length;
  }
  async runLifecycleSweep() { return { achievedCount: 0, purgedCount: 0 }; }
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
    const result = { submissionId, wordId, rating, reviewedAt, previousDueAt, nextDueAt,
      previousStability: 1, stability: rating === "forgot" ? 0.5 : 3,
      difficulty: rating === "forgot" ? 5.5 : 4.85, lapseCount: rating === "forgot" ? 1 : 0,
      encounterCount: word.encounters.length, repeatedForgetting: false };
    this.reviewSubmissions.set(submissionId, result);
    return { ...result };
  }
  async getReviewSessionInsight(submissionIds: string[], nextDayEnd: string): Promise<ReviewSessionInsight> {
    const results = [...new Set(submissionIds)]
      .map((submissionId) => this.reviewSubmissions.get(submissionId))
      .filter((result): result is ReviewResult => Boolean(result));
    return {
      reviewedCount: results.length,
      rememberedCount: results.filter((result) => result.rating === "remembered").length,
      forgottenCount: results.filter((result) => result.rating === "forgot").length,
      attentionWordIds: results.filter((result) => result.repeatedForgetting).map((result) => result.wordId),
      nextDayDueCount: this.words.filter((word) =>
        word.item.status === "learning" && Boolean(word.item.nextReviewAt) &&
        new Date(word.item.nextReviewAt!).getTime() < new Date(nextDayEnd).getTime()).length,
    };
  }
  async getGlobalInsight(): Promise<GlobalInsight> {
    const reviews = [...this.reviewSubmissions.values()];
    return {
      currentVocabularyCount: this.words.length,
      currentAchievedCount: this.words.filter((word) => word.item.achievedAt).length,
      lifetimeVocabularyCount: this.words.length,
      lifetimeEncounterCount: this.words.reduce((total, word) => total + word.encounters.length, 0),
      lifetimeReviewCount: reviews.length,
      lifetimeRememberedCount: reviews.filter((review) => review.rating === "remembered").length,
      lifetimeForgottenCount: reviews.filter((review) => review.rating === "forgot").length,
      lifetimeRatingBreakdownComplete: true,
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
  private captureRequest(input: CaptureInput) {
    return {
      selectedText: input.selectedText, lemma: input.selectedText, sentence: input.sentence,
      sourceLanguage: "en", targetLanguage: "de", translation: input.translation,
      partOfSpeech: undefined, sourceApp: input.sourceApp, sourceTitle: input.sourceTitle,
      sourceUrl: input.sourceUrl, captureOrigin: input.captureOrigin ?? "manual",
      capturedAt: new Date().toISOString(),
    };
  }
  capture(input: CaptureInput) {
    return invoke<CaptureCard>("capture_word", { request: this.captureRequest(input) });
  }
  findAchievedCapture(input: CaptureInput) { return invoke<AchievedCaptureConflict | null>("find_achieved_capture", { request: this.captureRequest(input) }); }
  restoreAchievedAndCapture(wordId: string, input: CaptureInput) { return invoke<CaptureCard>("restore_achieved_and_capture", { wordId, request: this.captureRequest(input) }); }
  undoCapture(encounterId: string) { return invoke<void>("undo_capture", { encounterId }); }
  getToday() { return invoke<TodayView>("get_today"); }
  listWords() { return invoke<WordListItem[]>("list_words"); }
  listAchievedWords() { return invoke<AchievedWordListItem[]>("list_achieved_words"); }
  achieveWord(wordId: string) { return invoke<AchievedWordListItem>("achieve_word", { wordId }); }
  unachieveWords(wordIds: string[]) { return invoke<number>("unachieve_words", { wordIds }); }
  deleteAchievedWords(wordIds: string[]) { return invoke<number>("delete_achieved_words", { wordIds }); }
  runLifecycleSweep() { return invoke<{ achievedCount: number; purgedCount: number }>("run_lifecycle_sweep"); }
  getWord(wordId: string) { return invoke<WordDetail>("get_word", { wordId }); }
  submitReview(wordId: string, rating: ReviewRating, submissionId = id()) {
    return invoke<ReviewResult>("submit_review", { submissionId, wordId, rating });
  }
  getReviewSessionInsight(submissionIds: string[], nextDayEnd: string) {
    return invoke<ReviewSessionInsight>("get_review_session_insight", { submissionIds, nextDayEnd });
  }
  getGlobalInsight() { return invoke<GlobalInsight>("get_global_insight"); }
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
