<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { backend } from "./lib/backend";
  import type {
    CaptureCandidate,
    CaptureCard,
    CaptureFailure,
    CaptureFailureCode,
    PlatformCapabilities,
  } from "./lib/types";

  type NativeCaptureEvent = { requestId: string; candidate: CaptureCandidate };
  type NativeCaptureErrorEvent = CaptureFailure & { requestId: string };
  const failureCodes = new Set<CaptureFailureCode>([
    "permission_required", "permission_denied", "empty_selection", "unsupported_element",
    "translation_unavailable", "translation_failed", "cancelled", "operation",
  ]);
  const unavailableCapabilities: PlatformCapabilities = {
    selectionCapture: false,
    selectionBounds: false,
    screenshotOcr: false,
    translation: false,
    nonActivatingWindow: false,
  };
  let candidate: CaptureCandidate | null = null;
  let saved: CaptureCard | null = null;
  let failure: CaptureFailure | null = null;
  let saving = false;
  let translationFailure: CaptureFailure | null = null;
  let ocrNeedsConfirmation = false;
  let permissionAction: "accessibility" | "screen_recording" = "accessibility";
  let platformCapabilities = unavailableCapabilities;
  let dismissTimer: ReturnType<typeof setTimeout> | undefined;
  let activeRequest = "";
  let mounted = false;

  function isActiveRequest(requestId: string) {
    return mounted && Boolean(requestId) && requestId === activeRequest;
  }

  function clearDismissTimer() {
    if (dismissTimer) clearTimeout(dismissTimer);
    dismissTimer = undefined;
  }

  function asCaptureFailure(cause: unknown): CaptureFailure {
    if (typeof cause === "object" && cause !== null) {
      const value = cause as { code?: unknown; message?: unknown };
      if (typeof value.code === "string" && failureCodes.has(value.code as CaptureFailureCode)) {
        return {
          code: value.code as CaptureFailureCode,
          message: typeof value.message === "string" ? value.message : "Capture operation failed.",
        };
      }
    }
    return {
      code: "operation",
      message: cause instanceof Error ? cause.message : String(cause),
    };
  }

  async function hide(requestId = activeRequest) {
    if (!isActiveRequest(requestId)) return;
    clearDismissTimer();
    try {
      await invoke("hide_capture_window", { requestId });
    } catch (cause) {
      if (isActiveRequest(requestId)) failure = asCaptureFailure(cause);
    }
  }

  function scheduleDismissal(requestId: string) {
    if (!isActiveRequest(requestId)) return;
    clearDismissTimer();
    dismissTimer = setTimeout(() => { void hide(requestId); }, 4_000);
  }

  async function accept(event: NativeCaptureEvent, confirmOcr = false) {
    const { requestId, candidate: next } = event;
    if (!mounted) return;
    activeRequest = requestId;
    clearDismissTimer();
    candidate = next;
    saved = null;
    failure = null;
    translationFailure = null;
    ocrNeedsConfirmation = false;
    permissionAction = "accessibility";
    saving = true;
    try {
      if (confirmOcr) {
        await invoke("confirm_ocr", { requestId });
        if (!isActiveRequest(requestId)) return;
      }
      const settings = await invoke<{ sourceLanguage: string; targetLanguage: string }>("get_settings");
      if (!isActiveRequest(requestId)) return;
      try {
        await invoke<{ translatedText: string }>("translate_text", {
          requestId,
          text: next.selectedText,
          sourceLanguage: settings.sourceLanguage,
          targetLanguage: settings.targetLanguage,
        });
      } catch (cause) {
        if (!isActiveRequest(requestId)) return;
        translationFailure = asCaptureFailure(cause);
        return;
      }
      if (isActiveRequest(requestId)) await persist(requestId, false);
    } catch (cause) {
      if (isActiveRequest(requestId)) failure = asCaptureFailure(cause);
    } finally {
      if (isActiveRequest(requestId)) saving = false;
    }
  }

  async function persist(requestId = activeRequest, withoutTranslation = false) {
    if (!isActiveRequest(requestId)) return;
    try {
      const nextSaved = await invoke<CaptureCard>("save_native_capture", { requestId, withoutTranslation });
      if (!isActiveRequest(requestId)) return;
      saved = nextSaved;
      scheduleDismissal(requestId);
    } catch (cause) {
      if (!isActiveRequest(requestId)) return;
      if (withoutTranslation) translationFailure = asCaptureFailure(cause);
      else failure = asCaptureFailure(cause);
    }
  }

  function offerOcr(event: NativeCaptureEvent) {
    if (!mounted) return;
    activeRequest = event.requestId;
    clearDismissTimer();
    candidate = event.candidate;
    saved = null;
    failure = null;
    saving = false;
    translationFailure = null;
    ocrNeedsConfirmation = true;
    permissionAction = "accessibility";
  }

  async function undo() {
    const requestId = activeRequest;
    const encounterId = saved?.encounterId;
    if (!encounterId || !isActiveRequest(requestId)) return;
    clearDismissTimer();
    try {
      await invoke("undo_capture", { encounterId });
      if (!isActiveRequest(requestId)) return;
      saved = null;
      await hide(requestId);
    } catch (cause) {
      if (isActiveRequest(requestId)) failure = asCaptureFailure(cause);
    }
  }

  async function grantAccessibility() {
    const requestId = activeRequest;
    if (!isActiveRequest(requestId)) return;
    try {
      await invoke("request_accessibility_permission");
      if (!isActiveRequest(requestId)) return;
      failure = { code: "operation", message: "Permission requested. Select text and press your shortcut again." };
    } catch (cause) {
      if (isActiveRequest(requestId)) failure = asCaptureFailure(cause);
    }
  }

  async function useOcr() {
    const requestId = activeRequest;
    if (!isActiveRequest(requestId)) return;
    permissionAction = "screen_recording";
    try {
      const status = await invoke<string>("request_screen_recording_permission");
      if (!isActiveRequest(requestId)) return;
      if (status !== "granted") {
        permissionAction = "screen_recording";
        failure = { code: "permission_required", message: "Screen Recording permission was requested. Enable it in System Settings, then try OCR again." };
        return;
      }
      await invoke("start_region_ocr_capture");
    } catch (cause) {
      if (isActiveRequest(requestId)) failure = asCaptureFailure(cause);
    }
  }

  onMount(() => {
    mounted = true;
    void backend.getPlatformCapabilities()
      .then((capabilities) => { if (mounted) platformCapabilities = capabilities; })
      .catch(() => { /* Conservative defaults remain active. */ });
    const ready = listen<NativeCaptureEvent>("capture-ready", ({ payload }) => { if (mounted) void accept(payload); });
    const failed = listen<NativeCaptureErrorEvent>("capture-error", ({ payload }) => { if (mounted) { activeRequest = payload.requestId; clearDismissTimer(); candidate = null; saved = null; translationFailure = null; permissionAction = "accessibility"; failure = payload; } });
    const ocr = listen<NativeCaptureEvent>("ocr-candidate", ({ payload }) => { if (mounted) offerOcr(payload); });
    return () => { mounted = false; ready.then((unlisten) => unlisten()); failed.then((unlisten) => unlisten()); ocr.then((unlisten) => unlisten()); clearDismissTimer(); };
  });
</script>

<main class="native-capture" onmouseenter={clearDismissTimer} onmouseleave={() => saved && scheduleDismissal(activeRequest)}>
  <header data-tauri-drag-region><span data-tauri-drag-region><i data-tauri-drag-region></i> Vocab Collector</span><button aria-label="Close capture" onclick={() => hide(activeRequest)}>×</button></header>
  {#if failure}
    <section class="capture-message"><strong>Capture needs attention</strong><p>{failure.message}</p>
      {#if failure.code === "permission_required" && permissionAction === "accessibility"}<button class="primary" onclick={grantAccessibility}>Allow Accessibility</button>{/if}
      {#if failure.code === "permission_required" && permissionAction === "screen_recording"}<button class="primary" onclick={useOcr}>Allow Screen Recording</button>{/if}
      {#if failure.code === "empty_selection" && platformCapabilities.screenshotOcr}<button class="primary" onclick={useOcr}>Start Region OCR</button>{/if}
    </section>
  {:else if saved}
    <section class="capture-result"><span class="check">✓</span><div><small>Saved</small><h1>{saved.displayForm}</h1><strong>{saved.translation ?? "Translation pending"}</strong><p>“{saved.context}”</p><small>{saved.isExistingWord ? `Seen ${saved.encounterCount} times · New context saved` : "Added to your review queue"}</small></div></section>
    <button class="undo" onclick={undo}>Undo</button>
  {:else if candidate}
    <section class="capture-result"><div><small>{ocrNeedsConfirmation ? "OCR suggestion · Confirm before saving" : saving ? "Translating…" : "Captured"}</small><h1>{candidate.selectedText}</h1><p>“{candidate.sentence}”</p><small>{candidate.sourceApp ?? "Current application"}</small>
      {#if ocrNeedsConfirmation}<button class="primary" onclick={() => candidate && accept({ requestId: activeRequest, candidate }, true)}>Use this text</button>
      {:else if translationFailure}<p>{translationFailure.message}</p>{#if translationFailure.code === "translation_unavailable" || translationFailure.code === "translation_failed"}<button class="primary" onclick={() => candidate && accept({ requestId: activeRequest, candidate })}>Retry translation</button><button class="primary" onclick={() => persist(activeRequest, true)}>Save without translation</button>{/if}{/if}
    </div></section>
  {:else}
    <section class="capture-message"><strong>Ready to capture</strong><p>Select text in another app, then press your shortcut.</p></section>
  {/if}
</main>

<style>
  :global(html), :global(body), :global(#app) { width: 100%; height: 100%; margin: 0; background: transparent; overflow: hidden; }
  .native-capture { box-sizing: border-box; width: 100%; min-height: 220px; padding: 14px 16px 16px; color: var(--foreground); background: var(--card); border: 1px solid var(--border); border-radius: 12px; box-shadow: 0 18px 48px rgba(0,0,0,.42); font: 14px -apple-system, BlinkMacSystemFont, sans-serif; }
  header { display:flex; justify-content:space-between; align-items:center; color:var(--muted-foreground); font-size:12px; cursor:move; user-select:none; }
  header span { display:flex; align-items:center; gap:7px; } header i { width:7px; height:7px; border-radius:50%; background:var(--primary); box-shadow:0 0 10px var(--primary); }
  button { border:0; color:inherit; background:transparent; cursor:pointer; } header button { font-size:20px; cursor:pointer; }
  section { padding:18px 4px 6px; } h1 { margin:3px 0 4px; font-size:25px; font-weight:650; } p { color:var(--muted-foreground); line-height:1.45; } small { color:var(--muted-foreground); }
  .capture-result { display:flex; gap:12px; } .check { display:grid; place-items:center; flex:0 0 25px; height:25px; border-radius:50%; background:var(--primary); }
  .undo { margin:9px 0 0 40px; color:var(--primary); } .primary { padding:8px 12px; border-radius:7px; background:var(--primary); color:var(--primary-foreground); }
</style>
