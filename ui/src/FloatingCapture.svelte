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
    "translation_unavailable", "cancelled", "operation",
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

  async function hide() {
    if (dismissTimer) clearTimeout(dismissTimer);
    await invoke("hide_capture_window");
  }

  async function accept(event: NativeCaptureEvent, confirmOcr = false) {
    const { requestId, candidate: next } = event;
    activeRequest = requestId;
    if (dismissTimer) clearTimeout(dismissTimer);
    candidate = next;
    saved = null;
    failure = null;
    translationFailure = null;
    ocrNeedsConfirmation = false;
    permissionAction = "accessibility";
    saving = true;
    try {
      if (confirmOcr) await invoke("confirm_ocr", { requestId });
      const settings = await invoke<{ sourceLanguage: string; targetLanguage: string }>("get_settings");
      try {
        await invoke<{ translatedText: string }>("translate_text", {
          requestId,
          text: next.selectedText,
          sourceLanguage: settings.sourceLanguage,
          targetLanguage: settings.targetLanguage,
        });
      } catch (cause) {
        if (requestId !== activeRequest) return;
        translationFailure = asCaptureFailure(cause);
        return;
      }
      if (requestId === activeRequest) await persist(requestId, false);
    } catch (cause) {
      if (requestId === activeRequest) failure = asCaptureFailure(cause);
    } finally {
      if (requestId === activeRequest) saving = false;
    }
  }

  async function persist(requestId = activeRequest, withoutTranslation = false) {
    if (!requestId || requestId !== activeRequest) return;
    saved = await invoke<CaptureCard>("save_native_capture", { requestId, withoutTranslation });
    dismissTimer = setTimeout(hide, 4_000);
  }

  function offerOcr(event: NativeCaptureEvent) {
    activeRequest = event.requestId;
    if (dismissTimer) clearTimeout(dismissTimer);
    candidate = event.candidate;
    saved = null;
    failure = null;
    saving = false;
    translationFailure = null;
    ocrNeedsConfirmation = true;
    permissionAction = "accessibility";
  }

  async function undo() {
    if (!saved) return;
    await invoke("undo_capture", { encounterId: saved.encounterId });
    saved = null;
    await hide();
  }

  async function grantAccessibility() {
    await invoke("request_accessibility_permission");
    failure = { code: "operation", message: "Permission requested. Select text and press your shortcut again." };
  }

  async function useOcr() {
    permissionAction = "screen_recording";
    try {
      const status = await invoke<string>("request_screen_recording_permission");
      if (status !== "granted") {
        permissionAction = "screen_recording";
        failure = { code: "permission_required", message: "Screen Recording permission was requested. Enable it in System Settings, then try OCR again." };
        return;
      }
      await invoke("capture_with_ocr", { requestId: activeRequest });
    } catch (cause) { failure = asCaptureFailure(cause); }
  }

  onMount(() => {
    let mounted = true;
    void backend.getPlatformCapabilities()
      .then((capabilities) => { if (mounted) platformCapabilities = capabilities; })
      .catch(() => { /* Conservative defaults remain active. */ });
    const ready = listen<NativeCaptureEvent>("capture-ready", ({ payload }) => accept(payload));
    const failed = listen<NativeCaptureErrorEvent>("capture-error", ({ payload }) => { activeRequest = payload.requestId; candidate = null; saved = null; translationFailure = null; permissionAction = "accessibility"; failure = payload; });
    const ocr = listen<NativeCaptureEvent>("ocr-candidate", ({ payload }) => offerOcr(payload));
    return () => { mounted = false; ready.then((unlisten) => unlisten()); failed.then((unlisten) => unlisten()); ocr.then((unlisten) => unlisten()); if (dismissTimer) clearTimeout(dismissTimer); };
  });
</script>

<main class="native-capture" onmouseenter={() => dismissTimer && clearTimeout(dismissTimer)} onmouseleave={() => saved && (dismissTimer = setTimeout(hide, 4_000))}>
  <header><span><i></i> Vocab Collector</span><button aria-label="Close capture" onclick={hide}>×</button></header>
  {#if failure}
    <section class="capture-message"><strong>Capture needs attention</strong><p>{failure.message}</p>
      {#if failure.code === "permission_required" && permissionAction === "accessibility"}<button class="primary" onclick={grantAccessibility}>Allow Accessibility</button>{/if}
      {#if failure.code === "permission_required" && permissionAction === "screen_recording"}<button class="primary" onclick={useOcr}>Allow Screen Recording</button>{/if}
      {#if failure.code === "empty_selection" && platformCapabilities.screenshotOcr}<button class="primary" onclick={useOcr}>Use OCR near pointer</button>{/if}
    </section>
  {:else if saved}
    <section class="capture-result"><span class="check">✓</span><div><small>Saved</small><h1>{saved.displayForm}</h1><strong>{saved.translation ?? "Translation pending"}</strong><p>“{saved.context}”</p><small>{saved.isExistingWord ? `Seen ${saved.encounterCount} times · New context saved` : "Added to your review queue"}</small></div></section>
    <button class="undo" onclick={undo}>Undo</button>
  {:else if candidate}
    <section class="capture-result"><div><small>{ocrNeedsConfirmation ? "OCR suggestion · Confirm before saving" : saving ? "Translating…" : "Captured"}</small><h1>{candidate.selectedText}</h1><p>“{candidate.sentence}”</p><small>{candidate.sourceApp ?? "Current application"}</small>
      {#if ocrNeedsConfirmation}<button class="primary" onclick={() => candidate && accept({ requestId: activeRequest, candidate }, true)}>Use this text</button>
      {:else if translationFailure}<p>{translationFailure.message}</p>{#if translationFailure.code === "translation_unavailable"}<button class="primary" onclick={() => persist(activeRequest, true)}>Save without translation</button>{/if}{/if}
    </div></section>
  {:else}
    <section class="capture-message"><strong>Ready to capture</strong><p>Select text in another app, then press your shortcut.</p></section>
  {/if}
</main>

<style>
  :global(html), :global(body), :global(#app) { width: 100%; height: 100%; margin: 0; background: transparent; overflow: hidden; }
  .native-capture { box-sizing: border-box; width: 100%; min-height: 220px; padding: 14px 16px 16px; color: #e9e7f2; background: rgba(30,29,38,.97); border: 1px solid #4b475b; border-radius: 12px; box-shadow: 0 18px 48px rgba(0,0,0,.42); font: 14px -apple-system, BlinkMacSystemFont, sans-serif; }
  header { display:flex; justify-content:space-between; align-items:center; color:#a8a4b6; font-size:12px; }
  header span { display:flex; align-items:center; gap:7px; } header i { width:7px; height:7px; border-radius:50%; background:#9b7cff; box-shadow:0 0 10px #9b7cff; }
  button { border:0; color:inherit; background:transparent; cursor:pointer; } header button { font-size:20px; }
  section { padding:18px 4px 6px; } h1 { margin:3px 0 4px; font-size:25px; font-weight:650; } p { color:#b9b5c5; line-height:1.45; } small { color:#817c91; }
  .capture-result { display:flex; gap:12px; } .check { display:grid; place-items:center; flex:0 0 25px; height:25px; border-radius:50%; background:#765bd7; }
  .undo { margin:9px 0 0 40px; color:#a98fff; } .primary { padding:8px 12px; border-radius:7px; background:#765bd7; color:white; }
</style>
