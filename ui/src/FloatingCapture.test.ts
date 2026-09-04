import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import FloatingCapture from "./FloatingCapture.svelte";

const mocks = vi.hoisted(() => ({
  handlers: new Map<string, (event: { payload: unknown }) => void>(),
  invoke: vi.fn(),
  getPlatformCapabilities: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    mocks.handlers.set(name, handler);
    return () => {
      if (mocks.handlers.get(name) === handler) mocks.handlers.delete(name);
    };
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("./lib/backend", () => ({
  backend: { getPlatformCapabilities: mocks.getPlatformCapabilities },
}));

const capabilities = (screenshotOcr: boolean) => ({
  selectionCapture: true,
  selectionBounds: true,
  screenshotOcr,
  translation: true,
  nonActivatingWindow: true,
});
const candidate = (selectedText: string) => ({
  selectedText,
  sentence: `Context for ${selectedText}.`,
  sourceApp: "Safari",
  origin: "accessibility",
});

const savedCard = (displayForm: string) => ({
  wordId: `word-${displayForm}`,
  encounterId: `encounter-${displayForm}`,
  displayForm,
  translation: `${displayForm}-translation`,
  context: `Context for ${displayForm}.`,
  encounterCount: 1,
  isExistingWord: false,
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("floating capture request freshness", () => {
  beforeEach(() => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(false));
  });

  afterEach(() => {
    mocks.handlers.clear();
    mocks.invoke.mockReset();
    mocks.getPlatformCapabilities.mockReset();
    vi.restoreAllMocks();
  });

  it("uses the top bar as a native window drag region without making the close button draggable", () => {
    const view = render(FloatingCapture);
    const header = view.container.querySelector("header");
    const title = header?.querySelector("span");
    const closeButton = screen.getByRole("button", { name: "Close capture" });

    expect(header).toHaveAttribute("data-tauri-drag-region");
    expect(title).toHaveAttribute("data-tauri-drag-region");
    expect(closeButton).not.toHaveAttribute("data-tauri-drag-region");
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

  it("does not start translation after stale settings complete", async () => {
    const firstSettings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    const secondSettings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    let settingsCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") {
        settingsCalls += 1;
        return settingsCalls === 1 ? firstSettings.promise : secondSettings.promise;
      }
      return Promise.resolve();
    });

    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-a", candidate: candidate("old") },
    });
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-b", candidate: candidate("current") },
    });
    firstSettings.resolve({ sourceLanguage: "en", targetLanguage: "de" });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mocks.invoke).not.toHaveBeenCalledWith("translate_text", expect.objectContaining({
      requestId: "request-a",
    }));
    expect(screen.getByText("current")).toBeVisible();
  });

  it("does not start translation when settings complete after unmount", async () => {
    const settings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") return settings.promise;
      return Promise.resolve();
    });

    const { unmount } = render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-a", candidate: candidate("old") },
    });
    unmount();
    settings.resolve({ sourceLanguage: "en", targetLanguage: "de" });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mocks.invoke.mock.calls.filter(([command]) => command === "translate_text")).toHaveLength(0);
  });

  it("does not apply a completed save from a stale request or schedule its dismissal", async () => {
    let resolveFirstSave!: (card: ReturnType<typeof savedCard>) => void;
    const firstSave = new Promise<ReturnType<typeof savedCard>>((resolve) => {
      resolveFirstSave = resolve;
    });
    const secondSave = new Promise<ReturnType<typeof savedCard>>(() => {});
    const timerSpy = vi.spyOn(globalThis, "setTimeout");
    mocks.invoke.mockImplementation((command: string, args?: { requestId?: string }) => {
      if (command === "get_settings") {
        return Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" });
      }
      if (command === "translate_text") return Promise.resolve({ translatedText: "translated" });
      if (command === "save_native_capture" && args?.requestId === "request-a") return firstSave;
      if (command === "save_native_capture" && args?.requestId === "request-b") return secondSave;
      return Promise.resolve();
    });

    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-a", candidate: candidate("old") },
    });
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "save_native_capture",
      { requestId: "request-a", withoutTranslation: false },
    ));

    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-b", candidate: candidate("current") },
    });
    expect(await screen.findByText("current")).toBeVisible();
    resolveFirstSave(savedCard("old"));
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(screen.queryByText("old-translation")).not.toBeInTheDocument();
    expect(screen.getByText("current")).toBeVisible();
    expect(timerSpy.mock.calls.some(([, delay]) => delay === 4_000)).toBe(false);
  });

  it("does not schedule dismissal or hide work when save completes after unmount", async () => {
    let resolveSave!: (card: ReturnType<typeof savedCard>) => void;
    const pendingSave = new Promise<ReturnType<typeof savedCard>>((resolve) => {
      resolveSave = resolve;
    });
    const timerSpy = vi.spyOn(globalThis, "setTimeout");
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") {
        return Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" });
      }
      if (command === "translate_text") return Promise.resolve({ translatedText: "translated" });
      if (command === "save_native_capture") return pendingSave;
      return Promise.resolve();
    });

    const { unmount } = render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-a", candidate: candidate("old") },
    });
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "save_native_capture",
      { requestId: "request-a", withoutTranslation: false },
    ));
    unmount();

    resolveSave(savedCard("old"));
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(timerSpy.mock.calls.some(([, delay]) => delay === 4_000)).toBe(false);
    expect(mocks.invoke).not.toHaveBeenCalledWith("hide_capture_window");
  });

  it("does not continue OCR after an old permission request completes", async () => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(true));
    const permission = deferred<string>();
    const currentSettings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "request_screen_recording_permission") return permission.promise;
      if (command === "get_settings") return currentSettings.promise;
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));
    await waitFor(() => expect(mocks.getPlatformCapabilities).toHaveBeenCalled());
    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "request-a", code: "empty_selection", message: "Nothing selected",
    } });
    await fireEvent.click(await screen.findByRole("button", { name: "Start Region OCR" }));

    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-b", candidate: candidate("current") },
    });
    permission.resolve("granted");
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mocks.invoke).not.toHaveBeenCalledWith("start_region_ocr_capture", expect.anything());
    expect(screen.getByText("current")).toBeVisible();
  });

  it("does not continue OCR when permission completes after unmount", async () => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(true));
    const permission = deferred<string>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "request_screen_recording_permission") return permission.promise;
      return Promise.resolve();
    });
    const { unmount } = render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));
    await waitFor(() => expect(mocks.getPlatformCapabilities).toHaveBeenCalled());
    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "request-a", code: "empty_selection", message: "Nothing selected",
    } });
    await fireEvent.click(await screen.findByRole("button", { name: "Start Region OCR" }));
    unmount();
    permission.resolve("granted");
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mocks.invoke.mock.calls.filter(([command]) => command === "start_region_ocr_capture")).toHaveLength(0);
  });

  it("does not publish a late OCR failure over a newer capture", async () => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(true));
    const ocr = deferred<void>();
    const currentSettings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "request_screen_recording_permission") return Promise.resolve("granted");
      if (command === "start_region_ocr_capture") return ocr.promise;
      if (command === "get_settings") return currentSettings.promise;
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));
    await waitFor(() => expect(mocks.getPlatformCapabilities).toHaveBeenCalled());
    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "request-a", code: "empty_selection", message: "Nothing selected",
    } });
    await fireEvent.click(await screen.findByRole("button", { name: "Start Region OCR" }));
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("start_region_ocr_capture"));

    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-b", candidate: candidate("current") },
    });
    ocr.reject({ code: "operation", message: "late OCR failure" });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(screen.queryByText("late OCR failure")).not.toBeInTheDocument();
    expect(screen.getByText("current")).toBeVisible();
  });

  it("does not mutate a newer capture after an old accessibility request completes", async () => {
    const permission = deferred<void>();
    const currentSettings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "request_accessibility_permission") return permission.promise;
      if (command === "get_settings") return currentSettings.promise;
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));
    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "request-a", code: "permission_required", message: "Permission needed",
    } });
    await fireEvent.click(await screen.findByRole("button", { name: "Allow Accessibility" }));

    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-b", candidate: candidate("current") },
    });
    permission.resolve();
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(screen.queryByText(/Permission requested/)).not.toBeInTheDocument();
    expect(screen.getByText("current")).toBeVisible();
  });

  it("does not hide a newer capture when an old undo completes", async () => {
    const undo = deferred<void>();
    const currentSettings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    let settingsCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") {
        settingsCalls += 1;
        return settingsCalls === 1
          ? Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" })
          : currentSettings.promise;
      }
      if (command === "translate_text") return Promise.resolve({ translatedText: "translated" });
      if (command === "save_native_capture") return Promise.resolve(savedCard("old"));
      if (command === "undo_capture") return undo.promise;
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-a", candidate: candidate("old") },
    });
    await fireEvent.click(await screen.findByRole("button", { name: "Undo" }));

    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-b", candidate: candidate("current") },
    });
    undo.resolve();
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mocks.invoke.mock.calls.filter(([command]) => command === "hide_capture_window")).toHaveLength(0);
    expect(screen.getByText("current")).toBeVisible();
  });

  it("binds close and dismissal callbacks to the capture request", async () => {
    const timerSpy = vi.spyOn(globalThis, "setTimeout");
    const currentSettings = deferred<{ sourceLanguage: string; targetLanguage: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") return Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" });
      if (command === "translate_text") return Promise.resolve({ translatedText: "translated" });
      if (command === "save_native_capture") return Promise.resolve(savedCard("old"));
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-a", candidate: candidate("old") },
    });
    await screen.findByRole("button", { name: "Undo" });
    const oldDismiss = timerSpy.mock.calls.find(([, delay]) => delay === 4_000)?.[0];
    expect(oldDismiss).toBeTypeOf("function");

    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") return currentSettings.promise;
      return Promise.resolve();
    });
    mocks.handlers.get("capture-ready")?.({
      payload: { requestId: "request-b", candidate: candidate("current") },
    });
    (oldDismiss as () => void)();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "hide_capture_window")).toHaveLength(0);

    await fireEvent.click(screen.getByRole("button", { name: "Close capture" }));
    expect(mocks.invoke).toHaveBeenCalledWith(
      "hide_capture_window", { requestId: "request-b" },
    );
  });

  it("shows the permission action from a permission_required code despite misleading diagnostics", async () => {
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));

    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "permission-request",
      code: "permission_required",
      message: "noSelection translationUnavailable",
    } });

    expect(await screen.findByRole("button", { name: "Allow Accessibility" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "Start Region OCR" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Save without translation" })).not.toBeInTheDocument();
  });

  it("does not offer OCR for empty_selection while screenshot OCR is unavailable", async () => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(false));
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));

    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "empty-request",
      code: "empty_selection",
      message: "noSelection",
    } });

    expect(await screen.findByText("noSelection")).toBeVisible();
    expect(screen.queryByRole("button", { name: "Start Region OCR" })).not.toBeInTheDocument();
  });

  it("offers OCR for empty_selection when the reported capability allows it", async () => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(true));
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));
    await waitFor(() => expect(mocks.getPlatformCapabilities).toHaveBeenCalled());

    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "empty-request",
      code: "empty_selection",
      message: "Nothing selected",
    } });

    expect(await screen.findByRole("button", { name: "Start Region OCR" })).toBeVisible();
  });

  it("keeps the screen-recording permission action typed by the OCR command path", async () => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(true));
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "request_screen_recording_permission") return Promise.resolve("denied");
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));
    await waitFor(() => expect(mocks.getPlatformCapabilities).toHaveBeenCalled());
    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "screen-permission-request",
      code: "empty_selection",
      message: "Nothing selected",
    } });

    await fireEvent.click(await screen.findByRole("button", { name: "Start Region OCR" }));

    expect(await screen.findByRole("button", { name: "Allow Screen Recording" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "Allow Accessibility" })).not.toBeInTheDocument();
  });

  it("offers save without translation and retry for translation_unavailable", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") return Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" });
      if (command === "translate_text") return Promise.reject({
        code: "translation_unavailable",
        message: "No provider for this language pair",
      });
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));

    mocks.handlers.get("capture-ready")?.({ payload: {
      requestId: "translation-request",
      candidate: candidate("portable"),
    } });

    expect(await screen.findByRole("button", { name: "Save without translation" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Retry translation" })).toBeVisible();
    expect(screen.getByText("No provider for this language pair")).toBeVisible();
  });

  it("offers typed translation recovery for translation_failed without reading diagnostics", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") {
        return Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" });
      }
      if (command === "translate_text") {
        return Promise.reject({
          code: "translation_failed",
          message: "translationUnavailable no provider for this language pair",
        });
      }
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));

    mocks.handlers.get("capture-ready")?.({ payload: {
      requestId: "operation-translation-request",
      candidate: candidate("portable"),
    } });

    expect(await screen.findByText(/translationUnavailable/)).toBeVisible();
    expect(screen.getByRole("button", { name: "Save without translation" })).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Retry translation" }));
    await waitFor(() => expect(mocks.invoke.mock.calls.filter(
      ([command]) => command === "translate_text",
    )).toHaveLength(2));
  });

  it("does not offer translation recovery for generic operation diagnostics", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_settings") {
        return Promise.resolve({ sourceLanguage: "en", targetLanguage: "de" });
      }
      if (command === "translate_text") {
        return Promise.reject({
          code: "operation",
          message: "translationFailed translationUnavailable retry and save without translation",
        });
      }
      return Promise.resolve();
    });
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-ready")).toBe(true));

    mocks.handlers.get("capture-ready")?.({ payload: {
      requestId: "operation-translation-request",
      candidate: candidate("portable"),
    } });

    expect(await screen.findByText(/translationFailed/)).toBeVisible();
    expect(screen.queryByRole("button", { name: "Save without translation" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Retry translation" })).not.toBeInTheDocument();
  });

  it("never derives actions from arbitrary operation diagnostics", async () => {
    mocks.getPlatformCapabilities.mockResolvedValue(capabilities(true));
    render(FloatingCapture);
    await waitFor(() => expect(mocks.handlers.has("capture-error")).toBe(true));

    mocks.handlers.get("capture-error")?.({ payload: {
      requestId: "operation-request",
      code: "operation",
      message: "accessibilityPermissionRequired noSelection screenRecordingPermissionRequired translationUnavailable",
    } });

    expect(await screen.findByText(/accessibilityPermissionRequired/)).toBeVisible();
    expect(screen.queryByRole("button", { name: "Allow Accessibility" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Start Region OCR" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Save without translation" })).not.toBeInTheDocument();
  });
});

