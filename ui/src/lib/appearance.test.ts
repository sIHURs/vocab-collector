import { it, expect, vi } from "vitest";
import { connectAppearance } from "./appearance";
import { DemoBackend } from "./backend";

it("applies persisted appearance and refreshes another window only after a settings notification", async () => {
  const api = new DemoBackend(false);
  let changed = () => {};
  const unlisten = vi.fn();
  const stop = await connectAppearance({ getSettings: () => api.getSettings(), listenSettingsChanged: async (handler) => { changed = handler; return unlisten; } }, document.documentElement);
  expect(document.documentElement.dataset.appearance).toBe("system");
  const draft = { ...await api.getSettings(), appearance: "dark" as const, reducedMotion: true };
  expect(document.documentElement.dataset.appearance).toBe("system");
  await api.updateSettings(draft);
  changed();
  await vi.waitFor(() => expect(document.documentElement.dataset.appearance).toBe("dark"));
  expect(document.documentElement.dataset.reducedMotion).toBe("true");
  stop();
  expect(unlisten).toHaveBeenCalledOnce();
});

it("keeps the latest saved appearance when an older startup read finishes later", async () => {
  const settings = await new DemoBackend(false).getSettings();
  const root = document.createElement("html");
  let changed = () => {};
  let resolveInitial: (value: typeof settings) => void = () => {};
  const initial = new Promise<typeof settings>(resolve => { resolveInitial = resolve; });
  const getSettings = vi.fn().mockReturnValueOnce(initial).mockResolvedValue({ ...settings, appearance: "dark" });
  const connected = connectAppearance({ getSettings, listenSettingsChanged: async handler => { changed = handler; return () => {}; } }, root);
  await vi.waitFor(() => expect(getSettings).toHaveBeenCalledOnce());
  changed();
  await vi.waitFor(() => expect(root.dataset.appearance).toBe("dark"));
  resolveInitial(settings);
  const stop = await connected;
  expect(root.dataset.appearance).toBe("dark");
  stop();
});

it("still reads saved settings if subscribing to notifications fails", async () => {
  const api = new DemoBackend(false);
  const root = document.createElement("html");
  const stop = await connectAppearance({ getSettings: () => api.getSettings(), listenSettingsChanged: async () => { throw new Error("Unavailable"); } }, root);
  expect(root.dataset.appearance).toBe("system");
  stop();
});
