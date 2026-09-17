import type { Settings } from "./types";

type AppearanceSource = {
  getSettings(): Promise<Settings>;
  listenSettingsChanged?(handler: () => void): Promise<() => void>;
};

export function applyAppearance(settings: Pick<Settings, "appearance" | "reducedMotion">, root = document.documentElement) {
  root.dataset.appearance = settings.appearance;
  root.dataset.reducedMotion = String(settings.reducedMotion);
}

/** Listen before reading so an in-flight startup read cannot overwrite a newer save. */
export async function connectAppearance(source: AppearanceSource, root = document.documentElement): Promise<() => void> {
  let version = 0;
  let disposed = false;
  const refresh = async () => {
    const request = ++version;
    try {
      const settings = await source.getSettings();
      if (!disposed && request === version) applyAppearance(settings, root);
    } catch { /* Keep the last saved appearance when a read is unavailable. */ }
  };
  let unlisten: (() => void) | undefined;
  try { unlisten = await source.listenSettingsChanged?.(() => { void refresh(); }); }
  catch { /* Startup still reads the saved settings if notifications are unavailable. */ }
  await refresh();
  return () => { disposed = true; version++; unlisten?.(); };
}
