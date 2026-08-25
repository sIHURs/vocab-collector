import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import App from "./App.svelte";

describe("application scaffold", () => {
  it("identifies the product and its local-first foundation", () => {
    render(App);

    expect(screen.getByRole("heading", { name: "Vocab Collector" })).toBeVisible();
    expect(screen.getByText("Local-first application foundation")).toBeVisible();
  });
});
