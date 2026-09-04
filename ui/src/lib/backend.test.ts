import { describe, expect, it } from "vitest";

import { DemoBackend } from "./backend";

describe("browser backend contract", () => {
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
