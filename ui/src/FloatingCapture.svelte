<script lang="ts">
  import './native-capture.css';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import * as Alert from '$lib/components/ui/alert';
  import CaptureSource from './components/CaptureSource.svelte';
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
  let hovered = false;

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
    if (hovered) return;
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

<main class="native-capture capture-surface" aria-label="Capture" onmouseenter={() => { hovered = true; clearDismissTimer(); }} onmouseleave={() => { hovered = false; if (saved) scheduleDismissal(activeRequest); }}>
  <header data-tauri-drag-region><span data-tauri-drag-region><i aria-hidden="true" data-tauri-drag-region></i>Vocab Collector</span><Button variant="ghost" size="icon" aria-label="Close capture" onclick={() => hide(activeRequest)}>×</Button></header>
  <section class="capture-body" aria-live="polite" aria-busy={saving}>
    {#if failure}<Alert.Root variant="destructive"><Alert.Title>Capture needs attention</Alert.Title><Alert.Description>{failure.message}</Alert.Description></Alert.Root>
    {:else if saved}<Badge variant="secondary">Saved</Badge><h1>{saved.displayForm}</h1><p class="translation">{saved.translation ?? "Saved without translation"}</p><p class="context">“{saved.context}”</p><small>{saved.isExistingWord ? `Seen ${saved.encounterCount} times · New context saved` : "Added to your review queue"}</small><CaptureSource app={candidate?.sourceApp} title={candidate?.sourceTitle} url={candidate?.sourceUrl} />
    {:else if candidate}<small>{ocrNeedsConfirmation ? "OCR suggestion · Confirm before saving" : saving ? "Translating…" : "Captured"}</small><h1>{candidate.selectedText}</h1><p class="context">“{candidate.sentence}”</p><CaptureSource app={candidate.sourceApp ?? 'Current application'} title={candidate.sourceTitle} url={candidate.sourceUrl} />
      {#if translationFailure}<Alert.Root variant="destructive"><Alert.Title>Translation needs attention</Alert.Title><Alert.Description>{translationFailure.message}</Alert.Description></Alert.Root>{/if}
    {:else}<strong>Ready to capture</strong><p>Select text in another app, then press your shortcut.</p>{/if}
  </section>
  <footer aria-label="Capture actions">
    {#if failure}
      {#if failure.code === "permission_required" && permissionAction === "accessibility"}<Button onclick={grantAccessibility}>Allow Accessibility</Button>{/if}
      {#if failure.code === "permission_required" && permissionAction === "screen_recording"}<Button onclick={useOcr}>Allow Screen Recording</Button>{/if}
      {#if failure.code === "empty_selection" && platformCapabilities.screenshotOcr}<Button onclick={useOcr}>Start Region OCR</Button>{/if}
    {:else if saved}<Button variant="outline" onclick={undo}>Undo</Button>
    {:else if candidate}
      {#if ocrNeedsConfirmation}<Button onclick={() => candidate && accept({ requestId: activeRequest, candidate }, true)}>Use this text</Button>
      {:else if translationFailure && (translationFailure.code === "translation_unavailable" || translationFailure.code === "translation_failed")}<Button variant="outline" onclick={() => candidate && accept({ requestId: activeRequest, candidate })}>Retry translation</Button><Button onclick={() => persist(activeRequest, true)}>Save without translation</Button>{/if}
    {/if}
  </footer>
</main>
<style>
:global(html),:global(body),:global(#app){width:100%;height:100%;min-height:0;min-width:0;margin:0;background:transparent;overflow:hidden}
</style>
