<script lang="ts">
  import { onMount } from "svelte";
  import type { CaptureCandidate } from "../lib/types";
  import {
    tauriWindowsCaptureBackend,
    type WindowsCaptureBackend,
  } from "./captureBackend";

  export let captureBackend: WindowsCaptureBackend = tauriWindowsCaptureBackend;
  let activeRequest = "";
  let candidate: CaptureCandidate | null = null;
  let editing = false;

  async function beginEditing() {
    await captureBackend.focus();
    editing = true;
  }

  async function finishEditing() {
    await captureBackend.releaseFocus();
    editing = false;
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
    {#if candidate}
      <small>{editing ? "Editing" : "Captured"}</small>
      <h1>{candidate.selectedText}</h1>
      <p>“{candidate.sentence}”</p>
      {#if editing}
        <button class="primary" onclick={finishEditing}>Done editing</button>
      {:else}
        <button class="primary" onclick={beginEditing}>Edit capture</button>
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
</style>
