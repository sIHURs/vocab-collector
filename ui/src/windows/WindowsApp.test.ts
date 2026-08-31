import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import { DemoBackend, type Backend } from "../lib/backend";
import WindowsApp from "./WindowsApp.svelte";

async function saveManualCapture(word: string, sentence: string) {
  await fireEvent.click(screen.getAllByRole("button", { name: "Manual capture" })[0]);
  await fireEvent.input(screen.getByLabelText("Word or phrase"), { target: { value: word } });
  await fireEvent.input(screen.getByLabelText("Context"), { target: { value: sentence } });
  await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));
}

describe("Windows main presentation", () => {
  it("provides Windows navigation without the static Progress view", async () => {
    render(WindowsApp, { api: new DemoBackend(false) });

    expect(await screen.findByRole("heading", { level: 1, name: "Today" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Today" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Vocabulary" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Review" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Settings" })).toBeVisible();
    expect(screen.queryByText("Progress")).toBeNull();
  });

  it("saves repeat encounters, shows detail, and refreshes after Undo", async () => {
    render(WindowsApp, { api: new DemoBackend(false) });
    expect(await screen.findByText("No captures yet")).toBeVisible();

    await saveManualCapture("Lucid", "The explanation was lucid.");
    expect(await screen.findByRole("button", { name: /Lucid/ })).toBeVisible();

    await saveManualCapture("lucid", "Her second example was lucid too.");
    const saved = await screen.findByRole("dialog", { name: "Capture saved" });
    expect(within(saved).getByText("2 encounters")).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Vocabulary" }));
    expect(screen.getAllByRole("button", { name: /lucid/i })).toHaveLength(1);

    await fireEvent.click(screen.getByRole("button", { name: /lucid/i }));
    const detail = await screen.findByRole("dialog", { name: "Vocabulary detail" });
    expect(within(detail).getByText("The explanation was lucid.")).toBeVisible();
    expect(within(detail).getByText("Her second example was lucid too.")).toBeVisible();

    await fireEvent.click(within(saved).getByRole("button", { name: "Undo" }));
    expect(await within(detail).findByText(/1 encounter/)).toBeVisible();
    expect(screen.queryByText("Her second example was lucid too.")).toBeNull();
  });

  it("shows a recoverable load failure", async () => {
    const demo = new DemoBackend(false);
    const failing: Backend = {
      capture: demo.capture.bind(demo), undoCapture: demo.undoCapture.bind(demo),
      getToday: async () => { throw new Error("Database is temporarily unavailable"); },
      listWords: demo.listWords.bind(demo), getWord: demo.getWord.bind(demo),
      submitReview: demo.submitReview.bind(demo), getSettings: demo.getSettings.bind(demo),
      updateSettings: demo.updateSettings.bind(demo), replaceShortcut: demo.replaceShortcut.bind(demo),
      getPlatformCapabilities: demo.getPlatformCapabilities.bind(demo),
    };
    render(WindowsApp, { api: failing });

    expect(await screen.findByRole("alert")).toHaveTextContent("Database is temporarily unavailable");
    expect(screen.getByRole("button", { name: "Try again" })).toBeVisible();
  });

  it("keeps Manual Capture input available after a save failure", async () => {
    const demo = new DemoBackend(false);
    let attempts = 0;
    const flaky: Backend = {
      capture: async (input) => {
        if (attempts++ === 0) throw new Error("Could not save locally");
        return demo.capture(input);
      },
      undoCapture: demo.undoCapture.bind(demo), getToday: demo.getToday.bind(demo),
      listWords: demo.listWords.bind(demo), getWord: demo.getWord.bind(demo),
      submitReview: demo.submitReview.bind(demo), getSettings: demo.getSettings.bind(demo),
      updateSettings: demo.updateSettings.bind(demo), replaceShortcut: demo.replaceShortcut.bind(demo),
      getPlatformCapabilities: demo.getPlatformCapabilities.bind(demo),
    };
    render(WindowsApp, { api: flaky });
    await screen.findByText("No captures yet");

    await saveManualCapture("durable", "The local record should be durable.");
    const dialog = screen.getByRole("dialog", { name: "Manual capture" });
    expect(within(dialog).getByRole("alert")).toHaveTextContent("Could not save locally");
    expect(within(dialog).getByLabelText("Word or phrase")).toHaveValue("durable");

    await fireEvent.click(within(dialog).getByRole("button", { name: "Save capture" }));
    expect(await screen.findByRole("dialog", { name: "Capture saved" })).toBeVisible();
  });
});
