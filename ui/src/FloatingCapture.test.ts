import { render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import FloatingCapture from "./FloatingCapture.svelte";

const mocks = vi.hoisted(() => ({
  handlers: new Map<string, (event: { payload: unknown }) => void>(),
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    mocks.handlers.set(name, handler);
    return () => mocks.handlers.delete(name);
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));

const candidate = (selectedText: string) => ({
  selectedText,
  sentence: `Context for ${selectedText}.`,
  sourceApp: "Safari",
  origin: "accessibility",
});

describe("floating capture request freshness", () => {
  afterEach(() => {
    mocks.handlers.clear();
    mocks.invoke.mockReset();
  });

  it("does not let a late failure from an old request replace the current card", async () => {
    let rejectFirstSettings!: (error: Error) => void;
    const firstSettings = new Promise((_, reject) => { rejectFirstSettings = reject; });
    mocks.invoke.mockImplementation((command: string, args?: { requestId?: string }) => {
      if (command === "get_settings" && mocks.invoke.mock.calls.filter(([name]) => name === "get_settings").length === 1) return firstSettings;
      if (command === "get_settings") return Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" });
      if (command === "translate_text") return Promise.resolve({ translatedText: "aktuell" });
      if (command === "save_native_capture") return Promise.resolve({
        wordId: "word-b", encounterId: "encounter-b", displayForm: "current",
        translation: "aktuell", context: "Context for current.", encounterCount: 1,
        isExistingWord: false, requestId: args?.requestId,
      });
      return Promise.resolve();
    });

    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));
    mocks.handlers.get("capture-ready")?.({ payload: { requestId: "request-a", candidate: candidate("old") } });
    mocks.handlers.get("capture-ready")?.({ payload: { requestId: "request-b", candidate: candidate("current") } });

    expect(await screen.findByText("current")).toBeVisible();
    rejectFirstSettings(new Error("late failure"));
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(screen.queryByText("late failure")).not.toBeInTheDocument();
    expect(screen.getByText("current")).toBeVisible();
  });
});
