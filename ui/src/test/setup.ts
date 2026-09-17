import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/svelte";
import { afterEach, vi } from "vitest";
import { tick } from "svelte";

afterEach(async () => {
  cleanup();
  await tick();
  // Bits UI releases the last dialog scroll lock after a 24 ms grace period.
  // Let that cleanup finish while jsdom's document still exists.
  if (vi.isFakeTimers()) await vi.runOnlyPendingTimersAsync();
  else await new Promise((resolve) => setTimeout(resolve, 30));
});

// jsdom does not implement the browser's media-query API.
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (media: string): MediaQueryList => ({
    matches: false, media, onchange: null,
    addListener() {}, removeListener() {},
    addEventListener() {}, removeEventListener() {}, dispatchEvent() { return true; },
  }),
});

// jsdom has no layout observer; component resize callbacks are browser-verified.
globalThis.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
