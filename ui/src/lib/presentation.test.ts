import { describe, expect, it } from "vitest";
import "../styles.css";
import { markDocumentWindow, selectPresentation } from "./presentation";

describe("desktop presentation selection", () => {
  it("selects Windows-owned roots only for the Windows desktop family", () => {
    expect(selectPresentation("main", "windows")).toBe("windows-main");
    expect(selectPresentation("capture", "windows")).toBe("windows-capture");
    expect(selectPresentation("main", "shared")).toBe("shared-main");
    expect(selectPresentation("capture", "shared")).toBe("shared-capture");
  });

  it("marks the OCR document so the production-global background stays transparent", () => {
    markDocumentWindow(document.documentElement, "ocr-overlay");

    expect(document.documentElement.dataset.window).toBe("ocr-overlay");
    expect(getComputedStyle(document.documentElement).backgroundColor).toBe("rgba(0, 0, 0, 0)");
    expect(getComputedStyle(document.body).backgroundColor).toBe("rgba(0, 0, 0, 0)");
  });
});
