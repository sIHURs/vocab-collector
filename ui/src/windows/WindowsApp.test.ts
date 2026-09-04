import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import { DemoBackend, type Backend } from "../lib/backend";
import type { Settings } from "../lib/types";
import WindowsApp from "./WindowsApp.svelte";

class TrackingReviewBackend extends DemoBackend {
  ratings: Array<{ wordId: string; rating: "forgot" | "remembered"; submissionId?: string }> = [];

  override async submitReview(wordId: string, rating: "forgot" | "remembered", submissionId?: string) {
    this.ratings.push({ wordId, rating, submissionId });
    return super.submitReview(wordId, rating, submissionId);
  }
}

class PendingLoadBackend extends DemoBackend {
  release: (() => void) | undefined;

  override async getToday() {
    await new Promise<void>((resolve) => { this.release = resolve; });
    return super.getToday();
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

class TrackingSettingsBackend extends DemoBackend {
  updates: Settings[] = [];

  override async updateSettings(settings: Settings) {
    this.updates.push({ ...settings });
    await super.updateSettings(settings);
  }
}

class FlakySettingsRefreshBackend extends TrackingSettingsBackend {
  todayReads = 0;

  override async getToday() {
    this.todayReads += 1;
    if (this.todayReads === 2) throw new Error("Could not refresh Today");
    return super.getToday();
  }
}

class PartiallyFailingSystemSettingsBackend extends TrackingSettingsBackend {
  override async applyWindowsSettings(settings: Settings) {
    const current = await this.getSettings();
    const persisted = {
      ...settings,
      selectionCaptureShortcut: current.selectionCaptureShortcut,
      regionOcrCaptureShortcut: current.regionOcrCaptureShortcut,
      launchAtLogin: current.launchAtLogin,
      reviewTime: current.reviewTime,
    };
    await this.updateSettings(persisted);
    return {
      settings: persisted,
      selectionShortcutError: "Shortcut unavailable",
      autostartError: "Startup registration failed",
      notificationError: "Notification schedule failed",
    };
  }
}

class ExternalCaptureBackend extends DemoBackend {
  libraryChanged: (() => void) | undefined;

  async listenLibraryChanged(handler: () => void) {
    this.libraryChanged = handler;
    return () => { this.libraryChanged = undefined; };
  }
}

async function saveManualCapture(word: string, sentence: string) {
  await fireEvent.click(screen.getAllByRole("button", { name: "Manual capture" })[0]);
  await fireEvent.input(screen.getByLabelText("Word or phrase"), { target: { value: word } });
  await fireEvent.input(screen.getByLabelText("Context"), { target: { value: sentence } });
  await fireEvent.click(screen.getByRole("button", { name: "Save capture" }));
}

async function revealAndRate(rating: "Forgot" | "Remembered") {
  await fireEvent.click(screen.getByRole("button", { name: "Show answer" }));
  await fireEvent.click(screen.getByRole("button", { name: rating }));
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

  it("moves focus into the Manual Capture dialog and returns it when dismissed", async () => {
    render(WindowsApp, { api: new DemoBackend(false) });
    const trigger = (await screen.findAllByRole("button", { name: "Manual capture" }))[0];

    await fireEvent.click(trigger);
    expect(screen.getByLabelText("Word or phrase")).toHaveFocus();

    await fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Manual capture" })).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it("keeps keyboard focus inside Manual Capture", async () => {
    render(WindowsApp, { api: new DemoBackend(false) });
    await fireEvent.click((await screen.findAllByRole("button", { name: "Manual capture" }))[0]);
    const close = screen.getByRole("button", { name: "Close manual capture" });
    close.focus();

    await fireEvent.keyDown(close, { key: "Tab", shiftKey: true });

    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus();
  });

  it("labels changing page content and loading state for assistive technology", async () => {
    const api = new PendingLoadBackend(false);
    render(WindowsApp, { api });

    expect(await screen.findByRole("status")).toHaveTextContent("Loading your vocabulary");
    api.release?.();
    await screen.findByText("No captures yet");
    const region = screen.getByRole("region", { name: "Today" });
    expect(region).toHaveAttribute("aria-labelledby", "windows-page-title");
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
    const detail = await screen.findByRole("dialog", { name: "Lucid" });
    expect(within(detail).getByText("The explanation was lucid.")).toBeVisible();
    expect(within(detail).getByText("Her second example was lucid too.")).toBeVisible();

    await fireEvent.click(within(saved).getByRole("button", { name: "Undo" }));
    expect(await within(detail).findByText(/1 encounter/)).toBeVisible();
    expect(screen.queryByText("Her second example was lucid too.")).toBeNull();
  });

  it("refreshes Today and Vocabulary after a capture is saved in another window", async () => {
    const api = new ExternalCaptureBackend(false);
    render(WindowsApp, { api });
    await screen.findByText("No captures yet");

    await api.capture({ selectedText: "ambient", sentence: "The change was immediately ambient." });
    api.libraryChanged?.();

    expect(await screen.findByRole("button", { name: /ambient/i })).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Vocabulary" }));
    expect(screen.getByRole("button", { name: /ambient/i })).toBeVisible();
  });

  it("shows a recoverable load failure", async () => {
    const demo = new DemoBackend(false);
    const failing: Backend = {
      capture: demo.capture.bind(demo), undoCapture: demo.undoCapture.bind(demo),
      getToday: async () => { throw new Error("Database is temporarily unavailable"); },
      listWords: demo.listWords.bind(demo), getWord: demo.getWord.bind(demo),
      submitReview: demo.submitReview.bind(demo), getSettings: demo.getSettings.bind(demo),
      getReviewSessionInsight: demo.getReviewSessionInsight.bind(demo),
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
      getReviewSessionInsight: demo.getReviewSessionInsight.bind(demo),
      updateSettings: demo.updateSettings.bind(demo), replaceShortcut: demo.replaceShortcut.bind(demo),
      getPlatformCapabilities: demo.getPlatformCapabilities.bind(demo),
    };
    render(WindowsApp, { api: flaky });
    await screen.findByText("No captures yet");

    await saveManualCapture("durable", "The local record should be durable.");
    const dialog = screen.getByRole("dialog", { name: "Save a reading context" });
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

    await revealAndRate("Forgot");
    await fireEvent.click(await screen.findByRole("button", { name: "Next" }));
    expect(await screen.findByRole("heading", { name: "nuance" })).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Close review" }));

    expect(screen.getByRole("heading", { level: 1, name: "Review" })).toBeVisible();
    expect(await screen.findByText("2 words remaining.")).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Resume review" }));
    expect(screen.getByRole("heading", { name: "nuance" })).toBeVisible();

    await revealAndRate("Remembered");
    await fireEvent.click(await screen.findByRole("button", { name: "Next" }));
    await revealAndRate("Remembered");
    await fireEvent.click(await screen.findByRole("button", { name: "Next" }));
    expect(await screen.findByRole("heading", { name: "Review complete" })).toBeVisible();
    expect(screen.getByText("3 reviewed")).toBeVisible();
    expect(screen.getByText("2 remembered · 1 forgot")).toBeVisible();
    expect(screen.getByText(/Estimated due by the end of tomorrow: 1/)).toBeVisible();
    expect(api.ratings.map(({ rating }) => rating)).toEqual(["forgot", "remembered", "remembered"]);

    await fireEvent.click(screen.getByRole("button", { name: "Today" }));
    expect(await screen.findByText("Review queue is clear")).toBeVisible();
  });

  it("requires revealing the answer before a Review can be rated", async () => {
    render(WindowsApp, { api: new DemoBackend(true) });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    expect(screen.getByRole("heading", { name: "serendipity" })).toBeVisible();
    expect(screen.queryByText("glücklicher Zufall")).toBeNull();
    expect(screen.queryByRole("button", { name: "Forgot" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Remembered" })).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Show answer" }));
    expect(screen.getByText("glücklicher Zufall")).toBeVisible();
    expect(screen.getByRole("button", { name: "Forgot" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Remembered" })).toBeVisible();
  });

  it("shows core-backed Encounter and repeated-forgetting Insight after rating", async () => {
    const api = new DemoBackend(true);
    const submit = api.submitReview.bind(api);
    api.submitReview = async (wordId, rating) => ({
      ...(await submit(wordId, rating)), encounterCount: 4, repeatedForgetting: true,
    });
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await revealAndRate("Forgot");

    expect(await screen.findByText("Encountered 4 times")).toBeVisible();
    expect(screen.getByText(/repeatedly forgotten/i)).toBeVisible();
    expect(screen.getByRole("button", { name: "Review contexts" })).toBeVisible();
  });

  it("shows an empty Review state when nothing is due", async () => {
    render(WindowsApp, { api: new DemoBackend(false) });
    await screen.findByText("No captures yet");

    await fireEvent.click(screen.getByRole("button", { name: "Review" }));
    expect(screen.getByText("Nothing due")).toBeVisible();
  });

  it("distinguishes the complete due backlog from today's planned Review", async () => {
    const api = new DemoBackend(false);
    for (let index = 1; index <= 7; index += 1) {
      await api.capture({ selectedText: `due-${index}`, sentence: `Context ${index}` });
    }
    render(WindowsApp, { api });

    expect(await screen.findByText(/7 total due/)).toBeVisible();
    expect(screen.getByRole("button", { name: "Start review (5)" })).toBeVisible();
  });

  it("scrolls configured recent captures and paginates the full vocabulary", async () => {
    const api = new DemoBackend(false);
    for (let index = 1; index <= 21; index += 1) {
      await api.capture({ selectedText: `word-${index}`, sentence: `Context ${index}` });
    }
    const { container } = render(WindowsApp, { api });

    expect(await screen.findByRole("button", { name: /word-21/i })).toBeVisible();
    expect(container.querySelector(".recent-captures-list")).toHaveClass("recent-captures-list");

    await fireEvent.click(screen.getByRole("button", { name: "Vocabulary" }));
    expect(screen.getByRole("navigation", { name: "Vocabulary pages" })).toBeVisible();
    expect(screen.getByLabelText("Page number")).toHaveValue(1);
    expect(screen.getByText("of 3")).toBeVisible();
    expect(screen.queryByRole("button", { name: /^word-1,/i })).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Last" }));
    expect(screen.getByLabelText("Page number")).toHaveValue(3);
    expect(screen.getByRole("button", { name: /^word-1,/i })).toBeVisible();

    await fireEvent.input(screen.getByLabelText("Page number"), { target: { value: "2" } });
    await fireEvent.submit(screen.getByRole("form", { name: "Go to vocabulary page" }));
    expect(screen.getByLabelText("Page number")).toHaveValue(2);
    expect(screen.getByRole("button", { name: "First" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Previous" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Next" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Last" })).toBeEnabled();
  });

  it("keeps the current review card available when rating fails", async () => {
    const api = new TrackingReviewBackend(true);
    const submit = api.submitReview.bind(api);
    let attempts = 0;
    api.submitReview = async (wordId, rating, submissionId) => {
      if (attempts++ === 0) throw new Error("Could not save review");
      return submit(wordId, rating, submissionId);
    };
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await revealAndRate("Remembered");
    expect(screen.getByRole("alert")).toHaveTextContent("Could not save review");
    expect(screen.getByRole("heading", { name: "serendipity" })).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Retry Remembered" }));
    expect(await screen.findByRole("button", { name: "Next" })).toBeVisible();
    expect(api.ratings[0].submissionId).toBeTruthy();
  });

  it("resumes a revealed but unrated card in Recall state", async () => {
    const api = new TrackingReviewBackend(true);
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await fireEvent.click(screen.getByRole("button", { name: "Show answer" }));
    expect(screen.getByText("glücklicher Zufall")).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Close review" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Resume review" }));

    expect(screen.queryByText("glücklicher Zufall")).toBeNull();
    expect(screen.getByRole("button", { name: "Show answer" })).toBeVisible();
    expect(api.ratings).toHaveLength(0);
  });

  it("reuses the logical submission after an ambiguous response", async () => {
    const api = new TrackingReviewBackend(true);
    const submit = api.submitReview.bind(api);
    let first = true;
    api.submitReview = async (wordId, rating, submissionId) => {
      const result = await submit(wordId, rating, submissionId);
      if (first) { first = false; throw new Error("Response was lost"); }
      return result;
    };
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await revealAndRate("Remembered");
    await fireEvent.click(await screen.findByRole("button", { name: "Retry Remembered" }));

    expect(await screen.findByRole("button", { name: "Next" })).toBeVisible();
    expect(api.ratings).toHaveLength(2);
    expect(api.ratings[0].submissionId).toBe(api.ratings[1].submissionId);
    expect((await api.getToday()).totalDueCount).toBe(2);
  });

  it("blocks stale Resume until a failed close refresh succeeds", async () => {
    const api = new FlakyReviewRefreshBackend(true);
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await revealAndRate("Forgot");
    await fireEvent.click(await screen.findByRole("button", { name: "Next" }));
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
    const submit = api.submitReview.bind(api);
    api.submitReview = async (wordId, rating) => {
      await new Promise<void>((resolve) => { release = resolve; });
      return submit(wordId, rating);
    };
    render(WindowsApp, { api });

    await fireEvent.click(await screen.findByRole("button", { name: "Start review (3)" }));
    await fireEvent.click(screen.getByRole("button", { name: "Show answer" }));
    await fireEvent.click(screen.getByRole("button", { name: "Remembered" }));
    expect(screen.getByRole("button", { name: "Close review" })).toBeDisabled();

    release?.();
    expect(await screen.findByRole("button", { name: "Next" })).toBeVisible();
  });

  it("persists and reads back Windows settings before applying visual preferences", async () => {
    const api = new TrackingSettingsBackend(false);
    const { container } = render(WindowsApp, { api });
    await screen.findByText("No captures yet");

    await fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    await fireEvent.change(screen.getByLabelText("Source language"), { target: { value: "fr" } });
    await fireEvent.change(screen.getByLabelText("Translate into"), { target: { value: "es" } });
    await fireEvent.input(screen.getByLabelText("Daily limit"), { target: { value: "4" } });
    await fireEvent.input(screen.getByLabelText("Recent captures"), { target: { value: "12" } });
    await fireEvent.input(screen.getByLabelText("Selection Capture shortcut"), { target: { value: "Control+Shift+W" } });
    await fireEvent.input(screen.getByLabelText("Region OCR Capture shortcut"), { target: { value: "Control+Shift+O" } });
    await fireEvent.change(screen.getByLabelText("Theme"), { target: { value: "light" } });
    await fireEvent.click(screen.getByLabelText("Reduce motion"));
    await fireEvent.click(screen.getByRole("button", { name: "Save settings" }));

    expect(await screen.findByRole("status")).toHaveTextContent("Settings saved");
    expect(api.updates).toEqual([{
      sourceLanguage: "fr", targetLanguage: "es", reviewTime: "18:00", dailyLimit: 4,
      recentCapturesLimit: 12,
      selectionCaptureShortcut: "Control+Shift+W", regionOcrCaptureShortcut: "Control+Shift+O", launchAtLogin: false, appearance: "light",
      reducedMotion: true,
    }]);
    expect(screen.getByLabelText("Selection Capture shortcut")).toHaveValue("Control+Shift+W");
    expect(screen.getByLabelText("Region OCR Capture shortcut")).toHaveValue("Control+Shift+O");
    expect(container.querySelector(".windows-shell")).toHaveAttribute("data-appearance", "light");
    expect(container.querySelector(".windows-shell")).toHaveAttribute("data-reduced-motion", "true");
  });

  it("offers automatic source detection but requires an explicit target", async () => {
    const api = new TrackingSettingsBackend(false);
    render(WindowsApp, { api });
    await screen.findByText("No captures yet");

    await fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    const source = screen.getByLabelText("Source language");
    const target = screen.getByLabelText("Translate into");
    expect(within(source).getByRole("option", { name: "Auto detect" })).toHaveValue("auto");
    expect(within(target).queryByRole("option", { name: "Auto detect" })).toBeNull();
    expect(within(source).getByRole("option", { name: "Chinese (Simplified)" })).toHaveValue("zh-Hans");
    expect(within(target).getByRole("option", { name: "Chinese (Traditional)" })).toHaveValue("zh-Hant");

    await fireEvent.change(source, { target: { value: "auto" } });
    await fireEvent.change(target, { target: { value: "zh-Hant" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save settings" }));

    expect(api.updates[0]).toMatchObject({ sourceLanguage: "auto", targetLanguage: "zh-Hant" });
  });

  it("keeps saved visual preferences active when a settings save fails", async () => {
    const api = new TrackingSettingsBackend(false);
    api.updateSettings = async () => { throw new Error("Could not save settings"); };
    const { container } = render(WindowsApp, { api });
    await screen.findByText("No captures yet");

    await fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    await fireEvent.change(screen.getByLabelText("Theme"), { target: { value: "light" } });
    await fireEvent.click(screen.getByLabelText("Reduce motion"));
    await fireEvent.click(screen.getByRole("button", { name: "Save settings" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Could not save settings");
    expect(container.querySelector(".windows-shell")).toHaveAttribute("data-appearance", "system");
    expect(container.querySelector(".windows-shell")).toHaveAttribute("data-reduced-motion", "false");
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("applies persisted visual settings when the subsequent Today refresh fails", async () => {
    const { container } = render(WindowsApp, { api: new FlakySettingsRefreshBackend(false) });
    await screen.findByText("No captures yet");

    await fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    await fireEvent.change(screen.getByLabelText("Theme"), { target: { value: "light" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save settings" }));

    expect(await screen.findByRole("status")).toHaveTextContent("Settings saved");
    expect(await screen.findByRole("alert")).toHaveTextContent("Could not refresh Today");
    expect(container.querySelector(".windows-shell")).toHaveAttribute("data-appearance", "light");
  });

  it("shows per-setting system failures while preserving unrelated saved values", async () => {
    const api = new PartiallyFailingSystemSettingsBackend(false);
    const { container } = render(WindowsApp, { api });
    await screen.findByText("No captures yet");

    await fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    await fireEvent.input(screen.getByLabelText("Review time"), { target: { value: "08:30" } });
    await fireEvent.input(screen.getByLabelText("Selection Capture shortcut"), { target: { value: "Control+Shift+W" } });
    await fireEvent.click(screen.getByLabelText("Launch at login"));
    await fireEvent.change(screen.getByLabelText("Theme"), { target: { value: "light" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save settings" }));

    expect(await screen.findByText("Shortcut unavailable")).toBeVisible();
    expect(screen.getByText("Startup registration failed")).toBeVisible();
    expect(screen.getByText("Notification schedule failed")).toBeVisible();
    expect(screen.getByLabelText("Review time")).toHaveValue("18:00");
    expect(screen.getByLabelText("Selection Capture shortcut")).toHaveValue("Alt+Shift+V");
    expect(screen.getByLabelText("Launch at login")).not.toBeChecked();
    expect(container.querySelector(".windows-shell")).toHaveAttribute("data-appearance", "light");
    expect(screen.getByRole("status")).toHaveTextContent("Settings saved");
  });
});
