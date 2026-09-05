<script lang="ts">
  import { onMount } from "svelte";
  import type { AchievedCaptureConflict, CaptureCandidate, CaptureCard } from "../lib/types";
  import {
    tauriWindowsCaptureBackend,
    type OcrCandidate,
    type WindowsCaptureBackend,
  } from "./captureBackend";

  export let captureBackend: WindowsCaptureBackend = tauriWindowsCaptureBackend;
  let activeRequest = "";
  let candidate: CaptureCandidate | null = null;
  let editing = false;
  let selectedText = "";
  let sentence = "";
  let translation = "";
  let translationSource = "";
  let translationTarget = "";
  let translationAvailable = false;
  let translationFailed = false;
  let translationStale = false;
  let lastTranslatedText = "";
  let saved: CaptureCard | null = null;
  let achievedConflict: AchievedCaptureConflict | null = null;
  let busy = false;
  let error = "";
  let mounted = false;
  let dismissTimer: ReturnType<typeof setTimeout> | undefined;
  let ocrOffer = false;
  let ocrNeedsConfirmation = false;
  let ocrAvailable = false;
  let ocrEligibleFailure = false;
  let ocrCandidates: OcrCandidate[] = [];
  let ocrBusy = false;
  let ocrAttempted = false;

  function resetOcrState() {
    ocrOffer = false;
    ocrEligibleFailure = false;
    ocrNeedsConfirmation = false;
    ocrCandidates = [];
    ocrBusy = false;
    ocrAttempted = false;
  }

  function clearDismissTimer() {
    if (dismissTimer) clearTimeout(dismissTimer);
    dismissTimer = undefined;
  }

  function scheduleDismiss(requestId = activeRequest) {
    clearDismissTimer();
    if (!saved || !mounted || requestId !== activeRequest) return;
    dismissTimer = setTimeout(() => {
      if (mounted && requestId === activeRequest) void captureBackend.hide(requestId);
    }, 4_000);
  }

  async function beginEditing() {
    const requestId = activeRequest;
    await captureBackend.focus();
    if (!mounted || requestId !== activeRequest) return;
    editing = true;
  }

  function correction() {
    return {
      selectedText: selectedText.trim(),
      sentence: sentence.trim(),
      ...(translation.trim() ? { translation: translation.trim() } : {}),
    };
  }

  async function applyChanges() {
    if (!candidate || busy) return;
    const requestId = activeRequest;
    busy = true;
    error = "";
    try {
      await captureBackend.apply(requestId, correction());
      if (!mounted || requestId !== activeRequest) return;
      translationStale = Boolean(translation.trim()) && selectedText.trim() !== lastTranslatedText;
      editing = false;
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function save() {
    if (!candidate || busy) return;
    const requestId = activeRequest;
    busy = true;
    error = "";
    try {
      const conflict = await captureBackend.findAchieved(requestId);
      if (!mounted || requestId !== activeRequest) return;
      if (conflict) {
        achievedConflict = conflict;
        return;
      }
      const result = await captureBackend.save(requestId, !translation.trim());
      if (!mounted || requestId !== activeRequest) return;
      saved = result;
      editing = false;
      scheduleDismiss(requestId);
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function restoreToLearningAndSave() {
    if (!achievedConflict || busy) return;
    const requestId = activeRequest;
    const wordId = achievedConflict.wordId;
    busy = true;
    error = "";
    try {
      const result = await captureBackend.restoreAchievedAndSave(requestId, wordId, !translation.trim());
      if (!mounted || requestId !== activeRequest) return;
      achievedConflict = null;
      saved = result;
      editing = false;
      scheduleDismiss(requestId);
    } catch (cause) {
      if (mounted && requestId === activeRequest) error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function translateCandidate(requestId: string, text: string) {
    if (busy) return;
    busy = true;
    error = "";
    translationFailed = false;
    try {
      const [capabilities, currentSettings] = await Promise.all([
        captureBackend.getCapabilities(),
        captureBackend.getSettings(),
      ]);
      if (!mounted || requestId !== activeRequest) return;
      translationAvailable = capabilities.translation;
      if (!translationAvailable) return;
      const result = await captureBackend.translate(requestId, text, currentSettings.sourceLanguage, currentSettings.targetLanguage);
      if (!mounted || requestId !== activeRequest) return;
      translation = result.translatedText;
      lastTranslatedText = text;
      translationStale = false;
      translationSource = result.sourceLanguage;
      translationTarget = result.targetLanguage;
      await captureBackend.apply(requestId, correction());
    } catch {
      if (!mounted || requestId !== activeRequest) return;
      translationFailed = true;
      error = "Translation is temporarily unavailable. You can retry or continue editing.";
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function undo() {
    if (!saved || busy) return;
    const requestId = activeRequest;
    const encounterId = saved.encounterId;
    busy = true;
    error = "";
    try {
      await captureBackend.undo(requestId, encounterId);
      if (!mounted || requestId !== activeRequest) return;
      saved = null;
      achievedConflict = null;
      clearDismissTimer();
      await captureBackend.close(requestId);
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function confirmOcrCandidate() {
    if (busy) return;
    const requestId = activeRequest;
    if (!candidate || !selectedText.trim()) return;
    busy = true;
    try {
      await captureBackend.confirmOcr(requestId, selectedText, sentence);
      if (!mounted || requestId !== activeRequest) return;
      ocrNeedsConfirmation = false;
    } catch {
      if (mounted && requestId === activeRequest) error = "OCR confirmation failed. Please try again.";
      return;
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
    void translateCandidate(requestId, selectedText.trim());
  }

  async function startOcr() {
    if (!activeRequest || ocrBusy) return;
    const requestId = activeRequest;
    ocrBusy = true;
    ocrAttempted = true;
    error = "";
    try {
      await captureBackend.startRegionOcr();
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
      ocrOffer = true;
    } finally {
      if (mounted && requestId === activeRequest) ocrBusy = false;
    }
  }

  function useOcrDraft(candidates: OcrCandidate[]) {
    const text = candidates.map((item) => item.text.trim()).filter(Boolean).join(" ");
    const selected = candidates[0];
    if (!selected || !text) return;
    candidate = {
      selectedText: text,
      sentence: "",
      selectionBounds: selected.bounds,
      origin: "ocr",
    };
    selectedText = text;
    sentence = "";
  }

  async function cancel() {
    if (!activeRequest) return;
    try {
      await captureBackend.close(activeRequest);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function useManualCapture() {
    await captureBackend.openManualCapture();
    await cancel();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") void cancel();
  }

  onMount(() => {
    mounted = true;
    document.body.classList.add("windows-capture-document");
    const ready = captureBackend.listenReady((event) => {
      activeRequest = event.requestId;
      clearDismissTimer();
      candidate = event.candidate;
      selectedText = event.candidate.selectedText;
      sentence = event.candidate.sentence;
      translation = "";
      translationSource = "";
      translationTarget = "";
      translationFailed = false;
      translationStale = false;
      lastTranslatedText = "";
      saved = null;
      achievedConflict = null;
      error = "";
      editing = false;
      resetOcrState();
      if (event.candidate.origin !== "ocr") void translateCandidate(event.requestId, event.candidate.selectedText);
    });
    void captureBackend.getCapabilities().then((capabilities) => {
      if (mounted) {
        ocrAvailable = capabilities.screenshotOcr;
        ocrOffer = ocrEligibleFailure && ocrAvailable;
      }
    });
    const failed = captureBackend.listenError((event) => {
      activeRequest = event.requestId;
      candidate = null;
      saved = null;
      achievedConflict = null;
      error = event.failure.message;
      ocrEligibleFailure = event.failure.code === "empty_selection" || event.failure.code === "unsupported_element";
      ocrOffer = ocrAvailable && (ocrEligibleFailure || ocrAttempted);
      ocrNeedsConfirmation = false;
      ocrCandidates = [];
      ocrBusy = false;
    });
    const ocr = captureBackend.listenOcrCandidate((event) => {
      if (event.requestId !== activeRequest) return;
      ocrCandidates = event.candidates;
      useOcrDraft(event.candidates);
      error = "";
      ocrOffer = false;
      ocrNeedsConfirmation = true;
      void captureBackend.focus();
    });
    const regionOcr = captureBackend.listenRegionOcrStart((event) => {
      activeRequest = event.requestId;
      candidate = null;
      saved = null;
      error = "请框取词汇";
      ocrOffer = false;
      ocrAttempted = true;
    });
    return () => {
      mounted = false;
      clearDismissTimer();
      document.body.classList.remove("windows-capture-document");
      ready.then((unlisten) => unlisten());
      failed.then((unlisten) => unlisten());
      ocr.then((unlisten) => unlisten());
      regionOcr.then((unlisten) => unlisten());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="windows-capture" data-presentation="windows-capture" aria-label="Capture" onmouseenter={clearDismissTimer} onmouseleave={() => scheduleDismiss()} onfocusin={clearDismissTimer} onfocusout={() => scheduleDismiss()}>
  <header data-tauri-drag-region><span data-tauri-drag-region><i aria-hidden="true" data-tauri-drag-region></i>Vocab Collector</span><button aria-label="Cancel capture" onclick={cancel}>×</button></header>
  <section class="scrollable-content" aria-live="polite" aria-busy={busy || ocrBusy}>
    {#if saved}
      <small>Saved</small>
      {#if error}<p role="alert">{error}</p>{/if}
      <h1>{saved.displayForm}</h1>
      <p>{saved.translation ?? "Saved without translation"}</p>
      <p>{saved.isExistingWord ? `Seen ${saved.encounterCount} times · New Encounter saved` : "First Encounter saved"}</p>
      <button class="primary" disabled={busy} onclick={undo}>Undo</button>
    {:else if candidate}
      <small>{editing ? "Editing" : "Captured"}</small>
      {#if error}<p role="alert">{error}</p>{/if}
      {#if ocrNeedsConfirmation}
        {#if ocrCandidates.length > 1}<p role="status">识别到多个词，请保留你要收集的词汇。</p>{/if}
        <label>Vocabulary<input bind:value={selectedText} /></label>
        <label>Context sentence <small>Optional</small><textarea bind:value={sentence}></textarea></label>
        <button class="primary" disabled={busy || !selectedText.trim()} onclick={confirmOcrCandidate}>Confirm</button>
        <button class="secondary" onclick={cancel}>Cancel OCR</button>
      {:else if editing}
        <label>Selected text<input aria-label="Selected text" bind:value={selectedText} /></label>
        <label>Context<textarea aria-label="Context" bind:value={sentence}></textarea></label>
        <label>Translation <small>Optional</small><input aria-label="Translation (optional)" bind:value={translation} /></label>
        <button class="primary" disabled={busy || !selectedText.trim()} onclick={applyChanges}>Apply changes</button>
      {:else}
        {#if achievedConflict}
          <span class="status-tag">Achieved</span>
        {/if}
        <h1>{selectedText}</h1>
        <p>“{sentence}”</p>
        {#if translation}
          <p>{translation}</p>
          <p class="notice">{translationSource} → {translationTarget}</p>
        {:else if busy && translationAvailable}
          <p class="notice">Translating…</p>
        {:else if !translationAvailable}
          <p class="notice">Automatic translation is unavailable. You can add a translation manually.</p>
        {/if}
        {#if translationFailed}<button class="secondary" disabled={busy} onclick={() => translateCandidate(activeRequest, selectedText.trim())}>Retry translation</button>{/if}
        {#if translationStale}<p class="notice" role="status">Vocabulary changed. The translation may no longer match.</p><button class="secondary" disabled={busy} onclick={() => translateCandidate(activeRequest, selectedText.trim())}>Translate again</button>{/if}
        {#if achievedConflict}
          <p class="achieved-prompt" role="status">This Vocabulary Item is Achieved. Return it to Learning and save this Encounter?</p>
          <button class="primary" disabled={busy} onclick={restoreToLearningAndSave}>Return to Learning</button>
          <button class="secondary" disabled={busy} onclick={cancel}>Cancel</button>
        {:else}
          <button class="primary" disabled={busy} onclick={beginEditing}>Edit capture</button>
          <button class="secondary" disabled={busy} onclick={save}>Save capture</button>
        {/if}
      {/if}
    {:else if ocrOffer}
      <p role="alert">{error}</p>
      <button class="primary" disabled={ocrBusy} onclick={startOcr}>{ocrBusy ? "Starting OCR..." : ocrAttempted ? "Try Again" : "Start OCR"}</button>
      <button class="secondary" onclick={useManualCapture}>Manual Capture</button>
      <button class="secondary" onclick={cancel}>Cancel</button>
    {:else}
      <p>{error || "Ready to capture selected text."}</p>
    {/if}
  </section>
</main>

<style>
  :global(html), :global(body.windows-capture-document), :global(#app) { width: 100%; height: 100%; margin: 0; background: transparent; overflow: hidden; }
  :global(body.windows-capture-document) { min-width: 0; min-height: 0; }
  .windows-capture { box-sizing: border-box; display: flex; flex-direction: column; width: 100%; height: 100%; padding: 14px 16px; overflow: hidden; border: 1px solid #4b4f60; border-radius: 8px; color: #eeeef3; background: rgba(30, 31, 40, .98); box-shadow: 0 18px 48px rgba(0, 0, 0, .42); font: 14px "Segoe UI Variable", "Segoe UI", sans-serif; }
  header { display: flex; align-items: center; justify-content: space-between; color: #aaadba; font-size: 12px; cursor: move; user-select: none; }
  header span { display: flex; align-items: center; gap: 7px; }
  button { border: 0; color: inherit; background: transparent; cursor: pointer; }
  header button { font-size: 20px; cursor: pointer; }
  header i { width: 7px; height: 7px; border-radius: 50%; background: #7584ef; }
  .scrollable-content { min-height: 0; padding: 18px 4px 6px; overflow-x: hidden; overflow-y: auto; }
  h1 { margin: 3px 0 4px; font-size: 25px; }
  p { color: #b9bbc6; line-height: 1.45; }
  small { color: #858998; }
  .primary { padding: 8px 12px; border-radius: 7px; background: #6676e8; color: white; }
  .secondary { padding: 8px 12px; color: #c8cad4; }
  label { display: block; margin: 5px 0; color: #aaadba; font-size: 11px; }
  input, textarea { box-sizing: border-box; display: block; width: 100%; margin-top: 2px; padding: 5px 7px; border: 1px solid #55596a; border-radius: 5px; color: #eeeef3; background: #252732; font: inherit; }
  textarea { min-height: 42px; resize: vertical; }
  .notice { margin: 8px 0; font-size: 12px; }
  .status-tag { display: inline-block; margin-bottom: 4px; padding: 3px 7px; border: 1px solid #8b6f3d; border-radius: 999px; color: #f1ca78; background: #3a3020; font-size: 11px; font-weight: 700; }
  .achieved-prompt { padding: 8px 10px; border-left: 3px solid #d6a84f; background: rgba(214, 168, 79, .1); }
  button:focus-visible, input:focus-visible, textarea:focus-visible { outline: 2px solid #aab3ff; outline-offset: 2px; }
  @media (forced-colors: active) { .windows-capture { border-color: CanvasText; color: CanvasText; background: Canvas; box-shadow: none; } p, small, label, header { color: CanvasText; } .primary, .secondary, input, textarea { border: 1px solid ButtonText; color: ButtonText; background: ButtonFace; } }
  @media (prefers-reduced-motion: reduce) { .windows-capture, .windows-capture * { animation-duration: .01ms !important; animation-iteration-count: 1 !important; transition-duration: .01ms !important; } }
  @media (max-width: 320px), (min-resolution: 1.5dppx) and (max-width: 520px) { .windows-capture { font-size: 1rem; } .scrollable-content { padding-top: .75rem; } h1 { overflow-wrap: anywhere; font-size: 1.5rem; } .primary, .secondary { min-height: 2.5rem; } }
</style>
