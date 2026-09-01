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

  async function beginEditing() {
    await captureBackend.focus();
    editing = true;
  }

  async function save(withoutTranslation: boolean) {
    if (!candidate || busy) return;
    busy = true;
    error = "";
    try {
      saved = await captureBackend.save(activeRequest, {
        selectedText: selectedText.trim(),
        sentence: sentence.trim(),
        ...(translation.trim() ? { translation: translation.trim() } : {}),
      }, withoutTranslation);
      editing = false;
      await captureBackend.releaseFocus();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  async function undo() {
    if (!saved || busy) return;
    busy = true;
    error = "";
    try {
      await captureBackend.undo(activeRequest, saved.encounterId);
      saved = null;
      await captureBackend.hide(activeRequest);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  async function cancel() {
    if (activeRequest) await captureBackend.hide(activeRequest);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") void cancel();
  }

  onMount(() => {
    document.body.classList.add("windows-capture-document");
    const ready = captureBackend.listenReady((event) => {
      activeRequest = event.requestId;
      candidate = event.candidate;
      selectedText = event.candidate.selectedText;
      sentence = event.candidate.sentence;
      translation = "";
      saved = null;
      error = "";
      editing = false;
    });
    return () => {
      document.body.classList.remove("windows-capture-document");
      ready.then((unlisten) => unlisten());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="windows-capture" data-presentation="windows-capture" aria-label="Capture">
  <header><span><i aria-hidden="true"></i>Vocab Collector</span><button aria-label="Cancel capture" onclick={cancel}>×</button></header>
  <section aria-live="polite">
    {#if saved}
      <small>Saved</small>
      <h1>{saved.displayForm}</h1>
      <p>{saved.translation ?? "Saved without translation"}</p>
      <button class="primary" disabled={busy} onclick={undo}>Undo</button>
    {:else if candidate}
      <small>{editing ? "Editing" : "Captured"}</small>
      {#if error}<p role="alert">{error}</p>{/if}
      {#if editing}
        <label>Selected text<input aria-label="Selected text" bind:value={selectedText} /></label>
        <label>Context<textarea aria-label="Context" bind:value={sentence}></textarea></label>
        <label>Translation <small>Optional</small><input aria-label="Translation (optional)" bind:value={translation} /></label>
        <button class="primary" disabled={busy || !selectedText.trim() || !sentence.trim()} onclick={() => save(!translation.trim())}>Save capture</button>
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
