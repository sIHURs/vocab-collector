import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import WindowsFloatingCapture from "./WindowsFloatingCapture.svelte";
import type { CaptureReady, OcrCandidatesReady, RegionOcrStart, WindowsCaptureBackend } from "./captureBackend";

const mocks = {
  focus: vi.fn(async () => {}),
  releaseFocus: vi.fn(async () => {}),
  close: vi.fn(async (_requestId: string) => {}),
  hide: vi.fn(async (_requestId: string) => {}),
  apply: vi.fn(async () => {}),
  save: vi.fn(async () => ({ wordId: "word-1", encounterId: "encounter-1", displayForm: "nuance", context: "A useful nuance.", encounterCount: 1, isExistingWord: false })),
  undo: vi.fn(async (_requestId: string, _encounterId: string) => {}),
  recognizeRegion: vi.fn(async () => {}),
  startRegionOcr: vi.fn(async () => "region-request"),
  openManualCapture: vi.fn(async () => {}),
  confirmOcr: vi.fn(async (_requestId: string, _selectedText: string, _sentence: string) => {}),
  getCapabilities: vi.fn(async () => ({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: false, nonActivatingWindow: false })),
  getSettings: vi.fn(async () => ({ sourceLanguage: "auto", targetLanguage: "de", selectionCaptureShortcut: "Alt+Shift+V", regionOcrCaptureShortcut: "Alt+Shift+O", reviewTime: "18:00", dailyLimit: 20, recentCapturesLimit: 10, launchAtLogin: false, appearance: "system" as const, reducedMotion: false })),
  translate: vi.fn(async () => ({ translatedText: "Feinheit", sourceLanguage: "en", targetLanguage: "de" })),
  ready: undefined as ((event: CaptureReady) => void) | undefined,
  error: undefined as ((event: { requestId: string; failure: { code: "empty_selection" | "unsupported_element" | "operation"; message: string } }) => void) | undefined,
  ocr: undefined as ((event: OcrCandidatesReady) => void) | undefined,
  regionOcr: undefined as ((event: RegionOcrStart) => void) | undefined,
};

const captureBackend: WindowsCaptureBackend = {
  listenReady: async (handler) => { mocks.ready = handler; return () => { mocks.ready = undefined; }; },
  focus: mocks.focus,
  releaseFocus: mocks.releaseFocus,
  close: mocks.close,
  hide: mocks.hide,
  apply: mocks.apply,
  save: mocks.save,
  undo: mocks.undo,
  recognizeRegion: mocks.recognizeRegion,
  startRegionOcr: mocks.startRegionOcr,
  openManualCapture: mocks.openManualCapture,
  confirmOcr: mocks.confirmOcr,
  getCapabilities: mocks.getCapabilities,
  getSettings: mocks.getSettings,
  translate: mocks.translate,
  listenError: async (handler) => { mocks.error = handler; return () => { mocks.error = undefined; }; },
  listenOcrCandidate: async (handler) => { mocks.ocr = handler; return () => { mocks.ocr = undefined; }; },
  listenRegionOcrStart: async (handler) => { mocks.regionOcr = handler; return () => { mocks.regionOcr = undefined; }; },
};

describe("Windows floating capture presentation", () => {
  beforeEach(() => {
    mocks.focus.mockClear();
    mocks.releaseFocus.mockClear();
    mocks.close.mockClear();
    mocks.hide.mockClear();
    mocks.apply.mockClear();
    mocks.save.mockReset();
    mocks.save.mockResolvedValue({ wordId: "word-1", encounterId: "encounter-1", displayForm: "nuance", context: "A useful nuance.", encounterCount: 1, isExistingWord: false });
    mocks.undo.mockClear();
    mocks.recognizeRegion.mockClear();
    mocks.startRegionOcr.mockClear();
    mocks.openManualCapture.mockClear();
    mocks.confirmOcr.mockReset();
    mocks.confirmOcr.mockResolvedValue(undefined);
    mocks.getCapabilities.mockReset();
    mocks.getSettings.mockClear();
    mocks.translate.mockReset();
    mocks.translate.mockResolvedValue({ translatedText: "Feinheit", sourceLanguage: "en", targetLanguage: "de" });
    mocks.getCapabilities.mockResolvedValue({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: false, nonActivatingWindow: false });
    mocks.ready = undefined;
    mocks.error = undefined;
    mocks.ocr = undefined;
    mocks.regionOcr = undefined;
  });

  it("automatically previews a native translation, applies independent edits, and saves only on confirmation", async () => {
    mocks.getCapabilities.mockResolvedValue({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: true, nonActivatingWindow: false });
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));

    mocks.ready?.({ requestId: "translated-request", candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" } });

    expect(await screen.findByText("Feinheit")).toBeVisible();
    expect(screen.getByText("en → de")).toBeVisible();
    expect(mocks.translate).toHaveBeenCalledWith("translated-request", "nuance", "auto", "de");
    expect(mocks.save).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("button", { name: "Edit capture" }));
    await fireEvent.input(screen.getByLabelText("Selected text"), { target: { value: "subtlety" } });
    await fireEvent.input(screen.getByLabelText("Context"), { target: { value: "A subtle distinction." } });
    await fireEvent.input(screen.getByLabelText("Translation (optional)"), { target: { value: "Nuance" } });
    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));

    expect(mocks.apply).toHaveBeenCalledWith("translated-request", { selectedText: "subtlety", sentence: "A subtle distinction.", translation: "Nuance" });
    expect(mocks.save).not.toHaveBeenCalled();
    expect(await screen.findByText("Nuance")).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));
    expect(mocks.save).toHaveBeenCalledWith("translated-request", false);
  });

  it("uses the top bar as a native window drag region without making the close button draggable", () => {
    const view = render(WindowsFloatingCapture, { captureBackend });
    const header = view.container.querySelector("header");
    const title = header?.querySelector("span");
    const closeButton = screen.getByRole("button", { name: "Cancel capture" });

    expect(header).toHaveAttribute("data-tauri-drag-region");
    expect(title).toHaveAttribute("data-tauri-drag-region");
    expect(closeButton).not.toHaveAttribute("data-tauri-drag-region");
  });

  it("requires explicit confirmation before an OCR candidate can enter the save flow", async () => {
    mocks.getCapabilities.mockResolvedValue({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: true, nonActivatingWindow: false });
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.error).toBeTypeOf("function"));
    await waitFor(() => expect(mocks.getCapabilities).toHaveBeenCalled());
    mocks.error?.({ requestId: "ocr-request", failure: { code: "empty_selection", message: "No selection" } });

    await fireEvent.click(await screen.findByRole("button", { name: "Start OCR" }));
    expect(mocks.startRegionOcr).toHaveBeenCalledTimes(1);
    const suggestion = { text: "serendipity", bounds: { x: 1, y: 2, width: 30, height: 12 }, confidence: 0.91 };
    mocks.ocr?.({ requestId: "ocr-request", candidates: [suggestion], ambiguous: false });
    expect(screen.queryByRole("button", { name: "Save capture" })).not.toBeInTheDocument();
    expect(mocks.translate).not.toHaveBeenCalled();

    await fireEvent.click(await screen.findByRole("button", { name: "Confirm" }));
    expect(mocks.confirmOcr).toHaveBeenCalledWith("ocr-request", "serendipity", "");
    await waitFor(() => expect(mocks.translate).toHaveBeenCalledTimes(1));
    expect(mocks.translate).toHaveBeenCalledWith("ocr-request", "serendipity", "auto", "de");
    expect(await screen.findByRole("button", { name: "Save capture" })).toBeVisible();
  });

  it("prevents duplicate OCR confirmation while the first confirmation is pending", async () => {
    let finishConfirmation: (() => void) | undefined;
    mocks.confirmOcr.mockImplementationOnce(() => new Promise<void>((resolve) => { finishConfirmation = resolve; }));
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ocr).toBeTypeOf("function"));
    mocks.error?.({ requestId: "ocr-double", failure: { code: "empty_selection", message: "No selection" } });
    mocks.ocr?.({ requestId: "ocr-double", candidates: [{ text: "candidate", bounds: { x: 1, y: 2, width: 30, height: 12 }, confidence: 0.8 }], ambiguous: false });
    const button = await screen.findByRole("button", { name: "Confirm" });

    await fireEvent.click(button);
    await fireEvent.click(button);

    expect(mocks.confirmOcr).toHaveBeenCalledTimes(1);
    finishConfirmation?.();
  });

  it("recovers from a provider-neutral translation failure by retrying the current edited text", async () => {
    mocks.getCapabilities.mockResolvedValue({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: true, nonActivatingWindow: false });
    mocks.translate.mockRejectedValueOnce({ code: "translation_failed", message: "secret provider response" });
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "retry-request", candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" } });

    expect(await screen.findByRole("alert")).toHaveTextContent("Translation is temporarily unavailable");
    expect(screen.queryByText(/secret provider response/i)).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry translation" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Edit capture" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Save capture" })).toBeEnabled();

    await fireEvent.click(screen.getByRole("button", { name: "Edit capture" }));
    await fireEvent.input(screen.getByLabelText("Selected text"), { target: { value: "subtlety" } });
    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));
    mocks.translate.mockResolvedValueOnce({ translatedText: "Feinheit", sourceLanguage: "en", targetLanguage: "de" });
    await fireEvent.click(screen.getByRole("button", { name: "Retry translation" }));

    await waitFor(() => expect(mocks.translate).toHaveBeenLastCalledWith("retry-request", "subtlety", "auto", "de"));
    expect(await screen.findByText("Feinheit")).toBeVisible();
    expect(mocks.save).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));
    expect(mocks.save).toHaveBeenCalledTimes(1);
  });

  it("marks translation stale after Vocabulary changes and retranslates only on request", async () => {
    mocks.getCapabilities.mockResolvedValue({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: true, nonActivatingWindow: false });
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "stale-translation", candidate: { selectedText: "nuance", sentence: "A nuance.", origin: "accessibility" } });
    expect(await screen.findByText("Feinheit")).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Edit capture" }));
    await fireEvent.input(screen.getByLabelText("Selected text"), { target: { value: "subtlety" } });
    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));

    expect(await screen.findByText("Vocabulary changed. The translation may no longer match.")).toBeVisible();
    expect(mocks.translate).toHaveBeenCalledTimes(1);
    await fireEvent.click(screen.getByRole("button", { name: "Translate again" }));
    await waitFor(() => expect(mocks.translate).toHaveBeenLastCalledWith("stale-translation", "subtlety", "auto", "de"));
  });

  it("recovers a Region OCR recognition failure with retry or Manual Capture", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.regionOcr).toBeTypeOf("function"));
    mocks.regionOcr?.({ requestId: "ocr-failed" });
    mocks.error?.({ requestId: "ocr-failed", failure: { code: "operation", message: "OCR did not find readable text" } });

    expect(await screen.findByRole("button", { name: "Try Again" })).toBeEnabled();
    await fireEvent.click(screen.getByRole("button", { name: "Manual Capture" }));
    expect(mocks.openManualCapture).toHaveBeenCalledTimes(1);
    expect(mocks.close).toHaveBeenCalledWith("ocr-failed");
  });

  it("cancels OCR confirmation without saving", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ocr).toBeTypeOf("function"));
    mocks.error?.({ requestId: "ocr-cancel", failure: { code: "empty_selection", message: "No selection" } });
    mocks.ocr?.({ requestId: "ocr-cancel", candidates: [{ text: "candidate", bounds: { x: 1, y: 2, width: 30, height: 12 }, confidence: 0.8 }], ambiguous: false });

    await fireEvent.click(await screen.findByRole("button", { name: "Cancel OCR" }));

    expect(mocks.close).toHaveBeenCalledWith("ocr-cancel");
    expect(mocks.save).not.toHaveBeenCalled();
  });

  it("turns multiple recognized words into an editable OCR draft", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ocr).toBeTypeOf("function"));
    const candidates = [
      { text: "architecture", bounds: { x: 10, y: 10, width: 80, height: 16 }, confidence: 0.89 },
      { text: "heterogeneous", bounds: { x: 12, y: 30, width: 95, height: 16 }, confidence: 0.87 },
    ];

    mocks.error?.({ requestId: "ocr-ambiguous", failure: { code: "unsupported_element", message: "Unsupported" } });
    mocks.ocr?.({ requestId: "ocr-ambiguous", candidates, ambiguous: true });

    expect(await screen.findByText("识别到多个词，请保留你要收集的词汇。")).toBeVisible();
    expect(screen.getByLabelText("Vocabulary")).toHaveValue("architecture heterogeneous");
    expect(screen.getByLabelText(/Context sentence/)).toHaveValue("");
    await fireEvent.input(screen.getByLabelText("Vocabulary"), { target: { value: "heterogeneous" } });
    await fireEvent.input(screen.getByLabelText(/Context sentence/), { target: { value: "A heterogeneous system." } });
    await fireEvent.click(screen.getByRole("button", { name: "Confirm" }));

    expect(mocks.confirmOcr).toHaveBeenCalledWith("ocr-ambiguous", "heterogeneous", "A heterogeneous system.");
    expect(await screen.findByRole("button", { name: "Save capture" })).toBeVisible();
  });

  it("offers OCR after capabilities resolve when the eligible failure arrived first", async () => {
    let resolveCapabilities: ((value: Awaited<ReturnType<WindowsCaptureBackend["getCapabilities"]>>) => void) | undefined;
    mocks.getCapabilities.mockImplementationOnce(() => new Promise((resolve) => { resolveCapabilities = resolve; }));
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.error).toBeTypeOf("function"));

    mocks.error?.({ requestId: "capability-race", failure: { code: "empty_selection", message: "No selection" } });
    expect(screen.queryByRole("button", { name: "Start OCR" })).not.toBeInTheDocument();
    resolveCapabilities?.({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: false, nonActivatingWindow: false });

    expect(await screen.findByRole("button", { name: "Start OCR" })).toBeVisible();
  });

  it("announces OCR progress and keeps the fallback recoverable when OCR fails", async () => {
    mocks.startRegionOcr.mockRejectedValueOnce(new Error("OCR could not start"));
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.error).toBeTypeOf("function"));
    mocks.error?.({ requestId: "ocr-retry", failure: { code: "empty_selection", message: "No selection" } });

    await fireEvent.click(await screen.findByRole("button", { name: "Start OCR" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("OCR could not start");
    expect(screen.getByRole("button", { name: "Try Again" })).toBeEnabled();
  });

  it("corrects text and context, adds an optional translation, saves once, and undoes by request", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "request-10", candidate: { selectedText: "nuanc", sentence: "A useful nuanc.", origin: "accessibility" } });

    await fireEvent.click(await screen.findByRole("button", { name: "Edit capture" }));
    await fireEvent.input(screen.getByLabelText("Selected text"), { target: { value: "nuance" } });
    await fireEvent.input(screen.getByLabelText("Context"), { target: { value: "A useful nuance." } });
    await fireEvent.input(screen.getByLabelText("Translation (optional)"), { target: { value: "Feinheit" } });
    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));
    await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));

    expect(mocks.apply).toHaveBeenCalledWith("request-10", { selectedText: "nuance", sentence: "A useful nuance.", translation: "Feinheit" });
    expect(mocks.save).toHaveBeenCalledWith("request-10", false);
    await fireEvent.click(await screen.findByRole("button", { name: "Undo" }));
    expect(mocks.undo).toHaveBeenCalledWith("request-10", "encounter-1");
  });

  it("requires an explicit save-without-translation action when translation is blank", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "request-plain", candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" } });

    const saveButton = await screen.findByRole("button", { name: "Save capture" });
    await waitFor(() => expect(saveButton).toBeEnabled());
    await fireEvent.click(saveButton);

    expect(mocks.save).toHaveBeenCalledWith("request-plain", true);
  });

  it("allows a corrected selection to save when native context is unavailable", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "no-context", candidate: { selectedText: "nuanc", sentence: "", origin: "accessibility" } });
    await fireEvent.click(await screen.findByRole("button", { name: "Edit capture" }));
    await fireEvent.input(screen.getByLabelText("Selected text"), { target: { value: "nuance" } });

    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));
    await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));

    expect(mocks.apply).toHaveBeenCalledWith("no-context", { selectedText: "nuance", sentence: "" });
    expect(mocks.save).toHaveBeenCalledWith("no-context", true);
    expect(await screen.findByText("First Encounter saved")).toBeInTheDocument();
  });

  it("ignores a save completion after a newer request arrives", async () => {
    let finishSave: ((card: Awaited<ReturnType<WindowsCaptureBackend["save"]>>) => void) | undefined;
    mocks.save.mockImplementationOnce(() => new Promise((resolve) => { finishSave = resolve; }));
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "old-request", candidate: { selectedText: "old", sentence: "Old context.", origin: "accessibility" } });
    await fireEvent.click(await screen.findByRole("button", { name: "Save capture" }));
    mocks.ready?.({ requestId: "new-request", candidate: { selectedText: "new", sentence: "New context.", origin: "accessibility" } });

    finishSave?.({ wordId: "old-word", encounterId: "old-encounter", displayForm: "old", context: "Old context.", encounterCount: 1, isExistingWord: false });

    expect(await screen.findByRole("heading", { name: "new" })).toBeInTheDocument();
    expect(screen.queryByText("Saved")).not.toBeInTheDocument();
    expect(mocks.releaseFocus).not.toHaveBeenCalled();
  });

  it("dismisses a saved result after four unpaused seconds", async () => {
    const view = render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "timed-request", candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" } });
    const saveButton = await screen.findByRole("button", { name: "Save capture" });
    await waitFor(() => expect(mocks.getCapabilities).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(view.container.querySelector("section")).toHaveAttribute("aria-busy", "false"));
    await waitFor(() => expect(saveButton).toBeEnabled());
    await fireEvent.click(saveButton);
    expect(await screen.findByText("Saved")).toBeVisible();
    await fireEvent.mouseEnter(view.container.querySelector("main")!);
    vi.useFakeTimers();
    try {
      vi.advanceTimersByTime(5_000);
      expect(mocks.hide).not.toHaveBeenCalled();
      await fireEvent.mouseLeave(view.container.querySelector("main")!);
      vi.advanceTimersByTime(4_000);
      expect(mocks.hide).toHaveBeenCalledWith("timed-request");
    } finally {
      vi.useRealTimers();
    }
  });

  it("removes main-window minimum dimensions from the capture document", () => {
    const view = render(WindowsFloatingCapture, { captureBackend });

    expect(document.body.classList.contains("windows-capture-document")).toBe(true);

    view.unmount();
    expect(document.body.classList.contains("windows-capture-document")).toBe(false);
  });

  it("keeps the saved result interactive after editing until the window is dismissed", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({
      requestId: "request-9",
      candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" },
    });

    expect(mocks.focus).not.toHaveBeenCalled();
    await fireEvent.click(await screen.findByRole("button", { name: "Edit capture" }));
    expect(mocks.focus).toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));
    await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));
    expect(mocks.releaseFocus).not.toHaveBeenCalled();

    await fireEvent.click(await screen.findByRole("button", { name: "Cancel capture" }));
    expect(mocks.close).toHaveBeenCalledWith("request-9");
  });

  it("cancels the active request through the unconditional close command", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({
      requestId: "request-9",
      candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" },
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Cancel capture" }));
    expect(mocks.close).toHaveBeenCalledWith("request-9");
  });

  it("keeps overflowing capture content vertically scrollable at the normal window width", async () => {
    const view = render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({
      requestId: "long-content",
      candidate: { selectedText: "lengthy", sentence: "A ".repeat(500), origin: "accessibility" },
    });

    expect(view.container.querySelector("section")).toHaveClass("scrollable-content");
  });

  it("cancels the active request with Escape", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({
      requestId: "request-escape",
      candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" },
    });

    await fireEvent.keyDown(window, { key: "Escape" });
    expect(mocks.close).toHaveBeenCalledWith("request-escape");
  });
});
