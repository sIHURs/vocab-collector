import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import WindowsFloatingCapture from "./WindowsFloatingCapture.svelte";
import type { CaptureReady, OcrCandidatesReady, WindowsCaptureBackend } from "./captureBackend";

const mocks = {
  focus: vi.fn(async () => {}),
  releaseFocus: vi.fn(async () => {}),
  close: vi.fn(async (_requestId: string) => {}),
  hide: vi.fn(async (_requestId: string) => {}),
  save: vi.fn(async () => ({ wordId: "word-1", encounterId: "encounter-1", displayForm: "nuance", context: "A useful nuance.", encounterCount: 1, isExistingWord: false })),
  undo: vi.fn(async (_requestId: string, _encounterId: string) => {}),
  startOcr: vi.fn(async (_requestId: string) => {}),
  confirmOcr: vi.fn(async (_requestId: string, _candidateIndex: number) => {}),
  getCapabilities: vi.fn(async () => ({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: false, nonActivatingWindow: false })),
  ready: undefined as ((event: CaptureReady) => void) | undefined,
  error: undefined as ((event: { requestId: string; failure: { code: "empty_selection" | "unsupported_element" | "operation"; message: string } }) => void) | undefined,
  ocr: undefined as ((event: OcrCandidatesReady) => void) | undefined,
};

const captureBackend: WindowsCaptureBackend = {
  listenReady: async (handler) => { mocks.ready = handler; return () => { mocks.ready = undefined; }; },
  focus: mocks.focus,
  releaseFocus: mocks.releaseFocus,
  close: mocks.close,
  hide: mocks.hide,
  save: mocks.save,
  undo: mocks.undo,
  startOcr: mocks.startOcr,
  confirmOcr: mocks.confirmOcr,
  getCapabilities: mocks.getCapabilities,
  listenError: async (handler) => { mocks.error = handler; return () => { mocks.error = undefined; }; },
  listenOcrCandidate: async (handler) => { mocks.ocr = handler; return () => { mocks.ocr = undefined; }; },
};

describe("Windows floating capture presentation", () => {
  beforeEach(() => {
    mocks.focus.mockClear();
    mocks.releaseFocus.mockClear();
    mocks.close.mockClear();
    mocks.hide.mockClear();
    mocks.save.mockClear();
    mocks.undo.mockClear();
    mocks.startOcr.mockClear();
    mocks.confirmOcr.mockClear();
    mocks.getCapabilities.mockClear();
    mocks.ready = undefined;
    mocks.error = undefined;
    mocks.ocr = undefined;
  });

  it("requires explicit confirmation before an OCR candidate can enter the save flow", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.error).toBeTypeOf("function"));
    await waitFor(() => expect(mocks.getCapabilities).toHaveBeenCalled());
    mocks.error?.({ requestId: "ocr-request", failure: { code: "empty_selection", message: "No selection" } });

    await fireEvent.click(await screen.findByRole("button", { name: "Use OCR near pointer" }));
    expect(mocks.startOcr).toHaveBeenCalledWith("ocr-request");
    const suggestion = { text: "serendipity", bounds: { x: 1, y: 2, width: 30, height: 12 }, confidence: 0.91 };
    mocks.ocr?.({ requestId: "ocr-request", candidates: [suggestion], ambiguous: false });
    expect(screen.queryByRole("button", { name: "Save without translation" })).not.toBeInTheDocument();

    await fireEvent.click(await screen.findByRole("button", { name: "Confirm OCR candidate" }));
    expect(mocks.confirmOcr).toHaveBeenCalledWith("ocr-request", 0);
    expect(await screen.findByRole("button", { name: "Save without translation" })).toBeVisible();
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

  it("offers close OCR candidates as a keyboard-selectable accessible list", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ocr).toBeTypeOf("function"));
    const candidates = [
      { text: "architecture", bounds: { x: 10, y: 10, width: 80, height: 16 }, confidence: 0.89 },
      { text: "heterogeneous", bounds: { x: 12, y: 30, width: 95, height: 16 }, confidence: 0.87 },
    ];

    mocks.error?.({ requestId: "ocr-ambiguous", failure: { code: "unsupported_element", message: "Unsupported" } });
    mocks.ocr?.({ requestId: "ocr-ambiguous", candidates, ambiguous: true });

    const list = await screen.findByRole("listbox", { name: "OCR candidates" });
    expect(list).toBeVisible();
    expect(list).toHaveAttribute("aria-activedescendant", "ocr-candidate-0");
    expect(screen.getByRole("option", { name: "architecture" })).toHaveAttribute("aria-selected", "true");
    await fireEvent.keyDown(list, { key: "ArrowDown" });
    expect(screen.getByRole("option", { name: "heterogeneous" })).toHaveAttribute("aria-selected", "true");
    expect(list).toHaveAttribute("aria-activedescendant", "ocr-candidate-1");
    await fireEvent.keyDown(list, { key: "Enter" });

    expect(mocks.confirmOcr).toHaveBeenCalledWith("ocr-ambiguous", 1);
    expect(await screen.findByRole("button", { name: "Save without translation" })).toBeVisible();
  });

  it("offers OCR after capabilities resolve when the eligible failure arrived first", async () => {
    let resolveCapabilities: ((value: Awaited<ReturnType<WindowsCaptureBackend["getCapabilities"]>>) => void) | undefined;
    mocks.getCapabilities.mockImplementationOnce(() => new Promise((resolve) => { resolveCapabilities = resolve; }));
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.error).toBeTypeOf("function"));

    mocks.error?.({ requestId: "capability-race", failure: { code: "empty_selection", message: "No selection" } });
    expect(screen.queryByRole("button", { name: "Use OCR near pointer" })).not.toBeInTheDocument();
    resolveCapabilities?.({ selectionCapture: false, selectionBounds: false, screenshotOcr: true, translation: false, nonActivatingWindow: false });

    expect(await screen.findByRole("button", { name: "Use OCR near pointer" })).toBeVisible();
  });

  it("announces OCR progress and keeps the fallback recoverable when OCR fails", async () => {
    mocks.startOcr.mockRejectedValueOnce(new Error("OCR could not start"));
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.error).toBeTypeOf("function"));
    mocks.error?.({ requestId: "ocr-retry", failure: { code: "empty_selection", message: "No selection" } });

    await fireEvent.click(await screen.findByRole("button", { name: "Use OCR near pointer" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("OCR could not start");
    expect(screen.getByRole("button", { name: "Retry OCR near pointer" })).toBeEnabled();
  });

  it("corrects text and context, adds an optional translation, saves once, and undoes by request", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "request-10", candidate: { selectedText: "nuanc", sentence: "A useful nuanc.", origin: "accessibility" } });

    await fireEvent.click(await screen.findByRole("button", { name: "Edit capture" }));
    await fireEvent.input(screen.getByLabelText("Selected text"), { target: { value: "nuance" } });
    await fireEvent.input(screen.getByLabelText("Context"), { target: { value: "A useful nuance." } });
    await fireEvent.input(screen.getByLabelText("Translation (optional)"), { target: { value: "Feinheit" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));

    expect(mocks.save).toHaveBeenCalledWith("request-10", { selectedText: "nuance", sentence: "A useful nuance.", translation: "Feinheit" }, false);
    await fireEvent.click(await screen.findByRole("button", { name: "Undo" }));
    expect(mocks.undo).toHaveBeenCalledWith("request-10", "encounter-1");
  });

  it("requires an explicit save-without-translation action when translation is blank", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "request-plain", candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" } });

    await fireEvent.click(await screen.findByRole("button", { name: "Save without translation" }));

    expect(mocks.save).toHaveBeenCalledWith("request-plain", { selectedText: "nuance", sentence: "A useful nuance." }, true);
  });

  it("allows a corrected selection to save when native context is unavailable", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "no-context", candidate: { selectedText: "nuanc", sentence: "", origin: "accessibility" } });
    await fireEvent.click(await screen.findByRole("button", { name: "Edit capture" }));
    await fireEvent.input(screen.getByLabelText("Selected text"), { target: { value: "nuance" } });

    await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));

    expect(mocks.save).toHaveBeenCalledWith("no-context", { selectedText: "nuance", sentence: "" }, true);
    expect(await screen.findByText("First Encounter saved")).toBeInTheDocument();
  });

  it("ignores a save completion after a newer request arrives", async () => {
    let finishSave: ((card: Awaited<ReturnType<WindowsCaptureBackend["save"]>>) => void) | undefined;
    mocks.save.mockImplementationOnce(() => new Promise((resolve) => { finishSave = resolve; }));
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ requestId: "old-request", candidate: { selectedText: "old", sentence: "Old context.", origin: "accessibility" } });
    await fireEvent.click(await screen.findByRole("button", { name: "Save without translation" }));
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
    vi.useFakeTimers();
    try {
      await fireEvent.click(await screen.findByRole("button", { name: "Save without translation" }));
      vi.advanceTimersByTime(3_000);
      await fireEvent.mouseEnter(view.container.querySelector("main")!);
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
