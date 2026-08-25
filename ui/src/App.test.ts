import { fireEvent, render, screen } from "@testing-library/svelte";
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
});
