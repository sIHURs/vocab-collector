import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import WindowsOcrOverlay from "./WindowsOcrOverlay.svelte";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(async () => {}),
  ready: undefined as undefined | ((event: { payload: { requestId: string } }) => void),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_event: string, handler: typeof mocks.ready) => {
    mocks.ready = handler;
    return () => { mocks.ready = undefined; };
  }),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    outerPosition: async () => ({ x: 200, y: 100 }),
    scaleFactor: async () => 2,
  }),
}));

describe("Region OCR overlay", () => {
  beforeEach(() => {
    mocks.invoke.mockClear();
    mocks.ready = undefined;
  });

  it("turns a user drag into one logical desktop region", async () => {
    const { container } = render(WindowsOcrOverlay);
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ payload: { requestId: "ocr-region" } });
    const overlay = screen.getByRole("button", { name: "Region OCR selection" });

    await fireEvent.mouseDown(overlay, { button: 0, clientX: 20, clientY: 30 });
    await fireEvent.mouseMove(overlay, { clientX: 120, clientY: 70 });
    await waitFor(() => expect(container.querySelector(".selection")).not.toBeNull());
    await fireEvent.mouseUp(overlay);

    expect(mocks.invoke).toHaveBeenCalledWith("capture_ocr_region", {
      requestId: "ocr-region",
      region: { x: 120, y: 80, width: 100, height: 40 },
    });
  });

  it("ignores tiny regions and lets Escape cancel the request", async () => {
    render(WindowsOcrOverlay);
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ payload: { requestId: "cancel-region" } });
    const overlay = screen.getByRole("button", { name: "Region OCR selection" });
    await fireEvent.mouseDown(overlay, { button: 0, clientX: 10, clientY: 10 });
    await fireEvent.mouseMove(overlay, { clientX: 12, clientY: 12 });
    await fireEvent.mouseUp(overlay);
    expect(mocks.invoke).not.toHaveBeenCalled();

    await fireEvent.keyDown(window, { key: "Escape" });
    expect(mocks.invoke).toHaveBeenCalledWith("cancel_region_ocr_capture", { requestId: "cancel-region" });
  });
  it("normalizes a reverse drag and hides instructions while selecting", async () => {
    render(WindowsOcrOverlay);
    await waitFor(() => expect(mocks.ready).toBeTypeOf("function"));
    mocks.ready?.({ payload: { requestId: "reverse" } });
    const overlay = screen.getByRole("button", { name: "Region OCR selection" });
    expect(screen.getByRole("status")).toBeVisible();
    await fireEvent.mouseDown(overlay, { button: 0, clientX: 120, clientY: 70 });
    expect(screen.queryByRole("status")).toBeNull();
    await fireEvent.mouseMove(overlay, { clientX: 20, clientY: 30 });
    await fireEvent.mouseUp(overlay);
    expect(mocks.invoke).toHaveBeenCalledWith("capture_ocr_region", { requestId: "reverse", region: { x: 120, y: 80, width: 100, height: 40 } });
  });
});
