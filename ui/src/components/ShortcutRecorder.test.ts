import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import ShortcutRecorder from "./ShortcutRecorder.svelte";

describe("ShortcutRecorder", () => {
  it("records a modified key and cancels with Escape", async () => {
    const onRecorded = vi.fn();
    render(ShortcutRecorder, { value: "Alt+Space+V", onRecorded });
    await fireEvent.click(screen.getByRole("button", { name: "Record shortcut" }));
    await fireEvent.keyDown(window, { key: "w", ctrlKey: true, shiftKey: true });
    expect(onRecorded).toHaveBeenCalledWith("Control+Shift+W");
    expect(screen.getByText("⌃ ⇧ W")).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Record shortcut" }));
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.getByRole("button", { name: "Record shortcut" })).toBeVisible();
  });

  it("ignores unmodified keys", async () => {
    const onRecorded = vi.fn();
    render(ShortcutRecorder, { value: "Alt+Space+V", onRecorded });
    await fireEvent.click(screen.getByRole("button", { name: "Record shortcut" }));
    await fireEvent.keyDown(window, { key: "v" });
    expect(onRecorded).not.toHaveBeenCalled();
    expect(screen.getByText("Add a modifier such as ⌥, ⌘, or ⌃.")).toBeVisible();
  });
});
