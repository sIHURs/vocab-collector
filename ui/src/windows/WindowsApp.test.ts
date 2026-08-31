import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import { DemoBackend, type Backend } from "../lib/backend";
import WindowsApp from "./WindowsApp.svelte";

class TrackingReviewBackend extends DemoBackend {
  ratings: Array<{ wordId: string; rating: "forgot" | "remembered" }> = [];

  override async submitReview(wordId: string, rating: "forgot" | "remembered") {
    this.ratings.push({ wordId, rating });
    await super.submitReview(wordId, rating);
  }
}

class FlakyReviewRefreshBackend extends TrackingReviewBackend {
  reads = 0;

  override async getToday() {
    this.reads += 1;
    if (this.reads === 2) throw new Error("Could not refresh Review");
    return super.getToday();
  }
}

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

  it("runs, closes, resumes, and completes the shared due queue", async () => {
    const api = new TrackingReviewBackend(true);
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    expect(screen.getByRole("heading", { name: "serendipity" })).toBeVisible();
    expect(screen.getByText("1 of 3")).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Forgot" }));
    expect(await screen.findByRole("heading", { name: "nuance" })).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Close review" }));

    expect(screen.getByRole("heading", { level: 1, name: "Review" })).toBeVisible();
    expect(await screen.findByText("2 words remaining.")).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Resume review" }));
    expect(screen.getByRole("heading", { name: "nuance" })).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Remembered" }));
    await fireEvent.click(screen.getByRole("button", { name: "Remembered" }));
    expect(await screen.findByRole("heading", { name: "Review complete" })).toBeVisible();
    expect(api.ratings.map(({ rating }) => rating)).toEqual(["forgot", "remembered", "remembered"]);

    await fireEvent.click(screen.getByRole("button", { name: "Today" }));
    expect(await screen.findByText("Review queue is clear")).toBeVisible();
  });

  it("shows an empty Review state when nothing is due", async () => {
    render(WindowsApp, { api: new DemoBackend(false) });
    await screen.findByText("No captures yet");

    await fireEvent.click(screen.getByRole("button", { name: "Review" }));
    expect(screen.getByText("Nothing due")).toBeVisible();
  });

  it("keeps the current review card available when rating fails", async () => {
    const api = new TrackingReviewBackend(true);
    const submit = api.submitReview.bind(api);
    let attempts = 0;
    api.submitReview = async (wordId, rating) => {
      if (attempts++ === 0) throw new Error("Could not save review");
      await submit(wordId, rating);
    };
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await fireEvent.click(screen.getByRole("button", { name: "Remembered" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Could not save review");
    expect(screen.getByRole("heading", { name: "serendipity" })).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Remembered" }));
    expect(await screen.findByRole("heading", { name: "nuance" })).toBeVisible();
  });

  it("blocks stale Resume until a failed close refresh succeeds", async () => {
    const api = new FlakyReviewRefreshBackend(true);
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await fireEvent.click(screen.getByRole("button", { name: "Forgot" }));
    await fireEvent.click(screen.getByRole("button", { name: "Close review" }));

    expect(await screen.findByRole("button", { name: "Retry Review refresh" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "Resume review" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Today" }));
    await fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Start review (2)" }));
    expect(screen.getByRole("heading", { name: "nuance" })).toBeVisible();
  });

  it("does not allow Close while a rating submission is pending", async () => {
    const api = new TrackingReviewBackend(true);
    let release: (() => void) | undefined;
    api.submitReview = () => new Promise<void>((resolve) => { release = resolve; });
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await fireEvent.click(screen.getByRole("button", { name: "Remembered" }));
    expect(screen.getByRole("button", { name: "Close review" })).toBeDisabled();

    release?.();
    expect(await screen.findByRole("heading", { name: "nuance" })).toBeVisible();
  });
});
