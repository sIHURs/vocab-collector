import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import App from "./App.svelte";

describe("application shell", () => {
  it("provides compact navigation for the main product areas", async () => {
    render(App);

    expect(screen.getByRole("heading", { name: "Today" })).toBeVisible();
    expect(screen.getByText("Vocab Collector")).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Vocabulary" }));
    expect(screen.getByRole("heading", { name: "Vocabulary" })).toBeVisible();

    await fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(screen.getByRole("heading", { name: "Settings" })).toBeVisible();
  });

  it("supports quick capture without leaving the reading flow", async () => {
    render(App);
    await screen.findByText("3 words ready");

    await fireEvent.click(screen.getByRole("button", { name: "Quick capture" }));
    await fireEvent.input(screen.getByLabelText("Word"), { target: { value: "lucid" } });
    await fireEvent.input(screen.getByLabelText("Translation"), { target: { value: "klar" } });
    await fireEvent.input(screen.getByLabelText("Context"), {
      target: { value: "The author offered a lucid explanation." },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save word" }));

    expect(await screen.findByText("Saved · Undo")).toBeVisible();
    expect(within(screen.getByRole("dialog", { name: "Quick capture" })).getByText("klar")).toBeVisible();
  });

  it("runs the focused daily review flow", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "Review 3 words" }));

    expect(screen.getByRole("heading", { name: "serendipity" })).toBeVisible();
    await fireEvent.click(screen.getByRole("button", { name: "Remembered" }));
    expect(screen.getByRole("heading", { name: "nuance" })).toBeVisible();
  });
});
