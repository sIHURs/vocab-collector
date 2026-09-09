import { describe, expect, it } from "vitest";

import { DemoBackend, captureRequestFor } from "./backend";

describe("browser backend contract", () => {
  it("uses the saved language pair when constructing a desktop Manual Capture request", () => {
    const capturedAt = "2026-09-05T12:00:00.000Z";
    const request = captureRequestFor(
      { selectedText: "validate", sentence: "Validate the result." },
      { sourceLanguage: "auto", targetLanguage: "zh-Hans", selectionCaptureShortcut: "", regionOcrCaptureShortcut: "", reviewTime: "18:00", dailyLimit: 5, recentCapturesLimit: 20, launchAtLogin: false, appearance: "system", reducedMotion: false },
      capturedAt,
    );

    expect(request).toMatchObject({ sourceLanguage: "auto", targetLanguage: "zh-Hans", capturedAt });
  });

  it("deduplicates words while preserving encounters", async () => {
    const backend = new DemoBackend(false);

    const first = await backend.capture({
      selectedText: "Serendipity",
      sentence: "A moment of serendipity.",
      translation: "glücklicher Zufall",
      sourceApp: "Safari",
    });
    const repeated = await backend.capture({
      selectedText: "serendipity",
      sentence: "Another unexpected discovery.",
      translation: "glücklicher Zufall",
      sourceApp: "Reader",
    });

    expect(repeated.wordId).toBe(first.wordId);
    expect(repeated.isExistingWord).toBe(true);
    expect(repeated.encounterCount).toBe(2);
    expect((await backend.getWord(first.wordId)).encounters).toHaveLength(2);
  });

  it("removes reviewed cards from today's queue", async () => {
    const backend = new DemoBackend(true);
    const today = await backend.getToday();

    await backend.submitReview(today.reviewQueue[0].wordId, "remembered");

    expect((await backend.getToday()).totalDueCount).toBe(today.totalDueCount - 1);
  });

  it("summarizes each persisted review submission only once", async () => {
    const backend = new DemoBackend(true);
    const today = await backend.getToday();
    const result = await backend.submitReview(today.reviewQueue[0].wordId, "remembered", "submission-1");
    const nextDayEnd = new Date(Date.now() + 2 * 86_400_000).toISOString();

    const insight = await backend.getReviewSessionInsight(
      [result.submissionId, result.submissionId, "unknown"], nextDayEnd
    );

    expect(insight.reviewedCount).toBe(1);
    expect(insight.rememberedCount).toBe(1);
    expect(insight.forgottenCount).toBe(0);
  });
});

it('shares translations across target languages and undoes only an uncontested save', async () => {
  const backend = new DemoBackend(false);
  const first = await backend.capture({selectedText:'robust',sentence:'A robust design.',translation:'kräftig'});
  await backend.updateSettings({...await backend.getSettings(),targetLanguage:'zh-Hans'});
  const second = await backend.capture({selectedText:'robust',sentence:'Another robust design.',translation:'稳健的'});
  expect(second.wordId).toBe(first.wordId);
  expect((await backend.getWord(first.wordId)).translations).toHaveLength(2);
  expect((await backend.listWords())[0].translation).toBe('稳健的');
  await backend.undoCapture(second.encounterId);
  expect((await backend.getWord(first.wordId)).translations).toHaveLength(1);
  const third = await backend.capture({selectedText:'robust',sentence:'Third context.',translation:'强健的'});
  await backend.submitReview(first.wordId,'remembered');
  await expect(backend.undoCapture(third.encounterId)).rejects.toThrow('Cannot undo');
});
