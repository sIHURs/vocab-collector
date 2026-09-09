import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/svelte";
import { afterEach } from "vitest";

afterEach(cleanup);

// jsdom does not implement the browser's media-query API.
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (media: string): MediaQueryList => ({
    matches: false, media, onchange: null,
    addListener() {}, removeListener() {},
    addEventListener() {}, removeEventListener() {}, dispatchEvent() { return true; },
  }),
});
