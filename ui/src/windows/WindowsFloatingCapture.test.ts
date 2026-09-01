import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import WindowsFloatingCapture from "./WindowsFloatingCapture.svelte";
import type { CaptureReady, WindowsCaptureBackend } from "./captureBackend";

const mocks = {
  focus: vi.fn(async () => {}),
  releaseFocus: vi.fn(async () => {}),
  hide: vi.fn(async (_requestId: string) => {}),
  ready: undefined as ((event: CaptureReady) => void) | undefined,
};

const captureBackend: WindowsCaptureBackend = {
  listenReady: async (handler) => { mocks.ready = handler; return () => { mocks.ready = undefined; }; },
  focus: mocks.focus,
  releaseFocus: mocks.releaseFocus,
  hide: mocks.hide,
};

describe("Windows floating capture presentation", () => {
  beforeEach(() => {
    mocks.focus.mockClear();
    mocks.releaseFocus.mockClear();
    mocks.hide.mockClear();
    mocks.ready = undefined;
  });

  it("removes main-window minimum dimensions from the capture document", () => {
    const view = render(WindowsFloatingCapture, { captureBackend });

    expect(document.body.classList.contains("windows-capture-document")).toBe(true);

    view.unmount();
    expect(document.body.classList.contains("windows-capture-document")).toBe(false);
  });

  it("focuses only after explicit editing and restores source focus on done", async () => {
    render(WindowsFloatingCapture, { captureBackend });
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({
      requestId: "request-9",
      candidate: { selectedText: "nuance", sentence: "A useful nuance.", origin: "accessibility" },
    });

    expect(mocks.focus).not.toHaveBeenCalled();
    await fireEvent.click(await screen.findByRole("button", { name: "Edit capture" }));
    expect(mocks.focus).toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("button", { name: "Done editing" }));
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
