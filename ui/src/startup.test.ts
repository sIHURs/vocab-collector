import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), mount: vi.fn(), appearance: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("svelte", async (original) => ({ ...await original<typeof import("svelte")>(), mount: mocks.mount }));
vi.mock("./lib/appearance", () => ({ connectAppearance: mocks.appearance }));
vi.mock("./App.svelte", () => ({ default: {} }));
vi.mock("./FloatingCapture.svelte", () => ({ default: {} }));
vi.mock("./windows/WindowsApp.svelte", () => ({ default: {} }));
vi.mock("./windows/WindowsFloatingCapture.svelte", () => ({ default: {} }));
vi.mock("./windows/WindowsOcrOverlay.svelte", () => ({ default: {} }));

describe("desktop startup", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.useFakeTimers();
    mocks.invoke.mockReset();
    mocks.mount.mockReset();
    mocks.appearance.mockReset().mockResolvedValue(() => {});
    Object.defineProperty(globalThis, "__TAURI_INTERNALS__", { configurable: true, value: {} });
    document.body.innerHTML = '<div id="app"></div>';
  });
  afterEach(() => {
    Reflect.deleteProperty(globalThis, "__TAURI_INTERNALS__");
    vi.useRealTimers();
  });

  it("defers pages and settings reads until native setup is ready", async () => {
    let ready = false;
    mocks.invoke.mockImplementation(async (command: string) => command === "get_startup_ready" ? ready : "windows");
    await import("./main");
    expect(mocks.mount).not.toHaveBeenCalled();
    expect(mocks.appearance).not.toHaveBeenCalled();
    ready = true;
    await vi.advanceTimersByTimeAsync(100);
    expect(mocks.mount).toHaveBeenCalledTimes(1);
    expect(mocks.appearance).toHaveBeenCalledTimes(1);
  });

  it("reports a bounded startup failure without mounting business pages", async () => {
    mocks.invoke.mockResolvedValue(false);
    await import("./main");
    await vi.advanceTimersByTimeAsync(30_100);
    expect(mocks.mount).not.toHaveBeenCalled();
    expect(mocks.appearance).not.toHaveBeenCalled();
    expect(document.querySelector('[role="alert"]')).toHaveTextContent("could not finish starting");
  });

  it("keeps browser preview independent of native startup", async () => {
    Reflect.deleteProperty(globalThis, "__TAURI_INTERNALS__");
    await import("./main");
    expect(mocks.invoke).not.toHaveBeenCalled();
    expect(mocks.mount).toHaveBeenCalledTimes(1);
  });
});
