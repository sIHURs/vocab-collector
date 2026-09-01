<script lang="ts">
  import { onMount } from "svelte";
  import type { CaptureCandidate, CaptureCard } from "../lib/types";
  import {
    tauriWindowsCaptureBackend,
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
    });
    return () => {
      mounted = false;
      clearDismissTimer();
      document.body.classList.remove("windows-capture-document");
      ready.then((unlisten) => unlisten());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="windows-capture" data-presentation="windows-capture" aria-label="Capture" onmouseenter={clearDismissTimer} onmouseleave={() => scheduleDismiss()} onfocusin={clearDismissTimer} onfocusout={() => scheduleDismiss()}>
  <header><span><i aria-hidden="true"></i>Vocab Collector</span><button aria-label="Cancel capture" onclick={cancel}>×</button></header>
  <section aria-live="polite">
    {#if saved}
      <small>Saved</small>
      <h1>{saved.displayForm}</h1>
      <p>{saved.translation ?? "Saved without translation"}</p>
      <p>{saved.isExistingWord ? `Seen ${saved.encounterCount} times · New Encounter saved` : "First Encounter saved"}</p>
      <button class="primary" disabled={busy} onclick={undo}>Undo</button>
    {:else if candidate}
      <small>{editing ? "Editing" : "Captured"}</small>
      {#if error}<p role="alert">{error}</p>{/if}
      {#if editing}
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
    {:else}
      <p>Ready to capture selected text.</p>
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
</style>
