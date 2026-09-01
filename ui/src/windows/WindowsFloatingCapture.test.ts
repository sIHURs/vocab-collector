import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import WindowsFloatingCapture from "./WindowsFloatingCapture.svelte";
import type { CaptureReady, WindowsCaptureBackend } from "./captureBackend";

const mocks = {
  focus: vi.fn(async () => {}),
  releaseFocus: vi.fn(async () => {}),
  hide: vi.fn(async (_requestId: string) => {}),
  save: vi.fn(async () => ({ wordId: "word-1", encounterId: "encounter-1", displayForm: "nuance", context: "A useful nuance.", encounterCount: 1, isExistingWord: false })),
  undo: vi.fn(async (_requestId: string, _encounterId: string) => {}),
  ready: undefined as ((event: CaptureReady) => void) | undefined,
};

const captureBackend: WindowsCaptureBackend = {
  listenReady: async (handler) => { mocks.ready = handler; return () => { mocks.ready = undefined; }; },
  focus: mocks.focus,
  releaseFocus: mocks.releaseFocus,
  hide: mocks.hide,
  save: mocks.save,
  undo: mocks.undo,
};

describe("Windows floating capture presentation", () => {
  beforeEach(() => {
    mocks.focus.mockClear();
    mocks.releaseFocus.mockClear();
    mocks.hide.mockClear();
    mocks.save.mockClear();
    mocks.undo.mockClear();
    mocks.ready = undefined;
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

  it("focuses only after explicit editing and restores source focus after save", async () => {
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
    expect(mocks.releaseFocus).toHaveBeenCalled();
  });

  it("cancels the active request through the focus-restoring hide command", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({
      requestId: "request-9",
      candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" },
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Cancel capture" }));
    expect(mocks.hide).toHaveBeenCalledWith("request-9");
  });

  it("cancels the active request with Escape", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({
      requestId: "request-escape",
      candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" },
    });

    await fireEvent.keyDown(window, { key: "Escape" });
    expect(mocks.hide).toHaveBeenCalledWith("request-escape");
  });
});
