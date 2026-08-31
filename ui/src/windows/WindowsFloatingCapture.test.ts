import { render } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import WindowsFloatingCapture from "./WindowsFloatingCapture.svelte";

describe("Windows floating capture presentation", () => {
  it("removes main-window minimum dimensions from the capture document", () => {
    const view = render(WindowsFloatingCapture);

    expect(document.body.classList.contains("windows-capture-document")).toBe(true);

    view.unmount();
    expect(document.body.classList.contains("windows-capture-document")).toBe(false);
  });
});
