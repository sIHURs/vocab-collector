import { describe, expect, it } from "vitest";
import { selectPresentation } from "./presentation";

describe("desktop presentation selection", () => {
  it("selects Windows-owned roots only for the Windows desktop family", () => {
    expect(selectPresentation("main", "windows")).toBe("windows-main");
    expect(selectPresentation("capture", "windows")).toBe("windows-capture");
    expect(selectPresentation("main", "shared")).toBe("shared-main");
    expect(selectPresentation("capture", "shared")).toBe("shared-capture");
  });
});
