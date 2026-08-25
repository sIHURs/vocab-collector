import { invoke } from "@tauri-apps/api/core";
import type {
  CaptureCard, CaptureInput, Encounter, ReviewRating, Settings, TodayView, WordDetail,
  WordListItem,
} from "./types";

export interface Backend {
  capture(input: CaptureInput): Promise<CaptureCard>;
  undoCapture(encounterId: string): Promise<void>;
  getToday(): Promise<TodayView>;
  listWords(): Promise<WordListItem[]>;
  getWord(wordId: string): Promise<WordDetail>;
  submitReview(wordId: string, rating: ReviewRating): Promise<void>;
  getSettings(): Promise<Settings>;
  updateSettings(settings: Settings): Promise<void>;
  replaceShortcut(candidate: string): Promise<Settings>;
}

type DemoWord = WordDetail & { due: boolean };

const defaultSettings: Settings = {
  sourceLanguage: "en", targetLanguage: "de", captureShortcut: "Alt+Shift+V",
  reviewTime: "18:00", dailyLimit: 5, launchAtLogin: false, appearance: "system",
  reducedMotion: false,
};

const id = () => globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;

export class DemoBackend implements Backend {
  private words: DemoWord[] = [];
  private settings = { ...defaultSettings };

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
      sourceTitle: "Reading notes", capturedAt, updatedAt: capturedAt,
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
    const due = this.words.filter((word) => word.due).slice(0, this.settings.dailyLimit);
    return {
      dueCount: due.length, estimatedMinutes: due.length ? Math.max(1, Math.ceil(due.length / 3)) : 0,
      reviewQueue: due.map((word) => ({ wordId: word.item.id,
        displayForm: word.item.displayForm, translation: word.item.translation,
        context: word.encounters[0]?.sentence })),
      recentCaptures: this.words.map((word) => ({ ...word.item })).slice(0, 8),
      settings: { ...this.settings },
    };
  }

  async listWords() { return this.words.map((word) => ({ ...word.item })); }
  async getWord(wordId: string) {
    const word = this.words.find((candidate) => candidate.item.id === wordId);
    if (!word) throw new Error("word not found");
    return structuredClone(word) as WordDetail;
  }
  async submitReview(wordId: string, _rating: ReviewRating) {
    const word = this.words.find((candidate) => candidate.item.id === wordId);
    if (word) { word.due = false; word.item.nextReviewAt = new Date(Date.now() + 86_400_000).toISOString(); }
  }
  async getSettings() { return { ...this.settings }; }
  async updateSettings(settings: Settings) { this.settings = { ...settings }; }
  async replaceShortcut(candidate: string) { this.settings.captureShortcut = candidate; return { ...this.settings }; }
}

class TauriBackend implements Backend {
  capture(input: CaptureInput) {
    return invoke<CaptureCard>("capture_word", { request: {
      selectedText: input.selectedText, lemma: input.selectedText, sentence: input.sentence,
      sourceLanguage: "en", targetLanguage: "de", translation: input.translation,
      partOfSpeech: undefined, sourceApp: input.sourceApp, sourceTitle: undefined,
      sourceUrl: undefined, capturedAt: new Date().toISOString(),
    }});
  }
  undoCapture(encounterId: string) { return invoke<void>("undo_capture", { encounterId }); }
  getToday() { return invoke<TodayView>("get_today"); }
  listWords() { return invoke<WordListItem[]>("list_words"); }
  getWord(wordId: string) { return invoke<WordDetail>("get_word", { wordId }); }
  submitReview(wordId: string, rating: ReviewRating) { return invoke<void>("submit_review", { wordId, rating }); }
  getSettings() { return invoke<Settings>("get_settings"); }
  updateSettings(settings: Settings) { return invoke<void>("update_settings", { settings }); }
  replaceShortcut(candidate: string) { return invoke<Settings>("replace_shortcut", { candidate }); }
}

export const createBackend = (): Backend => "__TAURI_INTERNALS__" in globalThis
  ? new TauriBackend()
  : new DemoBackend(true);

export const backend: Backend = createBackend();
