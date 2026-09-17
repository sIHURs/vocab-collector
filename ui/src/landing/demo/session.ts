import type { Backend } from '../../lib/backend';
import type { CaptureInput, Settings, WordDetail, ReviewRating, ReviewResult } from '../../lib/types';
import type { WindowsCaptureBackend, CaptureReady } from '../../windows/captureBackend';

export const fixtureTime = '2026-09-17T12:00:00.000Z';
export const preparedCapture = {
  selectedText: 'ephemeral', sentence: 'The ephemeral light made the garden feel new.',
  sourceApp: 'Browser demo', sourceTitle: 'A quiet garden',
  translation: '短暂的', captureOrigin: 'manual' as const,
};
const settings: Settings = {
  sourceLanguage: 'en', targetLanguage: 'zh-Hans', selectionCaptureShortcut: '',
  regionOcrCaptureShortcut: '', reviewTime: '18:00', dailyLimit: 2, recentCapturesLimit: 20,
  launchAtLogin: false, appearance: 'light', reducedMotion: false,
};
const capabilities = { selectionCapture: false, selectionBounds: false, screenshotOcr: false, translation: false, nonActivatingWindow: false };
const unavailable = async (): Promise<never> => { throw new Error('Unavailable in this browser demo'); };
const clone = <T>(value: T): T => structuredClone(value);

/** One volatile browser session. No native imports, storage, or provider calls. */
export class LandingSession implements Backend {
  private sequence = 0;
  private nextId = () => `landing-${++this.sequence}`;
  private words = new Map<string, WordDetail>();
  private due = new Set(['serendipitous', 'resilient']);
  private listeners = new Set<() => void>();
  private reviews = new Map<string, ReviewResult>();
  private undo = new Map<string, { wordId: string; before?: WordDetail }>();
  constructor(private now: () => string = () => fixtureTime) {
    for (const [word, translation, sentence] of [
      ['serendipitous', '偶然发现的', 'Their serendipitous meeting began a friendship.'],
      ['resilient', '有韧性的', 'The resilient tree survived the storm.'],
      ['nuance', '细微差别', 'She noticed a nuance in his answer.'],
      ['luminous', '明亮的', 'The luminous sky slowly faded.'],
    ]) {
      const savedTranslation = { targetLanguage: 'zh-Hans', text: translation, savedAt: fixtureTime };
      this.words.set(word, {
        item: { id: word, displayForm: word, translation, translationLanguage: 'zh-Hans', status: 'learning', encounterCount: 1, lastSeenAt: fixtureTime, nextReviewAt: this.due.has(word) ? fixtureTime : undefined },
        lemma: word, translations: [savedTranslation],
        encounters: [{ id: `seed-${word}`, wordId: word, selectedText: word, sentence,
          sourceTitle: 'Reading notes', sourceApp: 'Browser demo', captureOrigin: 'manual',
          capturedAt: fixtureTime, updatedAt: fixtureTime, savedTranslation }],
      });
    }
  }
  private notify() { for (const listener of this.listeners) listener(); }
  async listenLibraryChanged(listener: () => void) { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; }
  async listWords() { return clone([...this.words.values()].map(word => word.item)); }
  async getWord(id: string) {
    const word = this.words.get(id);
    if (!word) throw new Error('Vocabulary Item not found');
    return clone(word);
  }
  async capture(input: CaptureInput) {
    const normalized = input.selectedText.trim().replace(/\s+/g, ' ').toLowerCase();
    if (!normalized) throw new Error('Enter a word');
    let word = [...this.words.values()].find(word => word.lemma === normalized);
    const before = word ? clone(word) : undefined;
    const now = this.now();
    if (!word) word = { lemma: normalized, encounters: [], item: { id: this.nextId(), displayForm: input.selectedText.trim(), status: 'learning', encounterCount: 0, lastSeenAt: now } };
    const encounterId = this.nextId();
    const savedTranslation = input.translation?.trim() ? { text: input.translation.trim(), targetLanguage: 'zh-Hans', savedAt: now } : undefined;
    word.encounters.unshift({ id: encounterId, wordId: word.item.id, selectedText: input.selectedText.trim(),
      sentence: input.sentence.trim(), sourceTitle: input.sourceTitle, sourceApp: input.sourceApp,
      sourceUrl: input.sourceUrl, captureOrigin: input.captureOrigin ?? 'manual',
      capturedAt: now, updatedAt: now, savedTranslation });
    if (savedTranslation) {
      word.translations = [savedTranslation];
      word.item.translation = savedTranslation.text;
      word.item.translationLanguage = savedTranslation.targetLanguage;
    }
    word.item.encounterCount = word.encounters.length;
    word.item.lastSeenAt = now;
    this.words.set(word.item.id, word);
    this.undo.set(encounterId, { wordId: word.item.id, before });
    this.notify();
    return { wordId: word.item.id, encounterId, displayForm: word.item.displayForm,
      translation: savedTranslation?.text, context: input.sentence,
      encounterCount: word.encounters.length, isExistingWord: Boolean(before) };
  }
  async undoCapture(encounterId: string) {
    const snapshot = this.undo.get(encounterId);
    if (!snapshot || this.words.get(snapshot.wordId)?.encounters[0]?.id !== encounterId) throw new Error('Only the latest capture can be undone');
    const current = this.words.get(snapshot.wordId)!;
    if (snapshot.before) {
      // Undo capture must not roll back a Review completed after that save.
      snapshot.before.item.nextReviewAt = current.item.nextReviewAt;
      this.words.set(snapshot.wordId, snapshot.before);
    } else this.words.delete(snapshot.wordId);
    this.undo.delete(encounterId);
    this.notify();
  }
  async getToday(completed: string[] = []) {
    const reviewQueue = [...this.due].filter(id => !completed.includes(id)).map(id => {
      const word = this.words.get(id)!;
      return { wordId: id, displayForm: word.item.displayForm, translation: word.item.translation,
        translationLanguage: 'zh-Hans', context: word.encounters[0]?.sentence };
    });
    return { totalDueCount: reviewQueue.length, plannedReviewCount: reviewQueue.length,
      estimatedMinutes: reviewQueue.length ? 1 : 0, reviewQueue, recentCaptures: await this.listWords(), settings: await this.getSettings() };
  }
  async submitReview(wordId: string, rating: ReviewRating, submissionId = this.nextId()) {
    const previous = this.reviews.get(submissionId);
    if (previous) {
      if (previous.wordId !== wordId || previous.rating !== rating) throw new Error('Review submission conflict');
      return clone(previous);
    }
    if (!this.due.has(wordId)) throw new Error('This Vocabulary Item is not in the due demo scope');
    const word = this.words.get(wordId)!;
    const nextDueAt = new Date(Date.parse(this.now()) + (rating === 'forgot' ? 1 : 3) * 86400000).toISOString();
    const result: ReviewResult = { submissionId, wordId, rating, reviewedAt: this.now(),
      previousDueAt: word.item.nextReviewAt!, nextDueAt, previousStability: 1,
      stability: rating === 'forgot' ? 0.5 : 3, difficulty: rating === 'forgot' ? 5.5 : 4.85,
      lapseCount: rating === 'forgot' ? 1 : 0, encounterCount: word.encounters.length, repeatedForgetting: false };
    word.item.nextReviewAt = nextDueAt;
    this.due.delete(wordId);
    this.reviews.set(submissionId, result);
    return clone(result);
  }
  async getReviewSessionInsight(ids: string[], _nextDayEnd: string) {
    const results = [...new Set(ids)].flatMap(id => this.reviews.has(id) ? [this.reviews.get(id)!] : []);
    // Fixed demo dates, independent of the visitor's real calendar/time zone.
    const nextDayEnd = Date.parse(this.now()) + 86400000;
    return { reviewedCount: results.length, rememberedCount: results.filter(r => r.rating === 'remembered').length,
      forgottenCount: results.filter(r => r.rating === 'forgot').length, attentionWordIds: [],
      nextDayDueCount: [...this.words.values()].filter(w => w.item.nextReviewAt && Date.parse(w.item.nextReviewAt) <= nextDayEnd).length };
  }
  async getSettings() { return clone(settings); }
  updateSettings = unavailable;
  replaceShortcut = unavailable;
  async getPlatformCapabilities() { return clone(capabilities); }
}

/** A fresh adapter per mounted Capture cancels the previous request by disposal. */
export function createCaptureAdapter(session: LandingSession, onclose: () => void): WindowsCaptureBackend {
  const requestId = 'prepared-capture';
  let draft: CaptureInput = { ...preparedCapture };
  const check = (id: string) => { if (id !== requestId) throw new Error('Stale Capture Candidate'); };
  const noopListen = async () => () => {};
  return {
    async listenReady(handler: (event: CaptureReady) => void) {
      handler({ requestId, candidate: { ...preparedCapture, origin: 'manual' } });
      return () => {};
    },
    listenError: noopListen, listenOcrCandidate: noopListen, listenRegionOcrStart: noopListen,
    focus: async () => {}, releaseFocus: async () => {},
    close: async id => { check(id); onclose(); }, hide: async id => { check(id); onclose(); },
    apply: async (id, correction) => { check(id); draft = { ...draft, ...correction, translation: correction.translation }; },
    save: async (id, withoutTranslation) => { check(id); return session.capture({ ...draft, translation: withoutTranslation ? undefined : draft.translation }); },
    findAchieved: async () => null, restoreAchievedAndSave: unavailable,
    undo: async (id, encounterId) => { check(id); await session.undoCapture(encounterId); },
    recognizeRegion: unavailable, startRegionOcr: unavailable, openManualCapture: unavailable, confirmOcr: unavailable,
    getCapabilities: async () => ({ ...capabilities, translation: true }),
    getSettings: () => session.getSettings(),
    translate: async (id, text) => {
      check(id);
      if (text.trim().toLowerCase() !== preparedCapture.selectedText) throw new Error('No preset translation; enter a manual translation');
      return { translatedText: preparedCapture.translation, sourceLanguage: 'en', targetLanguage: 'zh-Hans' };
    },
  };
}
