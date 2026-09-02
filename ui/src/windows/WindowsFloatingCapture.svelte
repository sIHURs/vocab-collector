<script lang="ts">
  import { onMount } from "svelte";
  import type { CaptureCandidate, CaptureCard } from "../lib/types";
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
  let saved: CaptureCard | null = null;
  let busy = false;
  let error = "";
  let mounted = false;
  let dismissTimer: ReturnType<typeof setTimeout> | undefined;
  let ocrOffer = false;
  let ocrNeedsConfirmation = false;
  let ocrAvailable = false;
  let ocrEligibleFailure = false;
  let ocrCandidates: OcrCandidate[] = [];
  let selectedOcrIndex = 0;
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

  async function save(withoutTranslation: boolean) {
    if (!candidate || busy) return;
    const requestId = activeRequest;
    busy = true;
    error = "";
    try {
      const result = await captureBackend.save(requestId, {
        selectedText: selectedText.trim(),
        sentence: sentence.trim(),
        ...(translation.trim() ? { translation: translation.trim() } : {}),
      }, withoutTranslation);
      if (!mounted || requestId !== activeRequest) return;
      saved = result;
      editing = false;
      await captureBackend.releaseFocus();
      if (!mounted || requestId !== activeRequest) return;
      scheduleDismiss(requestId);
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
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
      clearDismissTimer();
      await captureBackend.hide(requestId);
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function confirmOcrCandidate() {
    const requestId = activeRequest;
    const selected = ocrCandidates[selectedOcrIndex];
    if (!selected) return;
    await captureBackend.confirmOcr(requestId, selectedOcrIndex);
    if (!mounted || requestId !== activeRequest) return;
    ocrNeedsConfirmation = false;
  }

  async function startOcr() {
    if (!activeRequest || ocrBusy) return;
    const requestId = activeRequest;
    ocrBusy = true;
    ocrAttempted = true;
    error = "";
    try {
      await captureBackend.startOcr(requestId);
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
      ocrOffer = true;
    } finally {
      if (mounted && requestId === activeRequest) ocrBusy = false;
    }
  }

  function chooseOcrCandidate(index: number) {
    selectedOcrIndex = index;
    const selected = ocrCandidates[index];
    if (!selected) return;
    candidate = {
      selectedText: selected.text,
      sentence: selected.text,
      selectionBounds: selected.bounds,
      origin: "ocr",
    };
    selectedText = selected.text;
    sentence = selected.text;
  }

  function handleCandidateKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const delta = event.key === "ArrowDown" ? 1 : -1;
      chooseOcrCandidate((selectedOcrIndex + delta + ocrCandidates.length) % ocrCandidates.length);
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void confirmOcrCandidate();
    }
  }

  async function cancel() {
    if (activeRequest) await captureBackend.hide(activeRequest);
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
      saved = null;
      error = "";
      editing = false;
      resetOcrState();
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
      error = event.failure.message;
      ocrEligibleFailure = event.failure.code === "empty_selection" || event.failure.code === "unsupported_element";
      ocrOffer = ocrAvailable && ocrEligibleFailure;
      ocrNeedsConfirmation = false;
      ocrCandidates = [];
      ocrBusy = false;
      ocrAttempted = false;
    });
    const ocr = captureBackend.listenOcrCandidate((event) => {
      if (event.requestId !== activeRequest) return;
      ocrCandidates = event.candidates;
      chooseOcrCandidate(0);
      error = "";
      ocrOffer = false;
      ocrNeedsConfirmation = true;
      if (event.ambiguous) void captureBackend.focus();
    });
    return () => {
      mounted = false;
      clearDismissTimer();
      document.body.classList.remove("windows-capture-document");
      ready.then((unlisten) => unlisten());
      failed.then((unlisten) => unlisten());
      ocr.then((unlisten) => unlisten());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="windows-capture" data-presentation="windows-capture" aria-label="Capture" onmouseenter={clearDismissTimer} onmouseleave={() => scheduleDismiss()} onfocusin={clearDismissTimer} onfocusout={() => scheduleDismiss()}>
  <header><span><i aria-hidden="true"></i>Vocab Collector</span><button aria-label="Cancel capture" onclick={cancel}>×</button></header>
  <section aria-live="polite" aria-busy={busy || ocrBusy}>
    {#if saved}
      <small>Saved</small>
      <h1>{saved.displayForm}</h1>
      <p>{saved.translation ?? "Saved without translation"}</p>
      <p>{saved.isExistingWord ? `Seen ${saved.encounterCount} times · New Encounter saved` : "First Encounter saved"}</p>
      <button class="primary" disabled={busy} onclick={undo}>Undo</button>
    {:else if candidate}
      <small>{editing ? "Editing" : "Captured"}</small>
      {#if error}<p role="alert">{error}</p>{/if}
      {#if ocrNeedsConfirmation}
        {#if ocrCandidates.length > 1}
          <p id="ocr-choice-help">Choose the text nearest the pointer.</p>
          <div class="candidate-list" role="listbox" aria-label="OCR candidates" aria-describedby="ocr-choice-help" aria-activedescendant={`ocr-candidate-${selectedOcrIndex}`} tabindex="0" onkeydown={handleCandidateKeydown}>
            {#each ocrCandidates as item, index}
              <button id={`ocr-candidate-${index}`} role="option" aria-selected={index === selectedOcrIndex} tabindex="-1" onclick={() => chooseOcrCandidate(index)}>{item.text}</button>
            {/each}
          </div>
        {:else}
          <h1>{selectedText}</h1>
          <p>“{sentence}”</p>
        {/if}
        <button class="primary" onclick={confirmOcrCandidate}>Confirm OCR candidate</button>
        <button class="secondary" onclick={cancel}>Cancel OCR</button>
      {:else if editing}
        <label>Selected text<input aria-label="Selected text" bind:value={selectedText} /></label>
        <label>Context<textarea aria-label="Context" bind:value={sentence}></textarea></label>
        <label>Translation <small>Optional</small><input aria-label="Translation (optional)" bind:value={translation} /></label>
        <button class="primary" disabled={busy || !selectedText.trim()} onclick={() => save(!translation.trim())}>Save capture</button>
      {:else}
        <h1>{selectedText}</h1>
        <p>“{sentence}”</p>
        <p class="notice">Automatic translation is unavailable on Windows. Add one manually, or save without it.</p>
        <button class="primary" onclick={beginEditing}>Edit capture</button>
        <button class="secondary" disabled={busy} onclick={() => save(true)}>Save without translation</button>
      {/if}
    {:else if ocrOffer}
      <p role="alert">{error}</p>
      <button class="primary" disabled={ocrBusy} onclick={startOcr}>{ocrBusy ? "Starting OCR..." : ocrAttempted ? "Retry OCR near pointer" : "Use OCR near pointer"}</button>
      <button class="secondary" onclick={cancel}>Cancel</button>
    {:else}
      <p>{error || "Ready to capture selected text."}</p>
    {/if}
  </section>
</main>

<style>
  :global(html), :global(body.windows-capture-document), :global(#app) { width: 100%; height: 100%; margin: 0; background: transparent; overflow: hidden; }
  :global(body.windows-capture-document) { min-width: 0; min-height: 0; }
  .windows-capture { box-sizing: border-box; width: 100%; height: 100%; padding: 14px 16px; border: 1px solid #4b4f60; border-radius: 8px; color: #eeeef3; background: rgba(30, 31, 40, .98); box-shadow: 0 18px 48px rgba(0, 0, 0, .42); font: 14px "Segoe UI Variable", "Segoe UI", sans-serif; }
  header { display: flex; align-items: center; justify-content: space-between; color: #aaadba; font-size: 12px; }
  header span { display: flex; align-items: center; gap: 7px; }
  button { border: 0; color: inherit; background: transparent; cursor: pointer; }
  header button { font-size: 20px; }
  header i { width: 7px; height: 7px; border-radius: 50%; background: #7584ef; }
  section { min-height: 190px; padding: 18px 4px 6px; }
  h1 { margin: 3px 0 4px; font-size: 25px; }
  p { color: #b9bbc6; line-height: 1.45; }
  small { color: #858998; }
  .primary { padding: 8px 12px; border-radius: 7px; background: #6676e8; color: white; }
  .secondary { padding: 8px 12px; color: #c8cad4; }
  label { display: block; margin: 5px 0; color: #aaadba; font-size: 11px; }
  input, textarea { box-sizing: border-box; display: block; width: 100%; margin-top: 2px; padding: 5px 7px; border: 1px solid #55596a; border-radius: 5px; color: #eeeef3; background: #252732; font: inherit; }
  textarea { min-height: 42px; resize: vertical; }
  .notice { margin: 8px 0; font-size: 12px; }
  .candidate-list { display: grid; gap: 4px; max-height: 100px; margin: 6px 0 10px; overflow: auto; outline: none; }
  .candidate-list button { padding: 6px 8px; border-radius: 5px; text-align: left; }
  .candidate-list button[aria-selected="true"] { background: #6676e8; color: white; }
  button:focus-visible, input:focus-visible, textarea:focus-visible, .candidate-list:focus-visible { outline: 2px solid #aab3ff; outline-offset: 2px; }
  @media (forced-colors: active) { .windows-capture { border-color: CanvasText; color: CanvasText; background: Canvas; box-shadow: none; } p, small, label, header { color: CanvasText; } .primary, .secondary, input, textarea, .candidate-list button { border: 1px solid ButtonText; color: ButtonText; background: ButtonFace; } .candidate-list button[aria-selected="true"] { color: HighlightText; background: Highlight; } }
  @media (prefers-reduced-motion: reduce) { .windows-capture, .windows-capture * { animation-duration: .01ms !important; animation-iteration-count: 1 !important; transition-duration: .01ms !important; } }
  @media (max-width: 320px), (min-resolution: 1.5dppx) and (max-width: 520px) { .windows-capture { overflow: auto; font-size: 1rem; } section { min-height: 0; padding-top: .75rem; } h1 { overflow-wrap: anywhere; font-size: 1.5rem; } .primary, .secondary { min-height: 2.5rem; } }
</style>
